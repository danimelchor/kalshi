use crate::{
    forecast::{
        http::{ForecastHttpOptions, get_report, wait_for_report},
        model::{ComputeOptions, Model},
        parser::parse_report_with_opts,
    },
    station::Station,
};
use anyhow::Result;
use async_stream::stream;
use chrono::{DateTime, DurationRound, TimeDelta, Utc};
use chrono_tz::Tz;
use futures::{FutureExt, Stream, StreamExt, stream::FuturesUnordered};
use protocol::datetime::DateTimeZoned;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Semaphore;

pub use crate::forecast::parser::SingleWeatherForecast;

pub struct ForecastCycle {
    ts: DateTime<Tz>,
    max_lead_time: usize,
    station: Station,
    model: Model,
    compute_options: ComputeOptions,
    historical: bool,
}

impl ForecastCycle {
    pub fn new(
        station: Station,
        model: Model,
        compute_options: ComputeOptions,
        ts: DateTime<Tz>,
        max_lead_time: usize,
        historical: bool,
    ) -> Self {
        Self {
            ts,
            compute_options,
            model,
            station,
            max_lead_time,
            historical,
        }
    }

    async fn parse_report(&self, lead_time: usize) -> Result<SingleWeatherForecast> {
        let opts = ForecastHttpOptions::new(self.model, self.ts, lead_time, self.historical);
        let bytes = get_report(&opts).await?;
        let station = self.station;
        let model = self.model;
        let ts = self.ts;
        let compute_options = self.compute_options;
        tokio::task::spawn_blocking(move || {
            parse_report_with_opts(bytes, station, model, ts, lead_time, compute_options)
        })
        .await?
    }

    async fn wait_and_parse_report(
        &self,
        lead_time: usize,
        sem: Arc<Semaphore>,
    ) -> Result<SingleWeatherForecast> {
        let permit = sem.acquire().await.expect("Unwrapping semaphore");
        let opts = ForecastHttpOptions::new(self.model, self.ts, lead_time, self.historical);
        wait_for_report(&opts).await?;

        // Parsing the report can be done while we download the next one
        drop(permit);

        self.parse_report(lead_time).await
    }

    pub fn fetch(&self) -> impl Stream<Item = Result<SingleWeatherForecast>> {
        let semaphore = Arc::new(Semaphore::new(12));

        let tasks = FuturesUnordered::new();
        for lead_time in 0..=self.max_lead_time {
            let sem = semaphore.clone();

            // With historical runs we don't wait if it doesn't exist
            if self.historical {
                tasks.push(self.parse_report(lead_time).boxed());
            } else {
                tasks.push(self.wait_and_parse_report(lead_time, sem).boxed());
            }
        }
        tasks
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherForecast {
    #[serde(with = "serde_with::rust::maps_duplicate_key_is_error")]
    pub forecast: BTreeMap<DateTimeZoned, SingleWeatherForecast>,
    pub complete: bool,
    pub num_lead_times: usize,
    pub total_lead_times: usize,
}

impl WeatherForecast {
    fn new(forecast: BTreeMap<DateTimeZoned, SingleWeatherForecast>, max_lead_time: usize) -> Self {
        let num_lead_times = forecast.len();
        let total_lead_times = max_lead_time + 1;
        let complete = num_lead_times == total_lead_times;
        Self {
            num_lead_times,
            total_lead_times,
            forecast,
            complete,
        }
    }
}

pub struct ForecastFetcher {
    state: BTreeMap<DateTimeZoned, SingleWeatherForecast>,
    station: Station,
    model: Model,
    max_lead_time: usize,
    compute_options: ComputeOptions,
}

impl ForecastFetcher {
    pub fn new(
        station: Station,
        model: Model,
        max_lead_time: usize,
        compute_options: Option<ComputeOptions>,
    ) -> Self {
        let compute_options = compute_options.unwrap_or(ComputeOptions::Precomputed);
        if max_lead_time > model.max_runs() {
            panic!(
                "The {} model supports at most {} runs",
                model,
                model.max_runs()
            )
        }
        Self {
            compute_options,
            state: BTreeMap::new(),
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

                let forecast_cycle = ForecastCycle::new(
                    self.station,
                    self.model,
                    self.compute_options,
                    ts,
                    self.max_lead_time,
                    false,
                );
                let mut results = forecast_cycle.fetch();
                while let Some(update) = results.next().await {
                    match update {
                        Ok(update) => {
                            let _ = self.state.insert(update.at, update);
                            let forecast = WeatherForecast::new(
                                self.state.clone(),
                                self.max_lead_time,
                            );
                            yield Ok(forecast)
                        },
                        Err(err) => yield Err(err)
                    }
                }

                // Advance to the next report
                ts += TimeDelta::hours(1);
                self.state = BTreeMap::new();
            }
        }
    }
}
