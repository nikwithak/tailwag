#[allow(clippy::module_inception)]
mod web_service;
pub use web_service::*;
pub mod http;
pub mod middleware;
pub mod static_files;
pub mod stats;
pub mod threads;
pub use std::cell::OnceCell;

mod webhook;
pub use webhook::*;

macro_rules! const_from_env {
    ($fn_name:ident, $env_var_name:literal, $name:ident: $ty:ty = $val:expr) => {
        const $name: OnceCell<$ty> = OnceCell::new();
        pub fn $fn_name() -> u64 {
            *Self::$name.get_or_init(||
                std::env::var($env_var_name).ok().and_then(|s|s.parse().ok()).unwrap_or($val)
            )
        }
    };
    ($fn_name:ident, $env_var_name:literal, $name:ident = $val:expr) => {
        const_from_env!($fn_name, $env_var_name, $name: u64 = $val);
    };
}

/// A struct containing constants that can be overridden with by setting the corresponding ENV variable.
/// Contains helper functions to manage the OnceCell types.
pub struct ConfigConstants;
impl ConfigConstants {
    const_from_env!(
        request_line_max_length,
        "REQUEST_LINE_MAX_LENGTH",
        REQUEST_LINE_MAX_LENGTH = 8 * 1024
    ); // 8KB
    const_from_env!(headers_max_length, "HEADERS_MAX_LENGTH", HEADERS_MAX_LENGTH = 8 * 1024); // 8KB
    const_from_env!(
        max_content_length,
        "MAX_CONTENT_LENGTH",
        MAX_CONTENT_LENGTH = (50 * 1024 * 1024)
    ); // 50 MB
}
