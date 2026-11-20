use std::fmt::Display;

use sqlx::Postgres;

pub mod data_definition;
pub mod data_manager;
pub mod executor;
pub mod migration;
pub mod object_management;
pub mod queries;

#[derive(Debug)]
pub enum Error {
    Unknown(String),
    Sqlx(sqlx::Error),
    DataIntegrity(String),
    HttpError(reqwest::Error),
    MutexLock(String),
    IoError(std::io::Error),
    InvalidPath,
}
pub type OrmError = Error;
pub type OrmResult<T> = Result<T, OrmError>;
macro_rules! impl_from {
    ($enum:ident::$variant:ident($ty:ty)) => {
        impl From<$ty> for $enum {
            fn from(val: $ty) -> Self {
                Self::$variant(val)
            }
        }
    };
}
impl_from!(Error::Sqlx(sqlx::Error));
impl_from!(Error::IoError(std::io::Error));
impl_from!(Error::Unknown(String));
impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Unknown(val.into())
    }
}
impl_from!(Error::HttpError(reqwest::Error));
impl Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl<'a, T> From<LockResult<MutexGuard<'a, T>>> for Error {
//     fn from(value: LockResult<MutexGuard<'a, T>>) -> Self {
//         Self::MutexLock(value.to_string())
//     }
// }

// TODO: Move this somewhere else (not lib.rs)
/// Converts the struct into a SQL statement (or component)
pub trait AsSql {
    fn as_sql(&self) -> String;
}

pub trait BuildSql {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    );
}

/// Fallback default used where a type does not implement `Default`.
///
/// The generated `Insertable` code (and entity `Default` impls) use this for
/// fields whose types do not implement `std::default::Default` - currently
/// `time::PrimitiveDateTime`, which mirrors chrono's `NaiveDateTime` that used
/// to provide a `Default` impl. A blanket `impl<T: Default> OrDefault for T`
/// is intentionally avoided: it conflicts with the impl below because upstream
/// `time` could add a `Default` impl for `PrimitiveDateTime` (an alias of
/// `PlainDateTime`), which the compiler cannot rule out.
pub trait OrDefault {
    fn or_default() -> Self;
}

impl OrDefault for time::PrimitiveDateTime {
    fn or_default() -> Self {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(1970, time::Month::January, 1)
                .expect("1970-01-01 is a valid calendar date"),
            time::Time::MIDNIGHT,
        )
    }
}

impl<T> OrDefault for Option<T> {
    fn or_default() -> Self {
        None
    }
}

/// Serde helper for `time::PrimitiveDateTime` fields.
///
/// The Postgres manager deserializes rows through `serde_json::from_value`
/// where `time`'s builtin impl rejects the ISO-8601a string that `to_json` emits.
/// This module round-trips timestamps as plain ISO strings in both modes.
pub mod pg_ts {
    use serde::{de, Deserialize, Deserializer, Serializer};
    use time::{Date, PrimitiveDateTime, Time};

    pub fn serialize<S: Serializer>(
        value: &PrimitiveDateTime,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&value.to_string().replace(' ', "T"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de_: D) -> Result<PrimitiveDateTime, D::Error> {
        let raw = String::deserialize(de_)?;
        parse(&raw).ok_or_else(|| {
            de::Error::invalid_value(de::Unexpected::Str(&raw), &"an ISO-8601 timestamp")
        })
    }

    pub fn opt<'de, D: Deserializer<'de>>(de_: D) -> Result<Option<PrimitiveDateTime>, D::Error> {
        let raw = Option::<String>::deserialize(de_)?;
        match raw {
            None => Ok(None),
            Some(s) => parse(&s).map(Some).ok_or_else(|| {
                de::Error::invalid_value(de::Unexpected::Str(&s), &"an ISO-8601 timestamp")
            }),
        }
    }

    pub fn opt_serialize<S: Serializer>(
        value: &Option<PrimitiveDateTime>,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serialize(v, ser),
            None => ser.serialize_none(),
        }
    }

    fn parse(s: &str) -> Option<PrimitiveDateTime> {
        let (date_part, time_part) = s.split_once(['T', ' '])?;
        let mut d = date_part.split('-');
        let year: i32 = d.next()?.parse().ok()?;
        let month: u8 = d.next()?.parse().ok()?;
        let day: u8 = d.next()?.parse().ok()?;
        let (clock, fraction) = match time_part.split_once('.') {
            Some((c, f)) => (c, f),
            None => (time_part, ""),
        };
        let mut t = clock.split(':');
        let hour: u8 = t.next()?.parse().ok()?;
        let minute: u8 = t.next()?.parse().ok()?;
        let second: u8 = t.next()?.parse().ok()?;
        // Postgres emits 0-6 fractional digits; pad to microseconds.
        let mut micro = String::from(fraction);
        micro.push_str("000000");
        let microsecond: u32 = micro[..6.min(micro.len())].parse().ok()?;
        let date = Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()?;
        let time = Time::from_hms_micro(hour, minute, second, microsecond).ok()?;
        Some(PrimitiveDateTime::new(date, time))
    }
}
