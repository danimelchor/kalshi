use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use weather::{
    forecast::{
        fetcher::{WeatherForecast, fetch},
        model::Model,
    },
    station::Station,
};

use crate::datasource::datasource::DataSource;
use protocol::protocol::ServiceName;

use anyhow::Result;

pub struct WeatherForecastDataSource {
    station: Station,
    model: Model,
}

impl WeatherForecastDataSource {
    pub fn new(station: Station, model: Model) -> Self {
        Self { station, model }
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

    async fn fetch_data(&mut self) -> Result<WeatherForecast> {
        let ts = Utc::now().with_timezone(&self.station.timezone()) - TimeDelta::hours(1);
        fetch(&self.station, &self.model, ts).await
    }
}
