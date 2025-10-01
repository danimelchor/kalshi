use anyhow::{Result, anyhow};
use bytes::Bytes;
use chrono::{DateTime, TimeDelta};
use chrono_tz::Tz;
use grib::{Grib2, SeekableGrib2Reader, SubMessage};
use std::io::Cursor;

use crate::{
    forecast::model::{ComputeOptions, Model},
    station::Station,
    temperature::Temperature,
};

const TEMPERATURE_NUMBER: u8 = 0;
const SURFACE_TYPE: u8 = 103;
const METERS_ABOVE_GROUND: i32 = 2;

#[derive(Debug)]
pub struct SingleWeatherForecast {
    pub temperature: Temperature,
    pub timestamp: DateTime<Tz>,
    pub _lead_time: usize,
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

    Ok(Temperature::Kelvin(temp_kelvin).to_fahrenheit())
}

pub async fn parse_report_with_opts(
    bytes: Bytes,
    station: &Station,
    model: &Model,
    ts: &DateTime<Tz>,
    lead_time: usize,
    compute_opts: ComputeOptions,
) -> Result<SingleWeatherForecast> {
    let cursor = Cursor::new(&bytes);
    let grib2 = grib::from_reader(cursor)?;
    let submessage = find_message(&grib2)?;
    let temperature = temp_closest_to_station(station, model, submessage, compute_opts)?;
    Ok(SingleWeatherForecast {
        temperature,
        timestamp: *ts + TimeDelta::hours(lead_time as i64),
        _lead_time: lead_time,
    })
}
