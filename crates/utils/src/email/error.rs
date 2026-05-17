use std::{env::VarError, fmt::Display};

use crate::{email::TemplateId, templates::TemplateError};

#[derive(Debug)]
pub enum EmailError {
    Serialization(serde_json::Error),
    Http(reqwest::Error),
    Initialization(VarError),
    TemplateNotFound(TemplateId),
    TemplateError(TemplateError),
}

impl Display for EmailError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<TemplateError> for EmailError {
    fn from(value: TemplateError) -> Self {
        EmailError::TemplateError(value)
    }
}
impl From<VarError> for EmailError {
    fn from(value: VarError) -> Self {
        EmailError::Initialization(value)
    }
}
impl From<reqwest::Error> for EmailError {
    fn from(value: reqwest::Error) -> Self {
        EmailError::Http(value)
    }
}
impl From<serde_json::Error> for EmailError {
    fn from(value: serde_json::Error) -> Self {
        EmailError::Serialization(value)
    }
}
