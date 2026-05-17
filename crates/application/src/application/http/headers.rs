use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader},
    ops::DerefMut,
};

use tailwag_macros::Deref;

use crate::{application::ConfigConstants, Error};

type HeaderName = String;
// type HeaderValue = String;
#[derive(Debug, Clone, Deref)]
pub struct HeaderValue {
    #[deref]
    inner: String,
    params: HashMap<String, String>,
}
impl Display for HeaderValue {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: Into<String>> From<T> for HeaderValue {
    fn from(s: T) -> Self {
        let s: String = s.into();
        Self {
            params: HeaderValue::parse_params(&s),
            inner: s,
        }
    }
}

impl HeaderValue {
    fn parse_params(val: &str) -> HashMap<String, String> {
        HashMap::from_iter(val.split(';').filter_map(|param| {
            param.trim().split_once('=').map(|(a, b)| {
                (
                    a.trim()
                        .trim_end_matches('"')
                        .trim_start_matches('"')
                        .to_lowercase()
                        .to_string(),
                    b.trim().trim_end_matches('"').trim_start_matches('"').to_string(),
                )
            })
        }))
    }

    pub fn get_params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn get_param(
        &self,
        key: &str,
    ) -> Option<&String> {
        // TODO: Make this ref instead, since we don't need to keep it.
        // Sturggle is that it introduces lifetimes, and lifetimes are ocntagious.
        // I guess in reality this should all be tied to the lifetime of the request *anyway*.
        self.params.get(key)
    }
}

#[derive(Debug, Deref)]
pub struct Headers {
    headers: HashMap<HeaderName, HeaderValue>,
}
impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.headers
    }
}

impl Headers {
    pub fn parse_params(params_str: &str) -> HashMap<String, String> {
        HashMap::from_iter(params_str.split(';').filter_map(|param| {
            param
                .trim()
                .split_once('=')
                .map(|(a, b)| (a.trim().to_lowercase(), b.trim().to_string()))
        }))
    }

    pub fn parse_headers<T: std::io::Read>(stream: &mut BufReader<T>) -> Result<Self, Error> {
        let mut headers = Headers::default();
        let mut line = String::new();

        let mut stream = std::io::Read::take(stream, ConfigConstants::headers_max_length());
        while stream.read_line(&mut line)? > 2 {
            headers.insert_parsed(&line)?;
            line = String::new();
        }
        Ok(headers)
    }
}

impl Default for Headers {
    fn default() -> Self {
        // Creates a sensible-defaults header set based on OWASP recommendations.
        // Ref: https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html
        Self {
            headers: vec![
                ("X-Frame-Options", "DENY"),
                ("X-Content-Type-Options", "nosniff"),
                ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        }
    }
}

impl Headers {
    pub fn insert_parsed(
        &mut self,
        header_line: &str,
    ) -> Result<(HeaderName, &HeaderValue), Error> {
        let Some((name, value)) = header_line.split_once(':') else {
            return Err(Error::BadRequest(format!("Failed to parse header: {}", header_line)));
        };

        let name = name.to_lowercase();
        self.headers.insert(name.clone(), value.trim().into());
        let value = self
            .headers
            .get(&name)
            .expect("We literally just added this to the map on the previous line.");
        Ok((name, value))
    }
    pub fn get(
        &self,
        header_name: &str,
    ) -> Option<&HeaderValue> {
        self.headers.get(&header_name.to_lowercase())
    }
}

impl From<Vec<(&str, &str)>> for Headers {
    fn from(value: Vec<(&str, &str)>) -> Self {
        Headers {
            headers: value.into_iter().map(|(name, val)| (name.into(), val.into())).collect(),
        }
    }
}
