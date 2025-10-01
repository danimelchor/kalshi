use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DateTimeZoned {
    timestamp: DateTime<Utc>,
    zone: Tz,
}

impl From<DateTimeZoned> for DateTime<Tz> {
    fn from(dt: DateTimeZoned) -> Self {
        dt.timestamp.with_timezone(&dt.zone)
    }
}

impl From<DateTime<Tz>> for DateTimeZoned {
    fn from(dt: DateTime<Tz>) -> Self {
        Self {
            timestamp: dt.with_timezone(&Utc),
            zone: dt.timezone(),
        }
    }
}

impl From<DateTime<Utc>> for DateTimeZoned {
    fn from(dt: DateTime<Utc>) -> Self {
        Self {
            timestamp: dt,
            zone: Tz::from_str("UTC").unwrap(),
        }
    }
}

impl Display for DateTimeZoned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt_tz: DateTime<Tz> = (*self).into();
        write!(f, "{}", dt_tz)
    }
}

impl Debug for DateTimeZoned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt_tz: DateTime<Tz> = (*self).into();
        write!(f, "{}", dt_tz)
    }
}
