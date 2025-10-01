use crate::station::Station;
use bincode::{Decode, Encode};
use clap::ValueEnum;

#[derive(Debug, Copy, Clone, Decode, Encode, strum_macros::Display, ValueEnum)]
pub enum Model {
    HRRR,
}

impl Model {
    pub fn computed_grid_location_and_info(
        &self,
        station: &Station,
    ) -> ((usize, usize), (usize, usize)) {
        match (self, station) {
            (Model::HRRR, Station::KNYC) => ((1553, 698), (1799, 1059)),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ComputeOptions {
    Compute,
    Precomputed,
}
