use chrono_tz::{Tz, US::Eastern};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::coords::LatLon;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, strum_macros::Display, ValueEnum)]
pub enum Station {
    KNYC,
}

impl Station {
    pub fn latlon(&self) -> LatLon {
        match self {
            Station::KNYC => LatLon::new(40.78333, -73.96667),
        }
    }

    pub fn timezone(&self) -> Tz {
        match self {
            Station::KNYC => Eastern,
        }
    }

    pub fn area_code(&self) -> &'static str {
        match self {
            Station::KNYC => "NWS",
        }
    }
    pub fn city(&self) -> &'static str {
        match self {
            Station::KNYC => "NYC",
        }
    }
}
