use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::{Tz, UTC};
use reqwest::Client;
use tokio::time::sleep;

use crate::forecast::model::Model;

static BASE: &str = "https://nomads.ncep.noaa.gov/pub/data/nccf/com/hrrr/prod";
static FORECAST_TYPE: &str = "wrfsfcf";

fn get_url(model: &Model, ts: &DateTime<Tz>, lead_time: usize) -> String {
    let utc = ts.with_timezone(&UTC);
    let hh = format!("{:02}", utc.hour());
    let date = format!("{:04}{:02}{:02}", utc.year(), utc.month(), utc.day());
    let model = model.to_string().to_lowercase();
    format!("{BASE}/hrrr.{date}/conus/{model}.t{hh}z.{FORECAST_TYPE}{lead_time:0>2}.grib2")
}

async fn check_if_report_exists(
    model: &Model,
    ts: &DateTime<Tz>,
    lead_time: usize,
) -> Result<bool> {
    let url = get_url(model, ts, lead_time);
    let client = Client::new();
    let response = client.head(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch: {}", response.status())
    }
    Ok(response.status().is_success())
}

pub async fn get_report(model: &Model, ts: &DateTime<Tz>, lead_time: usize) -> Result<Bytes> {
    let url = get_url(model, ts, lead_time);
    let client = Client::new();
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch: {}", response.status())
    }
    response
        .bytes()
        .await
        .context("Extracting bytes from response")
}

pub async fn wait_for_report(model: &Model, ts: &DateTime<Tz>, lead_time: usize) {
    loop {
        if let Ok(true) = check_if_report_exists(model, ts, lead_time).await {
            return;
        }
        eprintln!(
            "Report for {} with {} lead time is not ready yet",
            ts, lead_time
        );
        sleep(Duration::from_secs(60)).await;
    }
}
