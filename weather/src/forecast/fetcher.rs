use anyhow::Result;
use async_stream::stream;
use bincode::{Decode, Encode};
use chrono::{DateTime, DurationRound, TimeDelta, Utc};
use chrono_tz::Tz;
use futures::{Stream, StreamExt, stream::FuturesUnordered};
use protocol::datetime::SerializableDateTime;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Semaphore;

use crate::{
    forecast::{
        http::{get_report, wait_for_report},
        model::{ComputeOptions, Model},
        parser::{SingleWeatherForecast, parse_report_with_opts},
    },
    station::Station,
    temperature::Temperature,
};

#[derive(Encode, Decode, Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureAtTime {
    pub timestamp: SerializableDateTime,
    pub temperature: Temperature,
}

impl TemperatureAtTime {
    fn new(timestamp: DateTime<Tz>, temperature: Temperature) -> Self {
        Self {
            timestamp: timestamp.into(),
            temperature,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub temperatures_at_times: Vec<TemperatureAtTime>,
}

struct ForecastCycle {
    ts: DateTime<Tz>,
    max_lead_time: usize,
    station: Station,
    model: Model,
    compute_options: ComputeOptions,
}

impl ForecastCycle {
    pub fn new(
        station: Station,
        model: Model,
        compute_options: ComputeOptions,
        ts: DateTime<Tz>,
        max_lead_time: usize,
    ) -> Self {
        Self {
            ts,
            compute_options,
            model,
            station,
            max_lead_time,
        }
    }

    async fn wait_and_parse_report(
        &self,
        lead_time: usize,
        sem: Arc<Semaphore>,
    ) -> Result<SingleWeatherForecast> {
        let permit = sem.acquire().await.expect("Unwrapping semaphore");
        wait_for_report(&self.model, &self.ts, lead_time).await;
        let bytes = get_report(&self.model, &self.ts, lead_time).await?;

        // Parsing the report can be done while we download the next one
        drop(permit);

        parse_report_with_opts(
            bytes,
            &self.station,
            &self.model,
            &self.ts,
            lead_time,
            self.compute_options,
        )
        .await
    }

    pub fn fetch(&self) -> impl Stream<Item = Result<SingleWeatherForecast>> {
        let semaphore = Arc::new(Semaphore::new(3)); // max 3 concurrent

        let tasks = FuturesUnordered::new();
        for lead_time in 0..self.max_lead_time {
            let sem = semaphore.clone();
            tasks.push(self.wait_and_parse_report(lead_time, sem));
        }
        tasks
    }
}

pub struct ForecastFetcher {
    state: HashMap<DateTime<Tz>, Temperature>,
    station: Station,
    model: Model,
    max_lead_time: usize,
    compute_options: ComputeOptions,
}

impl From<HashMap<DateTime<Tz>, Temperature>> for WeatherForecast {
    fn from(state: HashMap<DateTime<Tz>, Temperature>) -> Self {
        let temperatures_at_times: Vec<_> = state
            .into_iter()
            .map(|(time, temp)| TemperatureAtTime::new(time, temp))
            .collect();
        Self {
            temperatures_at_times,
        }
    }
}

impl ForecastFetcher {
    pub fn new(
        station: Station,
        model: Model,
        max_lead_time: usize,
        compute_options: Option<ComputeOptions>,
    ) -> Self {
        let compute_options = compute_options.unwrap_or(ComputeOptions::Precomputed);
        Self {
            compute_options,
            state: HashMap::new(),
            max_lead_time,
            station,
            model,
        }
    }

    pub fn fetch(&mut self) -> impl Stream<Item = Result<WeatherForecast>> {
        let mut ts = (Utc::now() - TimeDelta::hours(1))
            .with_timezone(&self.station.timezone())
            .duration_trunc(TimeDelta::hours(1))
            .unwrap();

        stream! {
            loop {
                println!("Waiting for {ts}'s report");

                let forecast_cycle =
                    ForecastCycle::new(self.station, self.model, self.compute_options, ts, self.max_lead_time);
                let mut results =forecast_cycle.fetch() ;
                while let Some(update) = results.next().await {
                    match update {
                        Ok(update) => {
                            let _ = self.state.insert(update.timestamp, update.temperature);
                            yield Ok( self.state.clone().into())
                        },
                        Err(err) => yield Err(err)
                    }
                }

                // Advance to the next report
                ts += TimeDelta::hours(1);
            }
        }
    }
}
