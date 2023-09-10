use tailwag::{
    macros::derive_magic,
    orm::{
        data_manager::GetTableDefinition,
        database_definition::database_definition::DatabaseDefinition,
    },
    web::application::WebServiceApplication,
};

derive_magic! {
    pub struct Item {
        id: uuid::Uuid,
        name: String,
        description: Option<String>,
    }
}

#[tokio::main]
async fn main() {
    // TODO: Goal to make functions / logic easy to override
    let data_model: DatabaseDefinition = DatabaseDefinition::new_unchecked("postgres")
        .table(Item::get_table_definition())
        .into();

    let app: WebServiceApplication = data_model.into();

    app.run().await;

    // DataModelRestServiceDefinition::default()
    // .with_resource::<Item>("/item")
    // .run_app();
}
