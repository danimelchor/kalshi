use anyhow::{Context, Result};
use chrono::{DateTime, Duration, DurationRound, NaiveDateTime, TimeDelta};
use chrono_tz::Tz;
use futures::{StreamExt, stream::FuturesUnordered};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::Semaphore};

use crate::{
    forecast::{
        fetcher::ForecastCycle,
        model::{ComputeOptions, Model},
        parser::SingleWeatherForecast,
    },
    station::Station,
};

async fn fetch_one(
    station: Station,
    model: Model,
    ts: DateTime<Tz>,
    sm: Arc<Semaphore>,
) -> Result<Vec<SingleWeatherForecast>> {
    let _permit = sm.acquire().await?;
    let fetcher = ForecastCycle::new(station, model, ComputeOptions::Precomputed, ts, 18, true);
    let results: Vec<Result<SingleWeatherForecast>> = fetcher.fetch().collect().await;
    let results: Result<Vec<SingleWeatherForecast>> = results.into_iter().collect();
    let results = results?;
    Ok(results)
}

pub async fn main(
    station: Station,
    model: Model,
    from: NaiveDateTime,
    to: NaiveDateTime,
    file: PathBuf,
) -> Result<()> {
    let mut start = from
        .and_local_timezone(station.timezone())
        .single()
        .expect("Single timezone for station")
        .duration_round(Duration::hours(1))
        .unwrap()
        - TimeDelta::days(1);
    let end = to
        .and_local_timezone(station.timezone())
        .single()
        .expect("Single timezone for station");

    let mut tasks = FuturesUnordered::new();
    let sm = Arc::new(Semaphore::new(1));
    while start < end {
        tasks.push(fetch_one(station, model, start, sm.clone()));
        start += TimeDelta::hours(1);
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file)
        .await?;
    while let Some(result) = tasks.next().await {
        match result {
            Ok(result) => {
                for forecast in result {
                    let dt: DateTime<Tz> = forecast.timestamp.into();
                    let obj = &json!({
                        "timestamp": dt,
                        "temperature": forecast.temperature.as_fahrenheit(),
                        "lead_time": forecast._lead_time,
                    });
                    println!("{}", obj);
                    let json = serde_json::to_string(obj).context("serialize to json")?;

                    file.write_all(json.as_bytes())
                        .await
                        .context("writing to file")?;
                    file.write_all(b"\n").await.context("writing to file")?;
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
    Ok(())
}
