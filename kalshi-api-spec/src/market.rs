use chrono::DateTime;
use chrono_tz::Tz;

use crate::{
    money::Price,
    ticker::{EventTicker, MarketTicker},
};

pub enum StrikeType {
    Between,
    Greater,
    Less,
}

pub struct Market {
    ticker: MarketTicker,
    event_ticker: EventTicker,
    title: String,
    open_time: DateTime<Tz>,
    close_time: DateTime<Tz>,
    strike_type: StrikeType,
    floor_strike: Option<i64>,
    cap_strike: Option<i64>,
    yes_bid_dollars: Price,
    yes_ask_dollars: Price,
    no_bid_dollars: Price,
    no_ask_dollars: Price,
}
