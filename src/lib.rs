/// This contains all of the library code needed for configuring the Postgres DB for a type.
#[cfg(feature = "orm")]
pub use tailwag_orm as orm;

/// Various common utilities for coding. Things like string manipulation, common struct patterns, etc.
#[cfg(feature = "utils")]
pub use tailwag_utils as utils;

/// Contains library code for creating a REST web service from data types.
#[cfg(feature = "web_service")]
pub use tailwag_web_service as web;

/// Crate containing all the macro code.
/// TODO: Move the macros into their respective cargo workspaces, to avoid circular dependency issues (which I currently have to dance around)
#[cfg(feature = "macros")]
pub use tailwag_macros as macros;

/// Crate containing GUI application logic and common widgets.
/// TODO: Rename to `egui` to represent the underlying framework
/// GUI features not yet released
// #[cfg(feature = "gui")]
// pub use tailwag_gui_tools as gui;

/// Various common utilities for coding. Things like string manipulation, common struct patterns, etc.
#[cfg(feature = "forms")]
pub use tailwag_forms as forms;

pub mod prelude {
    pub use super::macros::derive_magic;
    pub use super::orm::data_manager::traits::WithFilter;
    pub use super::orm::data_manager::PostgresDataProvider;
    pub use super::orm::queries::filterable_types::*;
    pub use super::web::application::http::route::*;
    pub use super::web::application::WebService;
}
