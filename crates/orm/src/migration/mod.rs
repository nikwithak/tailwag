mod alter_table;
mod create_table;
#[allow(clippy::module_inception)]
mod migration;

pub use alter_table::*;
pub use create_table::*;
// #[cfg(migrations)]
pub use migration::*;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{
        data_definition::table::{
            DatabaseColumnType, DatabaseTableDefinition, Identifier, TableColumn,
        },
        migration::{
            AlterColumn, AlterColumnAction, AlterTable, AlterTableAction, MigrationAction,
        },
        BuildSql,
    };

    use super::Migration;

    fn table_2() -> DatabaseTableDefinition {
        type T = TableColumn;
        DatabaseTableDefinition::new("table_2")
            .unwrap()
            .column(T::string("string_nullable").unwrap())
            .column(T::bool("bool").unwrap().non_null())
            .column(T::int("int").unwrap().non_null())
            .column(T::float("float").unwrap().non_null())
            .column(T::timestamp("timestamp").unwrap().non_null())
            .column(T::uuid("uuid").unwrap().non_null())
            .into()
    }

    fn table_1() -> DatabaseTableDefinition {
        DatabaseTableDefinition::new("table_1")
            .unwrap()
            .column(TableColumn::string("string_nullable").unwrap())
            .column(TableColumn::bool("bool").unwrap().non_null())
            .column(TableColumn::int("int").unwrap().non_null())
            .column(TableColumn::float("float").unwrap().non_null())
            .column(TableColumn::timestamp("timestamp").unwrap().non_null())
            .column(TableColumn::uuid("uuid").unwrap().non_null())
            .into()
    }

    fn table_3() -> DatabaseTableDefinition {
        type T = TableColumn;
        DatabaseTableDefinition::new("table_3")
            .unwrap()
            .column(T::timestamp("created_at").unwrap().non_null())
            .column(T::uuid("id").unwrap().non_null().pk())
            .into()
    }

    fn table_4() -> DatabaseTableDefinition {
        DatabaseTableDefinition::new("table_4")
            .unwrap()
            .column(TableColumn::uuid("id").unwrap().pk().non_null())
            .column(TableColumn::timestamp("created_at").unwrap().non_null())
            .into()
    }

    #[test]
    fn as_sql_generates_sql_script() {
        // Arrange
        let migration = Migration {
            actions: vec![MigrationAction::AlterTable(AlterTable {
                table_name: Identifier::new("table_1".to_string()).unwrap(),
                actions: vec![
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("bool".to_string()).unwrap(),
                        actions: vec![
                            AlterColumnAction::SetType(DatabaseColumnType::String),
                            AlterColumnAction::SetNullability(true),
                        ],
                    }),
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("int".to_string()).unwrap(),
                        actions: vec![AlterColumnAction::SetType(DatabaseColumnType::Float)],
                    }),
                    AlterTableAction::AddColumn(
                        TableColumn::string("new_column").unwrap().non_null().into(),
                    ),
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                        actions: vec![AlterColumnAction::SetNullability(false)],
                    }),
                    AlterTableAction::DropColumn(Identifier::new("timestamp".to_string()).unwrap()),
                ],
            })],
        };

        // Act
        let mut query_builder = sqlx::QueryBuilder::new("");
        migration.build_sql(&mut query_builder);
        let result_sql = query_builder.into_sql();

        // Assert
        // NOTE: This tests is a little finicky - does not account for different whitespace.
        //       This should be fine, but has room for improvement.
        let mut queries = result_sql.split('\n').filter(|l| !l.is_empty()).collect::<Vec<&str>>();
        let mut expected_queries: Vec<&str> = vec![
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN bool TYPE VARCHAR, ALTER COLUMN bool DROP NOT NULL;",
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN int TYPE FLOAT;",
            "ALTER TABLE IF EXISTS table_1 ADD COLUMN IF NOT EXISTS new_column VARCHAR NOT NULL;",
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN string_nullable SET NOT NULL;",
            "ALTER TABLE IF EXISTS table_1 DROP COLUMN IF EXISTS timestamp;",
        ];

        while !queries.is_empty() && !expected_queries.is_empty() {
            assert_eq!(queries.pop(), expected_queries.pop());
        }

        assert!(
            queries.is_empty() && expected_queries.is_empty(),
            "Number of queries did not match."
        );
    }

    #[test]
    fn compare_tables_new_database() {
        //     let before = None;
        //     let after = DatabaseDefinition::new("my_database")
        //         .unwrap()
        //         .table(table_1())
        //         .table(table_2())
        //         .into();

        //     let migration = Migration::compare(before.as_ref(), &after).unwrap();

        //     assert_eq!(
        //         migration,
        //         Migration {
        //             actions: vec![
        //                 MigrationAction::CreateTable(CreateTable::new(table_1())),
        //                 MigrationAction::CreateTable(CreateTable::new(table_2())),
        //             ],
        //         }
        //     );
        todo!("Need to refactor this")
    }

    #[test]
    fn compare_tables_no_diff() {
        // let before: DatabaseDefinition<()> = DatabaseDefinition::new("my_database")
        //     .unwrap()
        //     .table(table_1())
        //     .table(table_2())
        //     .into();
        // let after: DatabaseDefinition<()> = DatabaseDefinition::new("my_database")
        //     .unwrap()
        //     .table(table_1())
        //     .table(table_2())
        //     .into();

        // // Act
        // // let migration = Migration::compare(Some(&before), &after);
        // todo!("Need to fix these tests");

        // Assert
        // assert_eq!(migration, None);
        todo!("Need to rebuild the equality checks.");
    }

    // #[test]
    // fn compare_tables_builds_diff() {
    //     // Arrange
    //     let after_t1 = DatabaseTableDefinition::new("table_1")
    //         .unwrap()
    //         .column(TableColumn::string("string_nullable").unwrap().non_null())
    //         .column(TableColumn::string("bool").unwrap())
    //         .column(TableColumn::float("int").unwrap().non_null()) // Changing type for the test
    //         .column(TableColumn::float("float").unwrap().non_null())
    //         .column(TableColumn::uuid("uuid").unwrap().non_null())
    //         .column(TableColumn::string("new_column").unwrap().non_null());

    //     let after_t4 = DatabaseTableDefinition::new("table_4")
    //         .unwrap()
    //         .column(TableColumn::uuid("id").unwrap().pk().non_null())
    //         .column(TableColumn::timestamp("created_at").unwrap().non_null())
    //         .column(TableColumn::timestamp("updated_at").unwrap());

    //     let before = DatabaseDefinition::new("my_new_database")
    //         .unwrap()
    //         .table(table_1())
    //         .table(table_2())
    //         .table(table_4())
    //         .into();
    //     let after = DatabaseDefinition::new("my_new_database")
    //         .unwrap()
    //         .table(after_t1)
    //         .table(table_3())
    //         .table(after_t4)
    //         .into();

    //     // Act
    //     // let actual = Migration::compare(Some(&before), &after).unwrap();
    //     todo!("Need to rebuild the equality checks.");

    //     // Assert
    //     let expected = Migration {
    //         actions: vec![
    //             MigrationAction::AlterTable(AlterTable {
    //                 table_name: Identifier::new("table_1".to_string()).unwrap(),
    //                 actions: vec![
    //                     AlterTableAction::AlterColumn(AlterColumn {
    //                         column_name: Identifier::new("bool".to_string()).unwrap(),
    //                         actions: vec![
    //                             AlterColumnAction::SetType(DatabaseColumnType::String),
    //                             AlterColumnAction::SetNullability(true),
    //                         ],
    //                     }),
    //                     AlterTableAction::AlterColumn(AlterColumn {
    //                         column_name: Identifier::new("int".to_string()).unwrap(),
    //                         actions: vec![AlterColumnAction::SetType(DatabaseColumnType::Float)],
    //                     }),
    //                     AlterTableAction::AlterColumn(AlterColumn {
    //                         column_name: Identifier::new("string_nullable".to_string()).unwrap(),
    //                         actions: vec![AlterColumnAction::SetNullability(false)],
    //                     }),
    //                     AlterTableAction::DropColumn(
    //                         Identifier::new("timestamp".to_string()).unwrap(),
    //                     ),
    //                     AlterTableAction::AddColumn(
    //                         TableColumn::string("new_column").unwrap().non_null().into(),
    //                     ),
    //                 ],
    //             }),
    //             MigrationAction::DropTable(Identifier::new("table_2").unwrap()),
    //             MigrationAction::AlterTable(AlterTable {
    //                 table_name: Identifier::new("table_4".to_string()).unwrap(),
    //                 actions: vec![AlterTableAction::AddColumn(
    //                     TableColumn::timestamp("updated_at").unwrap().into(),
    //                 )],
    //             }),
    //             // MigrationAction::CreateTable(CreateTable::new(table_3())),
    //         ],
    //     };

    //     // assert_eq!(actual, expected);
    //     todo!("Need to rebuild the equality checks.");

    //     let mut builder = sqlx::QueryBuilder::new("");
    //     // actual.build_sql(&mut builder);
    //     todo!("Need to rebuild the equality checks.");
    //     println!("{}", builder.into_sql());
    // }
}
