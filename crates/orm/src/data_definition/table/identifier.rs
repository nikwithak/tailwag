use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};

/// Represents a Database Identifier, for column names and table names.
/// It's just a string under the hood, but forcing calls to use Identifier::new(String),
/// we are able to perform field validation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct Identifier {
    value: Arc<String>,
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // TODO: Once I'm confident that this is impossible (it should b), remove the validate() call  - it adds a string pass on every deref.
        self.validate()
            .unwrap_or_else(|_| panic!("Identifier {} is invalid - this should have been caught on create. If you are seeing this mesage then you have found a bug - please file a bug report", self.value()));
        &self.value
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl Identifier {
    fn validate(&self) -> Result<(), String> {
        if Self::is_valid(&self.value) {
            Ok(())
        } else {
            Err(format!(
                "Identifier {} contains invalid characters. [a-zA-Z0-9_.] are only allowed values.",
                &self.value
            ))
        }
    }

    fn is_valid(value: &str) -> bool {
        value
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'))
    }

    pub fn new<S: Into<String>>(value: S) -> Result<Self, String> {
        let identifier = Self {
            value: Arc::new(value.into()),
        };
        match identifier.validate() {
            Ok(()) => Ok(identifier),
            Err(e) => Err(e),
        }
    }

    pub fn new_unchecked<S: Into<String>>(value: S) -> Self {
        match Self::new(value) {
            Ok(ident) => ident,
            Err(e) => panic!("Identifier::new_unchecked() failed: {}", &e),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn as_str(&self) -> &str {
        &*self
    }
}
