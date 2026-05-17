use std::collections::HashMap;

use sqlx::Postgres;

use crate::BuildSql;

use crate::data_definition::table::{ColumnValue, DatabaseTableDefinition, Identifier, ObjectRepr};

use super::insert::InsertStatement;

pub struct UpdateStatement {
    pub(crate) table_name: Identifier,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    pub(crate) object_repr: HashMap<Identifier, ColumnValue>,
}

impl UpdateStatement {
    pub fn new(
        table_def: DatabaseTableDefinition,
        object_map: HashMap<Identifier, ColumnValue>,
    ) -> Self {
        Self {
            table_name: table_def.table_name.clone(),
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

impl UpdateStatement {
    /// This is a HACKY approach to getting upserts to work as a part of insert / update.
    /// It ignores OneToMany relationships (they should not be passed in at all, when used properly),
    /// and does a (SELECT id FROM _table_name_) for OneToOne tables.
    /// Hacky, but gets the job done.
    pub fn build_sql_no_build_children(
        &self,
        prefix: &str,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        // builder.push(format!("UPDATE {} SET ", self.table_name));
        builder.push("UPDATE SET ");

        let Some(ColumnValue::Uuid(id)) = self.object_repr.get(&Identifier::new_unchecked("id"))
        else {
            panic!("Received update without ID - shouldn't be possble.")
        };
        for (i, (column, value)) in self
            .object_repr
            .iter()
            .filter(|(k, _v)| &k as &str != &Identifier::new_unchecked("id") as &str)
            .enumerate()
        {
            match value {
                ColumnValue::Boolean(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Int(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Float(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::String(val) | ColumnValue::Json(val) => {
                    builder.push(column).push(" = ").push_bind(val.to_string())
                },
                ColumnValue::Timestamp(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Uuid(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::OneToOne {
                    child_table,
                    value: _,
                } => {
                    let fk_name = &Identifier::new_unchecked("id"); // TODO: This is the ONLY FK supported for now. Eventually replace with dynamic FKs.
                    builder
                        .push(column)
                        .push(" = ")
                        .push(format!("(SELECT {fk_name} FROM {prefix}{child_table})"))
                    // Safe to inject directly, because `Identifier` is validated at runtime.
                },
                ColumnValue::OneToMany {
                    ..
                } => todo!(),
            };
            if i < &self.object_repr.len() - 2 {
                // .len()-2 because we've stripped `id` from it above.
                builder.push(", ");
            }
        }
        builder.push(format!(" WHERE {}.id=", &self.table_name)).push_bind(*id);
    }
}

impl BuildSql for UpdateStatement {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        // HACK: This actually does an UPSERT instead of a raw UPDATE.
        // We accomplish this by converting toa n INSERT statement, then
        // building the insert with the UPSERT flag set.
        InsertStatement {
            table_name: self.table_name.clone(),
            object_repr: self.object_repr.clone(),
        }
        .build_insert_sql(true, builder);
    }
}
