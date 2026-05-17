use std::path::Path;

use serde::{Deserialize, Serialize};
pub use tailwag_forms_macros as macros;

#[derive(Serialize, Deserialize)]
pub struct Form {
    pub button_name: String,
    pub fields: Vec<FormField>,
}

impl Form {
    #[allow(unused)]
    pub fn save_json(
        &self,
        filepath: &str,
    ) -> Result<(), std::io::Error> {
        let json_def = serde_json::to_string(self)?;
        let dir = Path::new(filepath).parent().unwrap_or(Path::new(filepath));
        std::fs::create_dir_all(dir).expect("Failed to create directories.");
        std::fs::write(filepath, json_def.as_bytes())?;
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Number,
    Checkbox,
}

#[derive(Default, Serialize, Deserialize)]
pub struct FormField {
    name: String, // TODO: Verify this is unique with underscores
    #[serde(rename = "type")]
    field_type: InputType,
    is_required: bool,
    label: Option<String>,
    placeholder: Option<String>,
    initial_value: Option<String>,
    validate_regex: Option<String>,
}

impl FormField {
    pub fn text(name: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: InputType::Text,
            ..Default::default()
        }
    }

    pub fn password(name: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: InputType::Password,
            ..Default::default()
        }
    }
    pub fn number(name: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: InputType::Number,
            ..Default::default()
        }
    }
    pub fn boolean(name: &str) -> Self {
        Self::bool(name)
    }
    pub fn bool(name: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: InputType::Checkbox,
            ..Default::default()
        }
    }
}

macro_rules! builder_method {
    ($attr_name:ident, Some(String)) => {
        pub fn $attr_name(
            mut self,
            $attr_name: &str,
        ) -> Self {
            self.$attr_name = Some($attr_name.to_string());
            self
        }
    };
    ($attr_name:ident, Some($type:ty)) => {
        pub fn $attr_name(
            mut self,
            $attr_name: $type,
        ) -> Self {
            self.$attr_name = Some($attr_name);
            self
        }
    };
    ($attr_name:ident, String) => {
        pub fn $attr_name(
            mut self,
            $attr_name: &str,
        ) -> Self {
            self.$attr_name = $attr_name.to_string();
            self
        }
    };
    ($attr_name:ident, $type:ty) => {
        pub fn $attr_name(
            mut self,
            $attr_name: $type,
        ) -> Self {
            self.$attr_name = $attr_name;
            self
        }
    };
}

/// Builders
impl FormField {
    builder_method!(is_required, bool);
    builder_method!(label, Some(String));
    builder_method!(placeholder, Some(String));
    builder_method!(initial_value, Some(String));
    builder_method!(validate_regex, Some(String));
    builder_method!(name, String);
}

pub trait GetForm {
    fn get_form() -> Form;
}
