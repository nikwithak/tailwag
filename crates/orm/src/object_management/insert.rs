use sqlx::Postgres;

use crate::{
    data_definition::table::{ColumnValue, Identifier, ObjectRepr},
    BuildSql,
};

use super::update::UpdateStatement;

#[derive(Clone)]
pub struct InsertStatement {
    pub(crate) table_name: Identifier,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    pub(crate) object_repr: ObjectRepr,
}

impl InsertStatement {
    pub fn new(
        table_name: Identifier,
        object_map: ObjectRepr,
    ) -> Self {
        Self {
            table_name,
            object_repr: object_map,
        }
    }

    pub fn table_name(&self) -> Identifier {
        self.table_name.clone()
    }
    pub fn object_repr(&self) -> &ObjectRepr {
        &self.object_repr
    }
}

// REF:
// with
//      oneonechild as (insert into public.oneonechild (id, value) values (gen_random_uuid(), 'MyChild') returning *), -- first: insert all children
//      parent as (insert into public.parent (id, value, oneonechild_id) values (gen_random_uuid(), 'Myself', (select id from oneonechild)) returning *), -- then: insert self
//      onemanychild2 as (insert into public.child (id, parent_id, value) values (gen_random_uuid(), (select id from parent), 'OneManyChi2222ld1') returning *), -- then: insert all one to many children
//      onemanychild3 as (insert into public.child (id, parent_id, value) values (gen_random_uuid(), (select id from parent), 'OneManyChild2') returning *) -- all using 'with'
// --THEN: Not quite sure. A select of some kind.

// select json_agg(*) from parent as mypar
// inner join oneonechild ooc on ooc.id = mypar."oneonechild_id"
// inner join onemanychild2 omc on omc.parent_id = mypar.id
// inner join onemanychild3 omc2 on omc2.parent_id = mypar.id;

