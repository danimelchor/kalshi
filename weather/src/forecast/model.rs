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

    pub fn stdev(&self, lead_time: usize) -> f64 {
        // Populated by hand from a notebook in research/
        match self {
            Model::HRRR => match lead_time {
                0 => 0.096049,
                1 => 1.028724,
                2 => 1.464776,
                3 => 1.789209,
                4 => 1.932346,
                5 => 2.078711,
                6 => 2.277409,
                7 => 2.365806,
                8 => 2.485674,
                9 => 2.575617,
                10 => 2.624595,
                11 => 2.668690,
                12 => 2.678444,
                13 => 2.636398,
                14 => 2.627608,
                15 => 2.629799,
                16 => 2.584544,
                17 => 2.558621,
                18 => 2.653955,
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ComputeOptions {
    Compute,
    Precomputed,
}
