pub mod daily_weather_observations;
pub mod datasource;
pub mod hourly_weather_observations;
pub mod name;
pub mod weather_forecast;

use crate::datasource::name::DataSourceName;
use anyhow::Result;
use clap::Args;
use daily_weather_observations::DailyWeatherObservationDataSource;
use datasource::DataSource;
use hourly_weather_observations::HourlyWeatherObservationDataSource;
use weather::{forecast::model::Model, station::Station};
use weather_forecast::WeatherForecastDataSource;

#[derive(Debug, Clone, Args)]
pub struct DataSourceCommand {
    name: DataSourceName,
}

pub async fn run_data_source(command: &DataSourceCommand) -> Result<()> {
    match command.name {
        DataSourceName::WeatherForecast => {
            let mut source = WeatherForecastDataSource::new(Station::KNYC, Model::HRRR);
            source.run().await.unwrap()
        }
        DataSourceName::NwsHourlyObservations => {
            let mut source = HourlyWeatherObservationDataSource::new(Station::KNYC).await?;
            source.run().await.unwrap()
        }
        DataSourceName::NwsDailyObservations => {
            let mut source = DailyWeatherObservationDataSource::new(Station::KNYC);
            source.run().await.unwrap()
        }
    }

    Ok(())
}
