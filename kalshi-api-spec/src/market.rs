use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    money::Price,
    ticker::{EventTicker, MarketTicker},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrikeType {
    Between,
    Greater,
    Less,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    ticker: MarketTicker,
    event_ticker: EventTicker,
    title: String,
    open_time: DateTime<Utc>,
    close_time: DateTime<Utc>,
    strike_type: StrikeType,
    floor_strike: Option<i64>,
    cap_strike: Option<i64>,
    yes_bid_dollars: Price,
    yes_ask_dollars: Price,
    no_bid_dollars: Price,
    no_ask_dollars: Price,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketResponse {
    market: Market,
}