impl InsertStatement {
    fn build_consecutive_inserts(
        &self,
        prefix: &str,
        upsert: bool,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        // This block is a hacky way of propogating my ID to the OneToMany tables later - I need to update this to actually build a SQL for a FK relationship to an existing object.
        let parent_id = {
            // TODO: `parent_id` is assumed - eventually I need to make this actually parse the FK relationships.
            // ^^^^^ A(nother) big refactor is probably due to pass around references to the entire column instead of just the column name. Shouldn't have too much overhead, since the columns are already Arc'd under the hood.
            // TODO: THis is still very hacky. Need to handle a "Parent" type of column relationship.
            let Some(ColumnValue::Uuid(parent_id)) =
                self.object_repr.get(&Identifier::new_unchecked("id"))
            else {
                panic!("ID field required for parent/child relationships.")
            };
            parent_id.clone()
        };

        let mut raw_values: Vec<(Identifier, &ColumnValue)> = Vec::new();
        let mut one_to_one_inserts = Vec::new();
        let mut one_to_many_inserts = Vec::new();
        for (column, value) in &self.object_repr {
            match value {
                ColumnValue::OneToOne {
                    ..
                } => {
                    one_to_one_inserts.push((column, value));
                },
                ColumnValue::OneToMany {
                    ..
                } => one_to_many_inserts.push((column, value)),
                _ => raw_values.push((column.clone(), value)),
            }
        }

        // Build ONETOONE children
        let mut one_to_one_iter = one_to_one_inserts.into_iter().peekable();
        while let Some((child_col_name, col_value)) = one_to_one_iter.next() {
            let ColumnValue::OneToOne {
                child_table,
                value,
            } = col_value
            else {
                panic!("Wrong value type received when building one_to_one insert tables. This should not happen.")
            };

            InsertStatement {
                table_name: child_table.clone(),
                object_repr: *value.clone(),
            }
            .build_consecutive_inserts(prefix, true, builder);
            raw_values.push((child_col_name.clone(), col_value));
            // if one_to_one_iter.peek().is_some() {
            builder.push(", ");
            // }
        }

        // Build MYSELF
        {
            let table_name = &self.table_name;
            let col_names_joined =
                raw_values.iter().map(|entry| &*(entry.0)).collect::<Vec<_>>().join(", ");
            builder.push(format!(
                "{prefix}{table_name} as (INSERT INTO {table_name} ({col_names_joined}) VALUES (",
            ));
            // Begin insert values
            let mut values_iter = raw_values.iter().peekable();
            while let Some((_col_name, value)) = values_iter.next() {
                match value {
                    ColumnValue::Boolean(val) => builder.push_bind(*val),
                    ColumnValue::Int(val) => builder.push_bind(*val),
                    ColumnValue::Float(val) => builder.push_bind(*val),
                    ColumnValue::String(val) | ColumnValue::Json(val) => {
                        builder.push_bind(val.to_string())
                    },
                    ColumnValue::Timestamp(val) => builder.push_bind(*val),
                    ColumnValue::Uuid(val) => builder.push_bind(*val),
                    ColumnValue::OneToOne {
                        child_table: col_name,
                        value: _,
                    } => {
                        let fk_name = &Identifier::new_unchecked("id"); // TODO: This is the ONLY FK supported for now. Eventually replace with dynamic FKs.
                        builder.push(format!("(SELECT {fk_name} FROM {prefix}{col_name})"))
                        // Safe to inject directly, because `Identifier` is validated at runtime.
                    },
                    _ => todo!("This type of insert is not supported yet."),
                };
                if values_iter.peek().is_some() {
                    builder.push(", ");
                }
            }
            // End insert values
            builder.push(")");
            if upsert {
                // TODO: Another hardcode of `id`. Also this isn't modeled out as Postgres tokens, I've kinda given up on that idea.
                let update_statement = UpdateStatement {
                    table_name: self.table_name.clone(),
                    object_repr: raw_values
                        .into_iter()
                        .map(|(ident, val)| (ident.clone(), val.clone()))
                        .collect(),
                };
                builder.push(" ON CONFLICT (id) DO ");
                update_statement.build_sql_no_build_children(prefix, builder);
                // builder.push()
            }
            builder.push(" RETURNING * ");
            builder.push(")");
        }

        // Build ONETOMANY children)
        let mut one_to_many_iter = one_to_many_inserts.into_iter().peekable();
        while let Some((_col_name, stmt)) = one_to_many_iter.next() {
            // TODO: [PERFORMANCE] Some ugly overhead with the clone here...
            let ColumnValue::OneToMany {
                child_table,
                values,
            } = stmt.clone()
            else {
                panic!("Wrong value type received when building one_to_many insert tables. This should not happen.");
            };
            // TODO: I'm faking it here by adding a parent_id to the insert. In future, this should be able to pull from the INSERT result, as above.
            let insert_stmts = values.into_iter().map(|row| InsertStatement {
                table_name: child_table.clone(),
                object_repr: *row,
            });

            for (i, mut insert_stmt) in insert_stmts.enumerate() {
                insert_stmt.object_repr.insert(
                    Identifier::new_unchecked(format!("{}_id", self.table_name)),
                    ColumnValue::Uuid(parent_id),
                );

                builder.push(", "); // This should ALWAYS have at least one statement before it.
                let prefix = format!("{}_{}", &prefix, i);
                insert_stmt.build_consecutive_inserts(&prefix, true, builder);
            }
        }
    }

    pub(crate) fn build_insert_sql(
        &self,
        upsert: bool,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        let table_name = self.table_name.clone();
        builder.push("WITH ");
        self.build_consecutive_inserts("", upsert, builder);
        // Need *something* after the "WITH _ as (INSERT ....) statements"
        builder.push(format!("SELECT * FROM {table_name};"));
    }
}

impl BuildSql for InsertStatement {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        self.build_insert_sql(false, builder);
    }
}
