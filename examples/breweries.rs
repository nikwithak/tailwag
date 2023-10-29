use eframe::{epaint::Vec2, run_native, NativeOptions};
/// This example is a simple application that will
use sqlx::postgres::PgPoolOptions;
use std::{borrow::BorrowMut, fmt::Display, str::FromStr};
use tailwag::gui::widgets::item_manager::{item_edit_page::AsEguiForm, item_manager::ItemManager};
use tailwag::orm::data_manager::{rest_api::RestApiDataProvider, PostgresDataProvider};
use tailwag::{
    orm::data_manager::GetTableDefinition,
    web::{application::WebServiceApplication, traits::rest_api::BuildRoutes},
};
use tokio;
use uuid::Uuid;

/// All the derive macros, to add functionality. Eventually I hope to condense these into one single derive macro (for the base case)
/// where all the other pieces are derived from one.
#[derive(
    Clone, // Needed to be able to create an editable version from an Arc<Brewery> without affecting the saved data.
    Debug,
    Default,
    serde::Deserialize,                  // Needed for API de/serialization
    serde::Serialize,                    // Needed for API de/serialization
    sqlx::FromRow,                       // Needed for DB connectivity
    tailwag::macros::Queryable, // TODO: Clean up how these macros work. I think this one is literally just a marker right now.
    tailwag::macros::GetTableDefinition, // Creates the data structure needed for the ORM to work.
    tailwag::macros::Insertable,
    tailwag::macros::Updateable,
    tailwag::macros::Deleteable,
    tailwag::macros::BuildRoutes, // Creates the functions needed for a REST service (full CRUD)
    tailwag::macros::AsEguiForm, // Renders the object into an editable form for an egui application.
)]
pub struct Brewery {
    id: uuid::Uuid, // Immutable, and assigned after create. The macros make assumptions because the name is `id` and the type is Uuid
    name: String,
    description: Option<String>, // Option<_> tells the ORM to make the DB columns nullable.
    website_url: Option<String>,
    food_truck_extraction_regex: Option<String>,
    // location: Geometry,
}

// TODO: Derive macro this.
// Makes the `id` field accessible without being editable
impl tailwag::orm::data_manager::rest_api::Id for Brewery {
    fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

// TODO: Derive macro this
impl Display for Brewery {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", &self.name)
    }
}
// Alias the DataProvider type. Not necessarily required.
pub type Breweries = PostgresDataProvider<Brewery>;

///
#[tokio::main]
async fn main() {
    let web_svc = tokio::spawn(run_server());
    _ = run_gui().await;
    _ = web_svc.await;
}

async fn run_gui() {
    let provider =
        RestApiDataProvider::<Brewery>::from_endpoint("http://localhost:3001".to_string());
    let app = ItemManager::<Brewery>::from_data_provider(provider);

    // Standard egui shit
    let mut native_options = NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(640.0, 480.0));
    run_native("Breweries", native_options, Box::new(|_| Box::new(app)))
        .expect("Application crashed.");
}

async fn run_server() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .await
        .expect("[DATABASE] Unable to obtain connection to database");

    let provider = PostgresDataProvider {
        table_definition: Brewery::get_table_definition(),
        db_pool: pool,
        _t: Default::default(),
    };

    let app: WebServiceApplication =
        WebServiceApplication::default().add_routes("/item", Brewery::build_routes(provider).await);

    app.run().await;
}
