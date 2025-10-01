use std::time::Duration;

use crate::datasource::datasource::DataSource;
use anyhow::{Context, Result};
use async_stream::stream;
use futures::Stream;
use protocol::protocol::ServiceName;
use tokio::time::sleep;
use weather::{
    observations::hourly::{NWSHourlyObservationsScraper, NWSHourlyTemperatures},
    station::Station,
};

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

impl DataSource<NWSHourlyTemperatures> for HourlyWeatherObservationDataSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::HourlyWeatherObservations
    }

    fn fetch_data(&mut self) -> impl Stream<Item = Result<NWSHourlyTemperatures>> + Send {
        stream! {
            loop {

        let result = self.scraper
            .scrape()
            .await;
                yield result;
                    sleep(Duration::from_secs(60)).await;
            }
        }
    }
}
