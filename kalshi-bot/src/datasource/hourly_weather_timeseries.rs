use std::{fmt::Display, time::Duration};

use crate::datasource::datasource::DataSource;
use anyhow::{Context, Result};
use async_stream::stream;
use futures::Stream;
use protocol::protocol::ServiceName;
use tokio::time::sleep;
use weather::{
    observations::nws_hourly_timeseries::{
        NWSHourlyTimeseriesScraper, NWSHourlyTimeseriesTemperatures,
    },
    station::Station,
};

pub struct HourlyWeatherTimeseriesSource {
    scraper: NWSHourlyTimeseriesScraper,
}

impl Display for HourlyWeatherTimeseriesSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HourlyWeatherTimeseriesSource")
    }
}

impl HourlyWeatherTimeseriesSource {
    pub async fn new(station: Station) -> Result<Self> {
        let scraper = NWSHourlyTimeseriesScraper::new(station, None)
            .await
            .context("unable to start scraper")?;
        Ok(Self { scraper })
    }
}

impl DataSource<NWSHourlyTimeseriesTemperatures> for HourlyWeatherTimeseriesSource {
    fn service_name() -> ServiceName {
        ServiceName::HourlyWeatherTimeseries
    }

    fn fetch_data(&mut self) -> impl Stream<Item = Result<NWSHourlyTimeseriesTemperatures>> + Send {
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
