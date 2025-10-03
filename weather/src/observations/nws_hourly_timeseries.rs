use anyhow::{Result, anyhow};
use chrono::Datelike;
use fantoccini::error::CmdError;
use fantoccini::{Client, ClientBuilder, Locator};
use protocol::datetime::DateTimeZoned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::station::Station;
use crate::temperature::Temperature;

static PROD_BASE_URL: &str = "https://www.weather.gov/wrh/timeseries";

#[derive(Debug, Serialize, Deserialize)]
pub struct NWSHourlyTimeseriesTemperature {
    pub datetime: DateTimeZoned,
    pub station: Station,
    pub temperature: Temperature,
    pub six_hr_max_temperature: Option<Temperature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NWSHourlyTimeseriesTemperatures(pub Vec<NWSHourlyTimeseriesTemperature>);

fn clean_col(name: &str) -> String {
    let re_non_alnum = regex::Regex::new(r"[^a-zA-Z0-9 ]").unwrap();
    let re_spaces = regex::Regex::new(r"\s+").unwrap();
    let name = re_non_alnum.replace_all(name, " ");
    let name = re_spaces.replace_all(&name, " ").trim().to_lowercase();
    name.replace(' ', "_")
}

fn to_float(val: &String) -> Result<f64, &'static str> {
    if val.trim().is_empty() {
        return Err("Invalid float");
    }
    if val == "T" {
        return Ok(0.0);
    }
    let cleaned = val.replace('<', "");
    cleaned.parse::<f64>().map_err(|_| "Invalid float")
}

fn maybe_to_float(val: Option<&String>) -> Option<f64> {
    val.and_then(|v| to_float(v).ok())
}

impl NWSHourlyTimeseriesTemperature {
    pub fn from_row(station: Station, row: &HashMap<String, String>) -> Result<Self, String> {
        // Parse date
        let year = chrono::Local::now().year();
        let dt_str = format!("{}, {}", row.get("date_time_l").unwrap(), year);
        let dt = chrono::NaiveDateTime::parse_from_str(&dt_str, "%b %d, %I:%M %p, %Y")
            .map_err(|e| format!("Failed to parse datetime: {}", e))?;
        let tz = station.timezone();
        let dt = dt
            .and_local_timezone(tz)
            .single()
            .ok_or("Failed to convert timezone")?;

        let temp_f = to_float(row.get("temp_f").unwrap()).map_err(|e| e.to_string())?;
        let six_hr_max_f = maybe_to_float(row.get("6_hr_max_f")).map(Temperature::Fahrenheit);

        Ok(NWSHourlyTimeseriesTemperature {
            datetime: dt.into(),
            station,
            temperature: Temperature::Fahrenheit(temp_f),
            six_hr_max_temperature: six_hr_max_f,
        })
    }
}

pub struct NWSHourlyTimeseriesScraper {
    station: Station,
    base_url: String,
    client: Client,
    headers_cache: Vec<String>,
}

async fn connect_with_retries() -> Result<Client> {
    let delay = Duration::from_secs(1);
    let max_retries = 4;

    for attempt in 1..=max_retries {
        match ClientBuilder::native()
            .capabilities(serde_json::from_str(
                r#"{"moz:firefoxOptions": {"args": ["--headless"]}}"#,
            )?)
            .connect("http://localhost:4444")
            .await
        {
            Ok(client) => return Ok(client),
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {attempt} failed: {e}. Retrying in {:?}...", delay);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(anyhow!(e)),
        }
    }

    unreachable!()
}

impl NWSHourlyTimeseriesScraper {
    pub async fn new(station: Station, base_url: Option<&str>) -> Result<Self> {
        let base_url = base_url.unwrap_or(PROD_BASE_URL);
        let client = connect_with_retries().await?;
        println!("Connected to geckoclient");

        Ok(Self {
            station,
            client,
            base_url: base_url.to_string(),
            headers_cache: Vec::new(),
        })
    }

    fn url(&self) -> String {
        let params = vec![
            ("site", self.station.to_string()),
            ("hours", "72".to_string()),
            ("units", "english".to_string()),
            ("headers", "none".to_string()),
            ("chart", "off".to_string()),
            ("obs", "tabular".to_string()),
        ];
        let query: String = serde_urlencoded::to_string(params).unwrap();
        format!("{}?{}", self.base_url, query)
    }

    pub async fn scrape(&mut self) -> Result<NWSHourlyTimeseriesTemperatures> {
        self.client.goto(&self.url()).await?;

        let table = self
            .client
            .wait()
            .for_element(Locator::Css("table#OBS_DATA"))
            .await?;

        if self.headers_cache.is_empty() {
            let ths = table.find_all(Locator::Css("th")).await?;
            for th in ths {
                let text = th.text().await?;
                self.headers_cache.push(clean_col(&text));
            }
        }

        // Parse all rows
        let mut rows = Vec::new();
        let trs = table.find_all(Locator::Css("tr")).await?;
        for tr in trs {
            let tds = tr.find_all(Locator::Css("td")).await?;
            let texts =
                futures::future::join_all(tds.into_iter().map(|c| async move { c.text().await }))
                    .await;
            let texts: Vec<String> = texts.into_iter().collect::<Result<_, _>>()?;
            if !texts.is_empty() {
                rows.push(texts);
            }
        }

        let mut result = Vec::new();
        for row in rows.into_iter().rev() {
            let map: HashMap<_, _> = self.headers_cache.iter().cloned().zip(row).collect();
            if let Ok(temp) = NWSHourlyTimeseriesTemperature::from_row(self.station, &map) {
                result.push(temp);
            }
        }

        Ok(NWSHourlyTimeseriesTemperatures(result))
    }

    pub async fn close(self) -> Result<(), CmdError> {
        self.client.close().await
    }
}
