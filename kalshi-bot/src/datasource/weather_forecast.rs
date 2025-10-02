use async_trait::async_trait;
use futures::Stream;
use weather::{
    forecast::{
        fetcher::{ForecastFetcher, WeatherForecast},
        model::Model,
    },
    station::Station,
};

use crate::datasource::datasource::DataSource;
use protocol::protocol::ServiceName;

use anyhow::Result;

pub struct WeatherForecastDataSource {
    fetcher: ForecastFetcher,
}

impl WeatherForecastDataSource {
    pub fn new(station: Station, model: Model) -> Self {
        let fetcher = ForecastFetcher::new(station, model, 18, None);
        Self { fetcher }
    }
}

#[async_trait]
impl DataSource<WeatherForecast> for WeatherForecastDataSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::WeatherForecast
    }

    fn fetch_data(&mut self) -> impl Stream<Item = Result<WeatherForecast>> + Send {
        self.fetcher.fetch()
    }
}
