use anyhow::anyhow;
use bincode::{Decode, Encode, de::Decoder, enc::Encoder, error::DecodeError};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Etc::UTC, Tz};
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
struct SerializableDateTime {
    timestamp: i64,
    tz: String,
}

impl From<&DateTime<Tz>> for SerializableDateTime {
    fn from(dt: &DateTime<Tz>) -> Self {
        Self {
            timestamp: dt.timestamp(),
            tz: dt.timezone().name().to_string(),
        }
    }
}

impl From<&DateTime<Utc>> for SerializableDateTime {
    fn from(dt: &DateTime<Utc>) -> Self {
        Self {
            timestamp: dt.timestamp(),
            tz: UTC.name().to_string(),
        }
    }
}

impl From<SerializableDateTime> for DateTime<Tz> {
    fn from(sdt: SerializableDateTime) -> Self {
        let tz: Tz = sdt
            .tz
            .parse()
            .map_err(|e| anyhow!("Failed to parse timezone '{}': {}", sdt.tz, e))
            .expect("Serialized timezone could not be deserialized");

        tz.timestamp_opt(sdt.timestamp, 0)
            .single()
            .unwrap_or_else(|| panic!("Invalid timestamp {} for timezone {}", sdt.timestamp, tz))
    }
}

impl<'de> Decode<'de> for DateTime<Tz> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let sdt = SerializableDateTime::decode(decoder)?;
        Ok(sdt.into())
    }
}

impl Encode<Context> for DateTime<Tz> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let srd: SerializableDateTime = self.into();
        SerializableDateTime::encode(&srd, encoder)
    }
}
