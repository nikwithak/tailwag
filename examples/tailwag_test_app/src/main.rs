use sqlx::postgres::PgPoolOptions;
use tailwag::{
    orm::data_manager::{GetTableDefinition, PostgresDataProvider},
    web::{application::WebServiceApplication, traits::rest_api::BuildRoutes},
};

#[derive(
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    Clone,
    tailwag::macros::Queryable,
    tailwag::macros::GetTableDefinition,
    tailwag::macros::Insertable,
    tailwag::macros::Deleteable,
    tailwag::macros::Updateable,
    tailwag::macros::BuildRoutes,
)]
pub struct Item {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
}

#[tokio::main]
async fn main() {
    /////////////////////////////////////
    // Boilerplate for DB connectivity //
    /////////////////////////////////////
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .await
        .expect("[DATABASE] Unable to obtain connection to database");

    let provider = PostgresDataProvider {
        table_definition: Item::get_table_definition(),
        db_pool: pool,
        _t: Default::default(),
    };

    let app: WebServiceApplication =
        WebServiceApplication::default().add_routes("/item", Item::build_routes(provider).await);

    app.run().await;
    todo!()
}
