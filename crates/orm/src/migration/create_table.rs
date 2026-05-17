use crate::{
    data_definition::{data_system::TableDef, table::raw_data::TableDefinition},
    AsSql, BuildSql,
};

#[derive(Clone, Debug)]
pub struct CreateTable {
    pub(crate) table_definition: TableDef,
}

impl CreateTable {
    pub fn new(table: TableDef) -> Self {
        Self {
            table_definition: table,
        }
    }
}

impl BuildSql for CreateTable {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("CREATE TABLE IF NOT EXISTS ");
        // sql.push_bind(self.table_definition.table_name.to_string());
        sql.push(self.table_definition.table_name().to_string());
        sql.push(" (");
        let mut columns = self
            .table_definition
            .columns()
            .values()
            .filter(|col| match &col.column_type {
                // OneToMany and ManyToMany don't store the relationship in the parent
                crate::data_definition::table::DatabaseColumnType::OneToMany(_identifier, _) // Child stores a pointer to parent
                | crate::data_definition::table::DatabaseColumnType::ManyToMany(_identifier, _) => // Joint able links the tables
                {
                    false
                },
                _ => true,
            })
            .peekable();
        while let Some(column) = columns.next() {
            column.build_sql(sql);
            if columns.peek().is_some() {
                sql.push(",");
            }
        }
        sql.push(");");
    }
}

impl AsSql for CreateTable {
    fn as_sql(&self) -> String {
        let mut builder = sqlx::QueryBuilder::new("");
        self.build_sql(&mut builder);
        builder.into_sql()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use create_table::CreateTable;

    use crate::{
        data_definition::table::{DatabaseTableDefinition, Identifier, TableColumn},
        migration::create_table,
        AsSql,
    };

    #[test]
    fn as_sql_foreign_key_on_column_works() {
        todo!()
    }

    #[test]
    fn create_table_as_sql_works() -> Result<(), String> {
        let table_name = Identifier::new("new_table".to_string()).unwrap();
        let table_definition = DatabaseTableDefinition::new(&table_name)
            .unwrap()
            .column(TableColumn::new_uuid("uuid_pk_nonnull")?.non_null().pk())
            .column(TableColumn::new_string("string")?)
            .column(TableColumn::new_bool("bool_nonnull")?.non_null())
            .column(TableColumn::new_float("float_nonnull")?.non_null())
            .column(TableColumn::new_int("int")?)
            .column(TableColumn::new_timestamp("create_timestamp")?);

        let create_table = CreateTable {
            table_definition: Arc::new(table_definition),
        };

        // Act
        let queries = create_table.as_sql();
        // let mut queries = result_sql.split("\n").collect::<Vec<&str>>();

        #[rustfmt::skip]
        let expected_query = ["CREATE TABLE IF NOT EXISTS new_table (",
                "bool_nonnull BOOL NOT NULL,",
                "create_timestamp TIMESTAMP,",
                "float_nonnull FLOAT NOT NULL,",
                "int INT,",
                "string VARCHAR,",
                "uuid_pk_nonnull UUID NOT NULL PRIMARY KEY",
            " );"].join("");

        assert_eq!(queries, expected_query);
        Ok(())
    }

    //     #[test]
    //     fn create_table_one_to_one_works() -> Result<(), String> {
    //         let child_table: DatabaseTableDefinition<()> =
    //             DatabaseTableDefinition::<()>::new(&Identifier::new("child_table")?)?
    //                 .column(TableColumn::new_uuid("id")?.non_null().pk())
    //                 .column(TableColumn::new_int("value")?.non_null())
    //                 .into();
    //         let parent_table: DatabaseTableDefinition<_> =
    //             DatabaseTableDefinition::<()>::new(&Identifier::new("parent_table")?)?
    //                 .column(TableColumn::new_uuid("id")?.non_null().pk())
    //                 .column(
    //                     TableColumn::new_uuid("child_id")?.fk_to(
    //                         child_table.table_name.clone(),
    //                         child_table
    //                             .columns
    //                             .get(&Identifier::new("id")?)
    //                             .expect("Failed to get child table column")
    //                             .clone(),
    //                     ),
    //                 )
    //                 .into();

    //         let db: DatabaseDefinition<_> = DatabaseDefinitionBuilder::new("test_db")
    //             .unwrap()
    //             .add_table(child_table)
    //             .add_table(parent_table)
    //             .into();

    //         let mgirations = dbg!(Migration::compare(None, &db)).unwrap();
    //         let mut query_builder = QueryBuilder::new("");
    //         mgirations.build_sql(&mut query_builder);
    //         let sql = dbg!(query_builder.into_sql());
    //         assert_eq!(
    //             r#"CREATE TABLE IF NOT EXISTS child_table (id UUID NOT NULL PRIMARY KEY ,value INT NOT NULL);
    // CREATE TABLE IF NOT EXISTS parent_table (child_id UUID REFERENCES child_table (id),id UUID NOT NULL PRIMARY KEY );
    // "#,
    //             sql
    //         );
    //         Ok(())
    //     }
}
