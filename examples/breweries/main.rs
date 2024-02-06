mod brewery;
mod event;
mod food_truck;
use brewery::Brewery;
use event::Event;
use food_truck::FoodTruck;

use eframe::{epaint::Vec2, run_native, NativeOptions};
use sqlx::postgres::PgPoolOptions;
use std::path::Path;
use tailwag::forms::GetForm;
use tailwag::gui::widgets::item_manager::item_manager::ItemManager;
use tailwag::orm::data_manager::{rest_api::RestApiDataProvider, PostgresDataProvider};
use tailwag::{
    orm::data_manager::GetTableDefinition,
    web::{application::WebServiceApplication, traits::rest_api::BuildRoutes},
};
use tailwag_gui_tools::widgets::widget_selector::MultiItemManager;
use tailwag_web_service::application::WebServiceBuilder;

#[tokio::main]
async fn main() {
    let mut svc = WebServiceBuilder::new("Brewery Food Truck Finder")
        .with_resource::<Brewery>()
        .with_resource::<Event>()
        .with_resource::<FoodTruck>()
        .connect_data_sources()
        .await;

    let web_svc = tokio::spawn(svc.run());
    _ = run_gui().await;
    _ = web_svc.await;
}

/// Automate all below here
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
    let native_options = NativeOptions {
        initial_window_size: Some(Vec2::new(640.0, 480.0)),
        ..Default::default()
    };
    run_native("Breweries", native_options, Box::new(|_| Box::new(app)))
        .expect("Application crashed.");
}
