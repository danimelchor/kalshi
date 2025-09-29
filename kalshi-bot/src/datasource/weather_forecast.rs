use async_trait::async_trait;
use bincode::{self, Decode, Encode};
use chrono::{TimeDelta, Utc};
use weather::{model::Model, parser::parse_report, station::Station, temperature::Temperature};

use crate::{
    datasource::datasource::DataSource,
    protocol::{datetime::SerializableDateTime, protocol::ServiceName},
};

// Example.com data structure
#[derive(Encode, Decode, Debug, Clone)]
pub struct WeatherForecast {
    pub temperature: Temperature,
    pub ts: SerializableDateTime,
}

// Implementation that fetches example.com
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

    async fn fetch_data(
        &mut self,
    ) -> Result<WeatherForecast, Box<dyn std::error::Error + Send + Sync>> {
        let ts = Utc::now() - TimeDelta::hours(2);
        let forecast = parse_report(&self.station, &self.model, ts, 0).await?;

        Ok(WeatherForecast {
            temperature: forecast.temperature,
            ts: forecast.ts.into(),
        })
    }
}
