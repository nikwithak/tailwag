use application::http::route::{IntoResponse, Response};

pub mod application;
pub mod auth;
mod components;
pub mod errors;
pub mod extras;
pub mod tasks;
pub mod traits;

#[derive(Debug)]
pub enum Error {
    BadRequest(String),
    InternalServerError(String),
    TaskSchedculingError(TaskError),
    Conflict,
    NotFound,
    Unauthorized,
    EntityTooLarge,
    UnsupportedMediaType,
}
use tasks::runner::TaskError;
pub use Error as HttpError;
pub type HttpResult<T> = Result<T, Error>;

/// Helper functions for quickly throwing errors from an API endpoint.
impl Error {
    pub fn bad_request<T>(msg: &str) -> HttpResult<T> {
        Err(Error::BadRequest(msg.into()))
    }
    pub fn internal_server_error<T>(msg: &str) -> HttpResult<T> {
        Err(Error::InternalServerError(msg.into()))
    }
    pub fn not_found<T>() -> HttpResult<T> {
        Err(Error::NotFound)
    }
    pub fn conflict<T>() -> HttpResult<T> {
        Err(Error::Conflict)
    }
    pub fn entity_too_large<T>() -> HttpResult<T> {
        Err(Error::EntityTooLarge)
    }
    pub fn unsupported_media_type<T>() -> HttpResult<T> {
        Err(Error::UnsupportedMediaType)
    }
    pub fn unauthorized<T>() -> HttpResult<T> {
        Err(Error::Unauthorized)
    }
}

impl<T: ToString> From<T> for Error {
    fn from(value: T) -> Self {
        Error::BadRequest(value.to_string())
    }
}

impl From<TaskError> for Error {
    fn from(value: TaskError) -> Self {
        Error::TaskSchedculingError(value)
    }
}

// impl From<tailwag_orm::Error> for Error {
//     fn from(value: tailwag_orm::Error) -> Self {
//         todo!()
//     }
// }

impl IntoResponse for crate::Error {
    fn into_response(self) -> Response {
        match self {
            Error::BadRequest(msg) => {
                log::warn!("[BAD REQUEST] {}", &msg);
                Response::bad_request()
            },
            Error::NotFound => Response::not_found(),
            Error::InternalServerError(msg) => {
                log::error!("[INTERNAL SERVER ERROR]: {}", &msg);
                Response::internal_server_error()
            },
            Error::Conflict => {
                log::warn!("[CONFLICT]");
                Response::conflict()
            },
            Error::TaskSchedculingError(task_error) => {
                log::error!("[TASK SCHEDULING ERROR]: {:?}", task_error);
                Response::internal_server_error()
            },
            Error::EntityTooLarge => {
                log::warn!("[ENTITY TOO LARGE]: ");
                Response::entity_too_large()
            },
            Error::UnsupportedMediaType => {
                log::warn!("[UNSUPPORTED MEDIA TYPE]: ");
                Response::unsupported_media_type()
            },
            Error::Unauthorized => {
                log::warn!("[UNAUTHORIZED]: ");
                Response::unauthorized()
            },
        }
    }
}

pub type ResponseResult<T> = core::result::Result<T, crate::Error>;

pub mod option_utils {
    use crate::{HttpError, HttpResult};

    trait Locked {}
    impl<T> Locked for Option<T> {}
    #[allow(private_bounds)]
    pub trait OrError<T>
    where
        Self: Locked,
    {
        fn or_404(self) -> HttpResult<T>;
        fn or_400(
            self,
            msg: &str,
        ) -> HttpResult<T>;
        fn or_401(self) -> HttpResult<T>;
        fn or_500(
            self,
            msg: &str,
        ) -> HttpResult<T>;
    }
    impl<T> OrError<T> for Option<T> {
        fn or_401(self) -> HttpResult<T> {
            self.ok_or(HttpError::Unauthorized)
        }
        fn or_404(self) -> HttpResult<T> {
            self.ok_or(HttpError::NotFound)
        }
        fn or_400(
            self,
            msg: &str,
        ) -> HttpResult<T> {
            self.ok_or(HttpError::BadRequest(msg.into()))
        }
        fn or_500(
            self,
            msg: &str,
        ) -> HttpResult<T> {
            self.ok_or(HttpError::InternalServerError(msg.into()))
        }
    }
}
