use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    market::Market,
    ticker::{EventTicker, SeriesTicker},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollateralReturnType {
    Binary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    event_ticker: EventTicker,
    series_ticker: SeriesTicker,
    title: String,
    sub_title: String,
    mutually_exclusive: bool,
    strike_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    event: Event,
    markets: Vec<Market>,
}
