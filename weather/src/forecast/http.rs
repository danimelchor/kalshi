use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::{Tz, UTC};
use reqwest::{Client, StatusCode, header::RANGE};
use std::time::Duration;
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

pub enum ReportState {
    Exists,
    DoesntExist,
    RateLimit,
    Error(StatusCode),
}

async fn check_if_report_exists(
    model: &Model,
    ts: &DateTime<Tz>,
    lead_time: usize,
) -> Result<ReportState> {
    let url = get_url(model, ts, lead_time);
    let client = Client::new();
    let response = client.head(url).send().await?;
    let status_code = response.status();
    let state = match status_code {
        StatusCode::OK => ReportState::Exists,
        StatusCode::NOT_FOUND => ReportState::DoesntExist,
        StatusCode::FOUND => ReportState::RateLimit,
        _ => ReportState::Error(status_code),
    };
    Ok(state)
}

pub fn parse_byte_offset_from_line(line: &str) -> Result<usize> {
    let byte_offset: &str = line
        .split(":")
        .nth(1)
        .with_context(|| format!("Malformed idx line: {}", line))?;

    let byte_offset: usize = byte_offset
        .parse()
        .context("Parsing byte offset as a usize")?;

    Ok(byte_offset)
}

pub async fn get_index(url: &str) -> Result<(usize, usize)> {
    let url = format!("{}.idx", url);
    let client = Client::new();
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch: {}", response.status())
    }
    let text = response
        .text()
        .await
        .context("Reading text from response")?;
    let context: Vec<&str> = text
        .lines()
        .skip_while(|line| !line.contains("TMP:2 m"))
        .take(2)
        .collect();

    let byte_start = parse_byte_offset_from_line(context[0])?;
    let byte_end = parse_byte_offset_from_line(context[1])?;
    Ok((byte_start, byte_end))
}

pub async fn get_report(model: &Model, ts: &DateTime<Tz>, lead_time: usize) -> Result<Bytes> {
    let url = get_url(model, ts, lead_time);
    let (byte_start, byte_end) = get_index(&url).await?;

    let client = Client::new();
    let response = client
        .get(url)
        .header(RANGE, format!("bytes={byte_start}-{byte_end})"))
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch: {}", response.status())
    }
    response
        .bytes()
        .await
        .context("Extracting bytes from response")
}

pub async fn wait_for_report(model: &Model, ts: &DateTime<Tz>, lead_time: usize) -> Result<()> {
    let mut retries = 0;
    loop {
        match check_if_report_exists(model, ts, lead_time).await? {
            ReportState::Exists => return Ok(()),
            ReportState::Error(status) => {
                return Err(anyhow!("Failed request with status {}", status));
            }
            ReportState::RateLimit => {
                eprintln!("Forecast fetcher rate limited");
                sleep(Duration::from_secs(60 * 2_u64.pow(retries))).await;
                retries += 1;
            }
            ReportState::DoesntExist => {
                sleep(Duration::from_secs(60)).await;
                retries = 0;
            }
        };
    }
}
