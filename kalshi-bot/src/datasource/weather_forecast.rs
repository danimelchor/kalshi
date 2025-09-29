use async_trait::async_trait;
use bincode::{self, Decode, Encode};
use chrono::{TimeDelta, Utc};
use weather::{
    forecast::{model::Model, parser::parse_report},
    station::Station,
    temperature::Temperature,
};

use crate::datasource::datasource::DataSource;
use protocol::{datetime::SerializableDateTime, protocol::ServiceName};

use anyhow::Result;

#[derive(Encode, Decode, Debug, Clone)]
pub struct WeatherForecast {
    pub temperature: Temperature,
    pub ts: SerializableDateTime,
}

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
        let ts = Utc::now() - TimeDelta::hours(2);
        let forecast = parse_report(&self.station, &self.model, ts, 0).await?;

        Ok(WeatherForecast {
            temperature: forecast.temperature,
            ts: forecast.ts.into(),
        })
    }
}
