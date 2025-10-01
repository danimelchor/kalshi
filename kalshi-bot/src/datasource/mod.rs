pub mod daily_weather_report;
pub mod datasource;
pub mod hourly_weather_table;
pub mod hourly_weather_timeseries;
pub mod name;
pub mod weather_forecast;

use crate::datasource::{hourly_weather_table::HourlyWeatherTableSource, name::DataSourceName};
use anyhow::Result;
use clap::Args;
use daily_weather_report::DailyWeatherReportSource;
use datasource::DataSource;
use hourly_weather_timeseries::HourlyWeatherTimeseriesSource;
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
        DataSourceName::NwsHourlyTimeseries => {
            let mut source = HourlyWeatherTimeseriesSource::new(Station::KNYC).await?;
            source.run().await.unwrap()
        }
        DataSourceName::NwsHourlyTable => {
            let mut source = HourlyWeatherTableSource::new(Station::KNYC);
            source.run().await.unwrap()
        }
        DataSourceName::NwsDailyReport => {
            let mut source = DailyWeatherReportSource::new(Station::KNYC);
            source.run().await.unwrap()
        }
    }

    Ok(())
}
