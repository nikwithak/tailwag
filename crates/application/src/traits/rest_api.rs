use crate::application::http::route::Route;

pub trait BuildRoutes<T> {
    fn build_routes() -> Route;
}
