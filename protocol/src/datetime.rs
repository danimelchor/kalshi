use anyhow::{Result, anyhow};
use bincode::{Decode, Encode};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Etc::UTC, Tz};
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
pub struct SerializableDateTime {
    timestamp: i64,
    tz: String,
}

impl From<DateTime<Tz>> for SerializableDateTime {
    fn from(dt: DateTime<Tz>) -> Self {
        Self {
            timestamp: dt.timestamp(),
            tz: dt.timezone().name().to_string(),
        }
    }
}

impl From<DateTime<Utc>> for SerializableDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self {
            timestamp: dt.timestamp(),
            tz: UTC.name().to_string(),
        }
    }
}

impl TryFrom<SerializableDateTime> for DateTime<Tz> {
    type Error = anyhow::Error;

    fn try_from(sdt: SerializableDateTime) -> Result<Self, Self::Error> {
        let tz: Tz = sdt
            .tz
            .parse()
            .map_err(|e| anyhow!("Failed to parse timezone '{}': {}", sdt.tz, e))?;

        tz.timestamp_opt(sdt.timestamp, 0)
            .single()
            .ok_or_else(|| anyhow!("Invalid timestamp {} for timezone {}", sdt.timestamp, tz))
    }
}
