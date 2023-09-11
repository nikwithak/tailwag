use tailwag::{
    macros::derive_magic,
    orm::{
        data_definition::database_definition::DatabaseDefinition, data_manager::GetTableDefinition,
    },
    web::application::WebServiceApplication,
};

#[derive(
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    Clone,
    tailwag::macros::Queryable,
    tailwag::macros::GetTableDefinition,
    tailwag::macros::Insertable,
    // tailwag::macros::BuildCrudRoutes,
)]
pub struct Item {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
}

#[tokio::main]
async fn main() {
    // TODO: Goal to make functions / logic easy to override
    let data_model: DatabaseDefinition = DatabaseDefinition::new_unchecked("postgres")
        .table(Item::get_table_definition())
        .into();

    let app: WebServiceApplication = WebServiceApplication::default();

    app.run().await;
    todo!()

    // DataModelRestServiceDefinition::default()
    // .with_resource::<Item>("/item")
    // .run_app();
}
