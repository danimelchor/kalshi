use anyhow::{Context, Result};
use chrono::{Datelike, Months, NaiveDate, NaiveTime, Utc};
use protocol::datetime::DateTimeZoned;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::station::Station;
use crate::temperature::Temperature;

static PROD_BASE_URL: &str = "https://forecast.weather.gov/data/obhistory";

#[derive(Debug, Serialize, Deserialize)]
pub struct NWSHourlyTableTemperature {
    pub datetime: DateTimeZoned,
    pub station: Station,
    pub temperature: Temperature,
    pub six_hr_max_temperature: Option<Temperature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NWSHourlyTableTemperatures(pub Vec<NWSHourlyTableTemperature>);

// Unfortunately the report's table has fancy headers which means that len(headers) != len(row)
// so I'll hardcode the indices of each field I care about for now
const DAY_IDX: usize = 0;
const TIME_IDX: usize = 1;
const TEMP_IDX: usize = 6;
const MAX_SIX_H_IDX: usize = 8;

impl NWSHourlyTableTemperature {
    pub fn parse_row(station: Station, cells: &[String], date: &NaiveDate) -> Result<Self> {
        // Parse day + time + current date into a single datetime
        // The "day" format here is shit so if e.g.: we see a 30 and today is 1 that
        // probably means it's part of the previous month
        let day = cells.get(DAY_IDX).context("Expected a day column")?;
        let day: u32 = day.parse().context("Expected day of month to be an int")?;
        let time = cells.get(TIME_IDX).context("Expected a time column")?;
        let time =
            NaiveTime::parse_from_str(time, "%H:%M").context("Expected time to follow %I:%M")?;
        let date = if day > date.day() {
            &date
                .checked_sub_months(Months::new(1))
                .expect("The date won't overflow a month ago")
        } else {
            date
        };
        let datetime = date
            .with_day(day)
            .context("Expected day of month to be valid")?
            .and_time(time)
            .and_local_timezone(station.timezone())
            .single()
            .context("Expected to be able to convert naive datetime to station's timezone")?;

        // Temperature measurement
        let temperature = cells.get(TEMP_IDX).context("Expected a temp column")?;
        let temperature: f32 = temperature
            .parse()
            .context("Expected temperature to be a float")?;
        let temperature = Temperature::Fahrenheit(temperature);

        // Max 6h temperature (sometimes reported)
        let six_hr_max_temperature = cells
            .get(MAX_SIX_H_IDX)
            .context("Expected a max 6h temp column")?;
        let six_hr_max_temperature: Option<Temperature> = six_hr_max_temperature
            .parse::<f32>()
            .ok()
            .map(Temperature::Fahrenheit);

        Ok(NWSHourlyTableTemperature {
            datetime: datetime.into(),
            station,
            temperature,
            six_hr_max_temperature,
        })
    }
}

pub struct NWSHourlyTableFetcher {
    station: Station,
    url: String,
    client: Client,
}

impl NWSHourlyTableFetcher {
    pub fn new(station: Station, base_url: Option<&str>) -> Self {
        let base_url = base_url.unwrap_or(PROD_BASE_URL);
        let url = base_url.to_string() + &format!("/{}.html", &station.to_string());

        let client = Client::new();
        Self {
            station,
            client,
            url,
        }
    }

    pub async fn fetch(&mut self) -> Result<NWSHourlyTableTemperatures> {
        let res = self
            .client
            .get(&self.url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (compatible; MyRustClient/0.1; +https://example.com)",
            )
            .send()
            .await?;
        res.error_for_status_ref()?;

        let text = res.text().await?;
        let document = Html::parse_document(&text);
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td").unwrap();

        let mut rows = Vec::new();
        let date = Utc::now()
            .with_timezone(&self.station.timezone())
            .date_naive();

        for row in document.select(&row_selector).rev() {
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|cell| cell.text().collect::<String>())
                .collect();

            if let Ok(temp) = NWSHourlyTableTemperature::parse_row(self.station, &cells, &date) {
                rows.push(temp);
            }
        }
        Ok(NWSHourlyTableTemperatures(rows))
    }
}
