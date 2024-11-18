use tailwag::prelude::*;
use uuid::Uuid;

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
pub struct ParentTable {
    id: Uuid,
    name: String,
    #[no_filter]
    child_table: Vec<ChildTable>,
}

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
pub struct ChildTable {
    id: Uuid,
    // #[serde(skip_deserializing)]
    #[serde(skip)]
    #[create_ignore]
    parent_id: Uuid,
    name: String,
}

#[tokio::main]
pub async fn main() {
    WebService::builder("One To Many relationships example service")
        .with_resource::<ParentTable>()
        .with_resource::<ChildTable>()
        .build_service()
        .run()
        .await
        .unwrap();
}
