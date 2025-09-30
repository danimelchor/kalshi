use anyhow::{Result, anyhow};
use bytes::Bytes;
use chrono::{DateTime, Datelike, TimeDelta, Timelike};
use chrono_tz::{Tz, UTC};
use clap::ValueEnum;
use grib::{Grib2, SeekableGrib2Reader, SubMessage};
use reqwest::Client;
use std::io::Cursor;

use crate::{forecast::model::Model, station::Station, temperature::Temperature};

static BASE: &str = "https://nomads.ncep.noaa.gov/pub/data/nccf/com/hrrr/prod";
static FORECAST_TYPE: &str = "wrfsfcf";

const TEMPERATURE_NUMBER: u8 = 0;
const SURFACE_TYPE: u8 = 103;
const METERS_ABOVE_GROUND: i32 = 2;

#[derive(Debug)]
pub struct SingleWeatherForecast {
    pub temperature: Temperature,
    pub timestamp: DateTime<Tz>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ComputeOptions {
    Compute,
    Precomputed,
}

fn get_url(ts: DateTime<Tz>, lead_time: i64) -> String {
    let utc = ts.with_timezone(&UTC);
    let hh = format!("{:02}", utc.hour());
    let date = format!("{:04}{:02}{:02}", utc.year(), utc.month(), utc.day());
    format!("{BASE}/hrrr.{date}/conus/hrrr.t{hh}z.{FORECAST_TYPE}{lead_time:0>2}.grib2")
}

fn find_message<'a>(
    grib2: &'a Grib2<SeekableGrib2Reader<Cursor<&'a Bytes>>>,
) -> Result<SubMessage<'a, SeekableGrib2Reader<Cursor<&'a Bytes>>>> {
    for (_, submessage) in grib2.iter() {
        let discipline = submessage.indicator().discipline;

        // Ignore sections other than temperature
        if discipline != TEMPERATURE_NUMBER {
            continue;
        }

        let category = submessage.prod_def().parameter_category().unwrap();
        let parameter = submessage.prod_def().parameter_number().unwrap();

        // Ignore metrics other than temperature
        if parameter != TEMPERATURE_NUMBER || category != TEMPERATURE_NUMBER {
            continue;
        }

        // Only 2m above ground temperatures
        let (first, _second) = submessage.prod_def().fixed_surfaces().unwrap();
        if first.surface_type != SURFACE_TYPE || first.scaled_value != METERS_ABOVE_GROUND {
            continue;
        }

        return Ok(submessage);
    }

    Err(anyhow!("Failed to find submessage for 2m temperature"))
}

fn temp_closest_to_station<'a>(
    station: &Station,
    model: &Model,
    submessage: SubMessage<'a, SeekableGrib2Reader<Cursor<&'a Bytes>>>,
    compute_opts: ComputeOptions,
) -> Result<Temperature> {
    let latlon = submessage.latlons()?;
    let ijs = submessage.ij()?;
    let grid_shape = submessage.grid_shape()?;
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;
    let mut values = decoder.dispatch()?;

    let target = station.latlon();

    let temp_kelvin = match compute_opts {
        ComputeOptions::Compute => {
            let (idx, (lat, lon), (i, j), value) = latlon
                .zip(ijs)
                .zip(values)
                .enumerate()
                .map(|(idx, ((latlon, ij), value))| (idx, latlon, ij, value))
                .min_by(|(_, ll1, _, _), (_, ll2, _, _)| {
                    let d1 = target.euclidean_sq(ll1);
                    let d2 = target.euclidean_sq(ll2);
                    d1.partial_cmp(&d2).unwrap()
                })
                .ok_or_else(|| anyhow!("No data points found in submessage"))?;

            println!("Idx: {}", idx);
            println!("i,j: {} {}", i, j);
            println!("Lat, lon: {} {}", lat, lon);
            println!("Grid size: {:?}", grid_shape);
            value
        }
        ComputeOptions::Precomputed => {
            let ((i, j), expected_grid_shape) = model.computed_grid_location_and_info(station);
            if expected_grid_shape != grid_shape {
                return Err(anyhow!(
                    "Model's grid shape seems to have changed. Expected {:?} but got {:?}",
                    expected_grid_shape,
                    grid_shape
                ));
            }

            let idx = grid_shape.0 * j + i;
            values
                .nth(idx)
                .ok_or_else(|| anyhow!("Index out of bounds for model grid"))?
        }
    };

    Ok(Temperature::Kelvin(temp_kelvin))
}

pub async fn parse_report_with_opts(
    station: &Station,
    model: &Model,
    ts: DateTime<Tz>,
    lead_time: i64,
    compute_opts: ComputeOptions,
) -> Result<SingleWeatherForecast> {
    let client = Client::new();
    let url = get_url(ts, lead_time);
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        panic!("Failed to fetch: {}", response.status());
    }

    let bytes = response.bytes().await?;
    let cursor = Cursor::new(&bytes);

    let grib2 = grib::from_reader(cursor)?;
    let submessage = find_message(&grib2)?;
    let temperature = temp_closest_to_station(station, model, submessage, compute_opts)?;
    Ok(SingleWeatherForecast {
        temperature,
        timestamp: ts + TimeDelta::hours(lead_time),
    })
}

pub async fn parse_report(
    station: &Station,
    model: &Model,
    ts: DateTime<Tz>,
    lead_time: i64,
) -> Result<SingleWeatherForecast> {
    parse_report_with_opts(station, model, ts, lead_time, ComputeOptions::Precomputed).await
}
