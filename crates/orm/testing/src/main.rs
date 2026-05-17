use derive_dataprovider_logic::{
    database_definition::{
        database_definition::DatabaseDefinition,
        migration::Migration,
        table_definition::{DatabaseColumnType, DatabaseTableDefinition, Identifier, TableColumn},
    },
    AsSql,
};
use sqlx::{postgres::PgPoolOptions, Executor};

fn table_2() -> DatabaseTableDefinition {
    DatabaseTableDefinition {
        table_name: Identifier::new("table_2".to_string()).unwrap(),
        columns: vec![
            TableColumn {
                column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                column_type: DatabaseColumnType::String,
                is_primary_key: false,
                is_nullable: true,
            },
            TableColumn {
                column_name: Identifier::new("bool".to_string()).unwrap(),
                column_type: DatabaseColumnType::Boolean,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("int".to_string()).unwrap(),
                column_type: DatabaseColumnType::Int,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("float".to_string()).unwrap(),
                column_type: DatabaseColumnType::Float,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("timestamp".to_string()).unwrap(),
                column_type: DatabaseColumnType::Timestamp,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("uuid".to_string()).unwrap(),
                column_type: DatabaseColumnType::Uuid,
                is_primary_key: false,
                is_nullable: false,
            },
        ],
    }
}

fn table_1() -> DatabaseTableDefinition {
    DatabaseTableDefinition {
        table_name: Identifier::new("table_1".to_string()).unwrap(),
        columns: vec![
            TableColumn {
                column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                column_type: DatabaseColumnType::String,
                is_primary_key: false,
                is_nullable: true,
            },
            TableColumn {
                column_name: Identifier::new("bool".to_string()).unwrap(),
                column_type: DatabaseColumnType::Boolean,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("int".to_string()).unwrap(),
                column_type: DatabaseColumnType::Int,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("float".to_string()).unwrap(),
                column_type: DatabaseColumnType::Float,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("timestamp".to_string()).unwrap(),
                column_type: DatabaseColumnType::Timestamp,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("uuid".to_string()).unwrap(),
                column_type: DatabaseColumnType::Uuid,
                is_primary_key: false,
                is_nullable: false,
            },
        ],
    }
}

fn table_3() -> DatabaseTableDefinition {
    DatabaseTableDefinition {
        table_name: Identifier::new("table_3".to_string()).unwrap(),
        columns: vec![
            TableColumn {
                column_name: Identifier::new("created_at".to_string()).unwrap(),
                column_type: DatabaseColumnType::Timestamp,
                is_primary_key: false,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("id".to_string()).unwrap(),
                column_type: DatabaseColumnType::Uuid,
                is_primary_key: true,
                is_nullable: false,
            },
        ],
    }
}

fn table_4() -> DatabaseTableDefinition {
    DatabaseTableDefinition {
        table_name: Identifier::new("table_4".to_string()).unwrap(),
        columns: vec![
            TableColumn {
                column_name: Identifier::new("id".to_string()).unwrap(),
                column_type: DatabaseColumnType::Uuid,
                is_primary_key: true,
                is_nullable: false,
            },
            TableColumn {
                column_name: Identifier::new("created_at".to_string()).unwrap(),
                column_type: DatabaseColumnType::Timestamp,
                is_primary_key: false,
                is_nullable: false,
            },
        ],
    }
}

#[tokio::main]
async fn main() {
    let old_def = DatabaseDefinition::new(
        "postgres".to_string(),
        vec![table_1(), table_2(), table_3(), table_4()],
    );

    let new_def = DatabaseDefinition::new(
        "postgres".to_string(),
        vec![
            table_1(),
            DatabaseTableDefinition {
                table_name: Identifier::new("table_5".to_string()).unwrap(),
                columns: vec![
                    TableColumn {
                        column_name: Identifier::new("id".to_string()).unwrap(),
                        column_type: DatabaseColumnType::String,
                        is_primary_key: true,
                        is_nullable: false,
                    },
                    TableColumn {
                        column_name: Identifier::new("created_at".to_string()).unwrap(),
                        column_type: DatabaseColumnType::Timestamp,
                        is_primary_key: false,
                        is_nullable: false,
                    },
                ],
            },
            DatabaseTableDefinition {
                table_name: Identifier::new("table_4".to_string()).unwrap(),
                columns: vec![
                    TableColumn {
                        column_name: Identifier::new("id".to_string()).unwrap(),
                        column_type: DatabaseColumnType::String,
                        is_primary_key: true,
                        is_nullable: false,
                    },
                    TableColumn {
                        column_name: Identifier::new("created_at".to_string()).unwrap(),
                        column_type: DatabaseColumnType::Timestamp,
                        is_primary_key: false,
                        is_nullable: false,
                    },
                    TableColumn {
                        column_name: Identifier::new("updated_at".to_string()).unwrap(),
                        column_type: DatabaseColumnType::Timestamp,
                        is_primary_key: false,
                        is_nullable: false,
                    },
                ],
            },
        ],
    );

    let migs = Migration::compare(Some(&old_def), &new_def).expect("");
    let sql = migs.as_sql();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();

    for sql in sql.split(";") {
        sqlx::query(&format!("{};", sql)).execute(&pool).await.unwrap();
    }
}
