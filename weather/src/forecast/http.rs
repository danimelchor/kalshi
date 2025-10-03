use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::{Tz, UTC};
use reqwest::{Client, StatusCode, header::RANGE};
use std::time::Duration;
use tokio::time::sleep;

use crate::forecast::model::Model;

static BASE: &str = "https://nomads.ncep.noaa.gov/pub/data/nccf/com/hrrr/prod";

static HISTORICAL: &str = "https://pando-rgw01.chpc.utah.edu";

static FORECAST_TYPE: &str = "wrfsfcf";

pub struct ForecastHttpOptions {
    model: Model,
    ts: DateTime<Tz>,
    lead_time: usize,
    historical: bool,
}

impl ForecastHttpOptions {
    pub fn new(model: Model, ts: DateTime<Tz>, lead_time: usize, historical: bool) -> Self {
        Self {
            model,
            ts,
            lead_time,
            historical,
        }
    }
}

fn get_url(opts: &ForecastHttpOptions) -> String {
    let utc = opts.ts.with_timezone(&UTC);
    let hh = format!("{:02}", utc.hour());
    let date = format!("{:04}{:02}{:02}", utc.year(), utc.month(), utc.day());
    let model = opts.model.to_string().to_lowercase();
    let lead_time = opts.lead_time;
    if opts.historical {
        format!("{HISTORICAL}/hrrr/sfc/{date}/{model}.t{hh}z.{FORECAST_TYPE}{lead_time:0>2}.grib2")
    } else {
        format!("{BASE}/hrrr.{date}/conus/{model}.t{hh}z.{FORECAST_TYPE}{lead_time:0>2}.grib2")
    }
}

pub enum ReportState {
    Exists,
    DoesntExist,
    RateLimit,
    Error(StatusCode),
}

async fn check_if_report_exists(opts: &ForecastHttpOptions) -> Result<ReportState> {
    let url = get_url(opts);
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

    if context.len() < 2 {
        anyhow::bail!("Invalid index file format: {}", text)
    }
    let byte_start = parse_byte_offset_from_line(context[0])?;
    let byte_end = parse_byte_offset_from_line(context[1])?;
    Ok((byte_start, byte_end))
}

pub async fn get_report(opts: &ForecastHttpOptions) -> Result<Bytes> {
    let url = get_url(opts);
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

pub async fn wait_for_report(opts: &ForecastHttpOptions) -> Result<()> {
    let mut retries = 0;
    loop {
        match check_if_report_exists(opts).await? {
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
