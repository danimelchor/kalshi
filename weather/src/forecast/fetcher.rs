use std::time::Instant;

use anyhow::Result;
use bincode::{Decode, Encode};
use chrono::DateTime;
use chrono_tz::Tz;
use futures::future::join_all;
use protocol::datetime::SerializableDateTime;

use crate::{
    forecast::{
        model::Model,
        parser::{SingleWeatherForecast, parse_report},
    },
    station::Station,
    temperature::Temperature,
};

#[derive(Encode, Decode, Debug, Clone)]
pub struct WeatherForecast {
    pub temperatures: Vec<Temperature>,
    pub timestamps: Vec<SerializableDateTime>,
}

pub async fn fetch(station: &Station, model: &Model, ts: DateTime<Tz>) -> Result<WeatherForecast> {
    let start = Instant::now();

    let mut tasks = Vec::new();
    for lead_time in 1..=12 {
        let task = parse_report(station, model, ts, lead_time);
        tasks.push(task);
    }
    let result = join_all(tasks).await;
    let result = result
        .into_iter()
        .collect::<Result<Vec<SingleWeatherForecast>>>()?;

    let mut temperatures = vec![];
    let mut timestamps = vec![];
    for forecast in result {
        temperatures.push(forecast.temperature);
        timestamps.push(forecast.timestamp.into());
    }

    let duration = start.elapsed(); // Measure elapsed time
    println!("Fetching full forecast took: {:?}", duration);

    Ok(WeatherForecast {
        timestamps,
        temperatures,
    })
}
