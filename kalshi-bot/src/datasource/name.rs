use clap::ValueEnum;
use strum::Display;
use strum_macros::EnumIter;

#[derive(Debug, Clone, ValueEnum, Display, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum DataSourceName {
    NwsDailyReport,
    NwsHourlyTimeseries,
    NwsHourlyTable,
    WeatherForecast,
}
