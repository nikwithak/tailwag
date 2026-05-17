use tailwag_orm::data_manager::{traits::DataProvider, PostgresDataProvider};
use tailwag_web_service::{
    application::http::route::Response,
    auth::gateway::{self, extract_session},
};
use uuid::Uuid;

mod tailwag {
    pub use tailwag_forms as forms;
    pub use tailwag_macros as macros;
    pub use tailwag_orm as orm;
    pub use tailwag_web_service as web;
}

#[derive(
    Clone,
    Debug,
    Default,
    serde::Deserialize,
    serde::Serialize,
    // sqlx::FromRow,
    tailwag_orm_macros::GetTableDefinition,
    tailwag_orm_macros::Insertable,
    tailwag_orm_macros::Updateable,
    tailwag_orm_macros::Deleteable,
    tailwag_orm_macros::Filterable,
    tailwag::macros::BuildRoutes,
    tailwag_orm_macros::Id,
    tailwag::macros::Display,
    tailwag::forms::macros::GetForm,
)]
#[views(display_event, css, form, banner)]
pub struct Event {
    id: Uuid,
    start_time: chrono::NaiveDateTime,
    // end_time: chrono::NaiveDateTime,
    name: String,
    #[no_filter]
    description: Option<String>,
    // attendees: Vec<String>,
}

#[tokio::main]
async fn main() {
    tailwag_web_service::application::WebService::builder("My Events Service")
        .with_middleware(extract_session)
        .post_public("/login", gateway::login)
        .post_public("/register", gateway::register)
        .with_resource::<Event>()
        .build_service()
        .run()
        .await
        .unwrap();
}

fn css() -> Response {
    load_static("globals.css")
}
fn form() -> Response {
    load_static("form.html")
}
fn banner() -> Response {
    load_static("banner.jpg")
}

pub async fn display_event(events: PostgresDataProvider<Event>) -> Response {
    load_template("event.md.template", events.all().await.unwrap().next().unwrap())
}

// TODO: Merge this with the MIME type in `image_upload.rs`
fn get_content_type(filename: &str) -> &'static str {
    filename
        .split('.')
        .last()
        .map(|ext| match ext {
            "html" => "text/html",
            "css" => "text/css",
            "json" => "application/json",
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "png" => "image/png",
            "webp" => "image/webp",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        })
        .unwrap_or("application/octet-stream")
}

fn load_template(
    filename: &str,
    obj: Event,
) -> Response {
    tailwag_web_service::application::static_files::load_template(filename, obj)
}

fn load_static(filename: &str) -> Response {
    let Ok(file) = std::fs::read(format!("static/{}", filename)) else {
        return Response::bad_request();
    };
    // TODO: DRY out to MimeType type
    let mime_type = get_content_type(filename);

    Response::ok()
        .with_body(file)
        // TODO: Parse the file extension into a content-type MIME-Type
        .with_header("content-type", mime_type)
}
