use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum DataSourceName {
    NwsDailyObservations,
    NwsHourlyObservations,
    WeatherForecast,
}
