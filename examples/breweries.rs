use eframe::{epaint::Vec2, run_native, NativeOptions};
/// This example is a simple application that will
use sqlx::postgres::PgPoolOptions;
use std::fmt::Display;
use std::path::Path;
use tailwag::forms::GetForm;
use tailwag::gui::widgets::item_manager::item_manager::ItemManager;
use tailwag::orm::data_manager::{rest_api::RestApiDataProvider, PostgresDataProvider};
use tailwag::{
    orm::data_manager::GetTableDefinition,
    web::{application::WebServiceApplication, traits::rest_api::BuildRoutes},
};
use tailwag_gui_tools::widgets::widget_selector::MultiItemManager;
use tailwag_macros::derive_magic;
use tokio;

derive_magic! {
    pub struct Brewery {
        id: uuid::Uuid, // Immutable, and assigned after create. The macros make assumptions because the name is `id` and the type is Uuid
        name: String,
        description: Option<String>, // Option<_> tells the ORM to make the DB columns nullable.
        website_url: Option<String>,
        food_truck_extraction_regex: Option<String>,
        // location: Geometry,
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

impl Default for Brewery {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "New Brewery".to_string(),
            description: None,
            website_url: None,
            food_truck_extraction_regex: None,
        }
    }
}

// Alias the DataProvider type. Not necessarily required.
pub type Breweries = PostgresDataProvider<Brewery>;

#[tokio::main]
async fn main() {
    save_form_def("./out/forms/brewery.json").unwrap();
    let web_svc = tokio::spawn(run_server());
    _ = run_gui().await;
    _ = web_svc.await;
}

fn save_form_def(filepath: &str) -> Result<(), std::io::Error> {
    let form_def = serde_json::to_string(&Brewery::get_form())?;
    let dir = Path::new(filepath).parent().unwrap_or(Path::new(filepath));
    std::fs::create_dir_all(dir).expect("Failed to create directories.");
    std::fs::write(filepath, form_def.as_bytes())?;
    Ok(())
}

async fn run_gui() {
    let provider =
        RestApiDataProvider::<Brewery>::from_endpoint("http://localhost:8081".to_string());
    // Uncomment to run without the web service (e.g. if Postgres isn't set up)
    // let provider = InMemoryDataProvider::<Brewery>::default();

    let app = MultiItemManager::default()
        .add("Breweries", ItemManager::from_data_provider(provider.clone()))
        .add("Food Trucks", ItemManager::from_data_provider(provider));

    // Standard egui shit
    let mut native_options = NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(640.0, 480.0));
    run_native("Breweries", native_options, Box::new(|_| Box::new(app)))
        .expect("Application crashed.");
}

async fn run_server() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        // TODO: Unhardcode things
        .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .await
        .expect("[DATABASE] Unable to obtain connection to database. Is postgres running?");

    let provider = PostgresDataProvider {
        table_definition: Brewery::get_table_definition(),
        db_pool: pool,
        _t: Default::default(),
    };

    let app: WebServiceApplication = WebServiceApplication::new_with_auth("Berewries")
        .await
        .add_routes("/brewery", Brewery::build_routes(provider).await);

    app.run_app().await;
}
