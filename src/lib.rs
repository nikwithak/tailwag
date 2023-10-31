/// Does nothing yet. Eventually this will include the logic for deploying Applications and Systems to multiple types of infrastructure (e.g. one-machine servers, clusters, k8s, AWS, etc.)
#[cfg(feature = "deployer")]
pub use tailwag_deployer as deployer;

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
#[cfg(feature = "gui")]
pub use tailwag_gui_tools as gui;
