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
                0 => 0.039297,
                1 => 0.882159,
                2 => 1.313472,
                3 => 1.610562,
                4 => 1.812974,
                5 => 1.886600,
                6 => 1.936268,
                7 => 2.052986,
                8 => 2.069630,
                9 => 2.059552,
                10 => 2.053210,
                11 => 2.128324,
                12 => 2.085564,
                13 => 2.070359,
                14 => 2.088182,
                15 => 2.059963,
                16 => 2.075709,
                17 => 2.090827,
                18 => 2.090827,
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
