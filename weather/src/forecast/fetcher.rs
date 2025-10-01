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
        http::wait_for_report,
        model::{ComputeOptions, Model},
        parser::{SingleWeatherForecast, parse_report_with_opts},
    },
    station::Station,
    temperature::Temperature,
};

#[derive(Encode, Decode, Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub temperatures: Vec<Temperature>,
    pub timestamps: Vec<SerializableDateTime>,
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
        let _permit = sem.acquire().await.expect("Unwrapping semaphore");
        wait_for_report(&self.model, &self.ts, lead_time).await;
        parse_report_with_opts(
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
    compute_options: ComputeOptions,
}

impl From<HashMap<DateTime<Tz>, Temperature>> for WeatherForecast {
    fn from(state: HashMap<DateTime<Tz>, Temperature>) -> Self {
        let (timestamps, temperatures): (Vec<_>, Vec<_>) =
            state.into_iter().map(|(k, v)| (k.into(), v)).unzip();
        Self {
            temperatures,
            timestamps,
        }
    }
}

impl ForecastFetcher {
    pub fn new(station: Station, model: Model, compute_options: Option<ComputeOptions>) -> Self {
        let compute_options = compute_options.unwrap_or(ComputeOptions::Precomputed);
        Self {
            compute_options,
            state: HashMap::new(),
            station,
            model,
        }
    }

    pub fn fetch(&mut self) -> impl Stream<Item = WeatherForecast> {
        let mut ts = Utc::now()
            .with_timezone(&self.station.timezone())
            .duration_round(TimeDelta::hours(1))
            .unwrap()
            - TimeDelta::hours(1);

        stream! {
            loop {
                eprintln!("Waiting {ts}'s report");

                let forecast_cycle =
                    ForecastCycle::new(self.station.clone(), self.model.clone(),self.compute_options, ts, 12);
                let mut results =forecast_cycle.fetch() ;
                while let Some(update) = results.next().await {
                    match update {
                        Ok(update) => {
                            let _ = self.state.insert(update.timestamp, update.temperature);
                            yield self.state.clone().into()
                        },
                        Err(err) => eprintln!("Error fetching forecast: {}", err)
                    }
                }

                // Advance to the next report
                ts += TimeDelta::hours(1);
            }
        }
    }
}
