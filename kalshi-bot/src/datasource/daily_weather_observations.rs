use anyhow::{Context, Result};
use async_trait::async_trait;
use weather::{
    observations::daily::{NWSDailyObservationFetcher, NWSDailyReport},
    station::Station,
};

use crate::datasource::datasource::DataSource;
use protocol::protocol::ServiceName;

pub struct DailyWeatherObservationDataSource {
    fetcher: NWSDailyObservationFetcher,
}

impl DailyWeatherObservationDataSource {
    pub fn new(station: Station) -> Self {
        let fetcher = NWSDailyObservationFetcher::new(station, None);
        Self { fetcher }
    }
}

#[async_trait]
impl DataSource<NWSDailyReport> for DailyWeatherObservationDataSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::DailyWeatherObservations
    }

    async fn fetch_data(&mut self) -> Result<NWSDailyReport> {
        self.fetcher
            .fetch(1, true)
            .await
            .context("scraping NWS daily temperatures")
    }
}
