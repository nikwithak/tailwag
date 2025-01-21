/// This contains all of the library code needed for configuring the Postgres DB for a type.
#[cfg(feature = "orm")]
pub use tailwag_orm as orm;

/// Various common utilities for coding. Things like string manipulation, common struct patterns, etc.
#[cfg(feature = "utils")]
pub use tailwag_utils as utils;

/// Contains library code for creating a REST web service from data types.
#[cfg(feature = "web_service")]
pub use tailwag_web_service as web;

#[macro_export]
macro_rules! derive_magic {
    ($i:item) => {
        #[derive(
            Clone, // Needed to be able to create an editable version from an Arc<Brewery> without affecting the saved data.
            Debug,
            Default,
            serde::Deserialize,                  // Needed for API de/serialization
            serde::Serialize,                    // Needed for API de/serialization
            // sqlx::FromRow,                       // Needed for DB connectivity
            tailwag::macros::GetTableDefinition, // Creates the data structure needed for the ORM to work.
            tailwag::macros::Insertable,
            tailwag::macros::Updateable,
            tailwag::macros::Deleteable,
            tailwag::macros::Filterable,
            tailwag::macros::BuildRoutes, // Creates the functions needed for a REST service (full CRUD)
            tailwag::macros::Id,
            // tailwag::macros::AsEguiForm, // Renders the object into an editable form for an egui application.
            tailwag::macros::Display,
            tailwag::forms::macros::GetForm,
        )]
        $i
    };
}

use tailwag_macros;
#[cfg(feature = "macros")]
pub mod macros {
    pub use super::derive_magic;
    pub use crate::utils::inline_macros as inline;
    pub use inline::*;
    use tailwag_macros;
    pub use tailwag_macros::*;
    pub use tailwag_orm_macros::*;
}

/// Crate containing GUI application logic and common widgets.
/// TODO: Rename to `egui` to represent the underlying framework
/// GUI features not yet released
// #[cfg(feature = "gui")]
// pub use tailwag_gui_tools as gui;

/// Various common utilities for coding. Things like string manipulation, common struct patterns, etc.
#[cfg(feature = "forms")]
pub use tailwag_forms as forms;

pub mod prelude {
    pub use super::derive_magic;
    pub use super::orm::data_manager::traits::WithFilter;
    pub use super::orm::data_manager::PostgresDataProvider;
    pub use super::orm::queries::filterable_types::*;
    pub use super::web::application::http::route::*;
    pub use super::web::application::WebService;
    pub use super::web::HttpError;
    pub use super::web::HttpResult;
}
