use anyhow::{Context, Result};
use async_trait::async_trait;
use weather::{
    observations::hourly::{NWSHourlyObservationsScraper, NWSHourlyTemperatures},
    station::Station,
};

use crate::datasource::datasource::DataSource;
use protocol::protocol::ServiceName;

pub struct HourlyWeatherObservationDataSource {
    scraper: NWSHourlyObservationsScraper,
}

impl HourlyWeatherObservationDataSource {
    pub async fn new(station: Station) -> Result<Self> {
        let scraper = NWSHourlyObservationsScraper::new(station, None)
            .await
            .context("unable to start scraper")?;
        Ok(Self { scraper })
    }
}

#[async_trait]
impl DataSource<NWSHourlyTemperatures> for HourlyWeatherObservationDataSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::HourlyWeatherObservations
    }

    async fn fetch_data(&mut self) -> Result<NWSHourlyTemperatures> {
        self.scraper
            .scrape()
            .await
            .context("scraping NWS hourly temperatures")
    }
}
