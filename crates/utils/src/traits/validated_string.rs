use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

/// Defines validation rules for a `&str`.
///
/// Implement this trait to created a new `ValidatedConstantString` type, which
/// enforces the rules defined in `ValidateString::validate`. See [ValidatedConstantString]
/// for more detail.
pub trait ValidateString {
    type Error: Debug;
    fn validate(value: &str) -> Result<(), Self::Error>;
}

/// An immutable string type, which enforces the string structure
/// via the [ValidateString] implementation `V`.
///
/// Use this struct to enforce rules on identifiers, where they must follow
/// a specific pattern.
///
/// The inline macro [create_validated_string_type] can be used to quickly
/// create new types with enforced validation rules based on a regular expression.
/// ```
#[derive(Clone)]
pub struct ValidatedConstantString<V>
where
    V: ValidateString,
{
    value: Arc<String>,
    _validator: PhantomData<V>,
}

impl<V: ValidateString> Deref for ValidatedConstantString<V> {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl<V> ValidatedConstantString<V>
where
    V: ValidateString,
{
    pub fn new<S: Into<String>>(value: S) -> Result<Self, V::Error> {
        let value: String = value.into();
        match V::validate(&value) {
            Ok(_) => Ok(Self {
                value: Arc::new(value),
                _validator: PhantomData::default(),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn new_unchecked<S: Into<String>>(value: S) -> Self {
        let value: String = value.into();
        Self::new(value)
            .expect(&format!("Invalid string passed to ValidatedConstantString::new_unchecked()",))
    }
}

/// Creates a ValidatedConstantString for the given regex pattern.
///
/// IMPORTANT NOTE: Due to constraints around macro_rules!, you will encounter namespace
/// collissions if you use this multiple times within the same scope. To get around this
/// restriction you may need to do
///
/// Examples:
///
/// TODO
#[macro_export]
macro_rules! create_validated_string_type {
    ($validator_name:ident=$regex:tt) => {
        type $validator_name = super::ValidatedConstantString<RegexValidator>;
        pub(crate) struct RegexValidator;
        impl super::ValidateString for RegexValidator {
            type Error = String;
            fn validate(value: &str) -> Result<(), Self::Error> {
                const REGEX_STRING: &'static str = $regex;
                let regex = regex::Regex::new(REGEX_STRING);
                match regex {
                    Ok(regex) => {
                        if regex.is_match(value) {
                            Ok(())
                        } else {
                            Err(format!("Input does not match regex '{}'", REGEX_STRING))
                        }
                    },
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    };
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use regex::Regex;

    use super::*;
    #[test]
    fn test_regex() {
        create_validated_string_type!(RegexType = "^testing$");
        assert_eq!(Ok(()), RegexValidator::validate("testing"));
        assert_eq!(
            Err("Input does not match regex '^testing$'".to_string()),
            RegexValidator::validate("testing    ")
        );
    }

    #[test]
    fn test_postgres_regex() {
        create_validated_string_type!(RegexType = "^[a-zA-Z0-9_]+$");
        assert_eq!(
            Ok(()),
            RegexValidator::validate("asjdiofjaeJOWJFOWEJF___WJFIOWJF02389asjdkjaisf")
        );
        assert_eq!(
            Err("Input does not match regex '^[a-zA-Z0-9_]+$'".to_string()),
            RegexValidator::validate("asjdiofjaeJOWJrOWEJF __WJFIOWJF02389asjdkjaisf")
        );
        assert_eq!(
            Err("Input does not match regex '^[a-zA-Z0-9_]+$'".to_string()),
            RegexValidator::validate("1=1")
        );
    }
}
