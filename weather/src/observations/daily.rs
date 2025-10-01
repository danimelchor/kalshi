use anyhow::{Context, Result, anyhow};
use bincode::{Decode, Encode};
use chrono::{TimeDelta, Utc};
use protocol::datetime::SerializableDateTime;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::station::Station;
use crate::temperature::Temperature;

static PROD_BASE_URL: &str = "https://forecast.weather.gov/product.php";

#[derive(Debug, Encode, Decode)]
pub struct NWSDailyReport {
    pub datetime: SerializableDateTime,
    pub station: Station,
    pub max_temperature: Temperature,
}

impl NWSDailyReport {
    pub fn parse_report(report: &str, station: Station, for_today: bool) -> Result<Self> {
        let mut lines = report.lines();

        // Find TEMPERATURE section
        for line in &mut lines {
            if line.trim_start().starts_with("TEMPERATURE (F)") {
                break;
            }
        }

        let for_when = lines.next().context("Malformed daily NWS report")?;
        let dt = if for_when.trim().to_lowercase() == "today" {
            Utc::now().with_timezone(&station.timezone())
        } else if for_when.trim().to_lowercase() == "today" {
            if for_today {
                return Err(anyhow!("Report is not for today"));
            }
            Utc::now().with_timezone(&station.timezone()) - TimeDelta::days(1)
        } else {
            return Err(anyhow!("Unexpected report date: {}", for_when));
        };

        let maximum_line = lines.next().context("Malformed daily report")?;
        let re = Regex::new(r"MAXIMUM\s+([0-9\.]+).*").unwrap();
        let caps = re
            .captures(maximum_line.trim())
            .context("Malformed daily report")?;
        let max_temp_f = caps[1].parse::<f32>().context("Malformed daily report")?;

        Ok(Self {
            datetime: dt.into(),
            station,
            max_temperature: Temperature::Fahrenheit(max_temp_f),
        })
    }
}

pub struct NWSDailyObservationFetcher {
    station: Station,
    base_url: String,
    client: Client,
}

impl NWSDailyObservationFetcher {
    pub fn new(station: Station, base_url: Option<&str>) -> Self {
        let base_url = base_url.unwrap_or(PROD_BASE_URL);
        let client = Client::new();
        Self {
            station,
            client,
            base_url: base_url.to_string(),
        }
    }

    pub async fn fetch(&mut self, version: u32, for_today: bool) -> Result<NWSDailyReport> {
        let params = [
            ("site", self.station.area_code()),
            ("issuedby", self.station.city()),
            ("product", "CLI"),
            ("format", "TXT"),
            ("version", &version.to_string()),
            ("highlight", "off"),
            ("glossary", "0"),
        ];
        let res = self
            .client
            .get(&self.base_url)
            .query(&params)
            .header(
                "User-Agent",
                "Mozilla/5.0 (compatible; MyRustClient/0.1; +https://example.com)",
            )
            .send()
            .await?;
        res.error_for_status_ref()?;

        let text = res.text().await?;
        let document = Html::parse_document(&text);
        let selector = Selector::parse("pre").unwrap();
        let pre_block = document
            .select(&selector)
            .next()
            .context("Report not found")?;

        let report_text = pre_block.text().collect::<Vec<_>>().join("\n");

        NWSDailyReport::parse_report(&report_text, self.station, for_today)
    }
}
