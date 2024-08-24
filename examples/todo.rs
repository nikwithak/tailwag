use tailwag::web::application::WebService;

#[tokio::main]
async fn main() {
    #[derive(
        Clone,
        Debug,
        Default,
        serde::Deserialize,
        serde::Serialize,
        sqlx::FromRow,
        tailwag::macros::GetTableDefinition,
        tailwag::macros::Insertable,
        tailwag::macros::Updateable,
        tailwag::macros::Deleteable,
        tailwag::macros::Filterable,
        tailwag::macros::BuildRoutes,
        tailwag::macros::Id,
        tailwag::macros::Display,
        tailwag::forms::macros::GetForm,
    )]
    struct Todo {
        id: uuid::Uuid,
        title: String,
        description: String,
        due_date: chrono::NaiveDateTime,
    }
    WebService::builder("Todo Service")
        .with_resource::<Todo>()
        .build_service()
        .run()
        .await
        .expect("Web service crashed.");
}
