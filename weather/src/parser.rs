use anyhow::{Result, anyhow};
use bytes::Bytes;
use chrono::{Datelike, Utc};
use clap::ValueEnum;
use grib::{Grib2, SeekableGrib2Reader, SubMessage};
use reqwest::Client;
use std::io::Cursor;

use crate::{coords::LatLon, model::Model, station::Station, temperature::Temperature};

static BASE: &str = "https://nomads.ncep.noaa.gov/pub/data/nccf/com/hrrr/prod";
static FORECAST_TYPE: &str = "wrfsfcf";

const TEMPERATURE_NUMBER: u8 = 0;
const SURFACE_TYPE: u8 = 103;
const METERS_ABOVE_GROUND: i32 = 2;

#[derive(Debug)]
pub struct WeatherForecast {
    latlon: LatLon,
    temperature: Temperature,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ComputeOptions {
    Compute,
    Precomputed,
}

fn get_url(lead_time: i8) -> String {
    // TODO: FIX THIS
    let now = Utc::now();
    let date = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day() - 1);
    let hh = format!("{:02}", 22);
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
) -> Result<WeatherForecast> {
    let latlon = submessage.latlons()?;
    let ijs = submessage.ij()?;
    let grid_shape = submessage.grid_shape()?;
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;
    let values = decoder.dispatch()?;

    let target = station.latlon();

    let ((lat, lon), temp_kelvin) = match compute_opts {
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

            println!("{} {} at idx {}", i, j, idx);
            println!("Grid size: {:?}", grid_shape);
            ((lat, lon), value)
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
            latlon
                .zip(values)
                .nth(idx)
                .ok_or_else(|| anyhow!("Index out of bounds for model grid"))?
        }
    };

    Ok(WeatherForecast {
        latlon: LatLon::new(lat, lon),
        temperature: Temperature::Kelvin(temp_kelvin),
    })
}

pub async fn parse_report(
    station: &Station,
    model: &Model,
    lead_time: i8,
    compute_opts: ComputeOptions,
) -> Result<WeatherForecast> {
    let client = Client::new();
    let url = get_url(lead_time);
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        panic!("Failed to fetch: {}", response.status());
    }

    let bytes = response.bytes().await?;
    let cursor = Cursor::new(&bytes);

    let grib2 = grib::from_reader(cursor)?;
    let submessage = find_message(&grib2)?;
    temp_closest_to_station(station, model, submessage, compute_opts)
}
