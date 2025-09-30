use anyhow::Result;
use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
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

pub async fn fetch(station: &Station, model: &Model, ts: DateTime<Utc>) -> Result<WeatherForecast> {
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

    Ok(WeatherForecast {
        timestamps,
        temperatures,
    })
}
