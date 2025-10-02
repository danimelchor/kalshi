use crate::station::Station;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, strum_macros::Display, ValueEnum)]
pub enum Model {
    HRRR,
}

impl Model {
    pub fn computed_grid_location_and_info(
        &self,
        station: Station,
    ) -> ((usize, usize), (usize, usize)) {
        match (self, station) {
            (Model::HRRR, Station::KNYC) => ((1553, 698), (1799, 1059)),
        }
    }

    pub fn max_runs(&self) -> usize {
        match self {
            Model::HRRR => 18,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ComputeOptions {
    Compute,
    Precomputed,
}
