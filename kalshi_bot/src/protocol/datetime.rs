use bincode::{Decode, Encode};
use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug, Encode, Decode, Clone)]
pub struct SerializableDateTime(i64);

impl From<DateTime<Utc>> for SerializableDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        SerializableDateTime(dt.timestamp())
    }
}

impl From<SerializableDateTime> for DateTime<Utc> {
    fn from(sd: SerializableDateTime) -> Self {
        Utc.timestamp_opt(sd.0, 0).unwrap() // unwrap is safe for i64 timestamps
    }
}
