mod brewery;
mod event;
mod food_truck;
use brewery::Brewery;
use event::Event;
use food_truck::FoodTruck;

use eframe::{epaint::Vec2, run_native, NativeOptions};
use tailwag::gui::widgets::item_manager::item_manager::ItemManager;
use tailwag::orm::data_manager::rest_api::RestApiDataProvider;
use tailwag_gui_tools::widgets::widget_selector::MultiItemManager;
use tailwag_web_service::application::WebService;

#[tokio::main]
async fn main() {
    let svc = WebService::builder("Brewery Food Truck Finder")
        .with_resource::<Brewery>()
        .with_resource::<Event>()
        .with_resource::<FoodTruck>()
        // .with_webhook("/brewery/{id}/")
        .build_service();

    // let gui_thread = tokio::spawn(run_gui());
    // _ = run_gui().await;
    svc.run().await.expect("App failed to exit gracefully");
    // _ = gui_thread.await;
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
