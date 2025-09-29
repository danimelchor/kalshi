use anyhow::Result;
use clap::{Parser, Subcommand};
use kalshi_bot::{
    datasource::{
        DataSourceCommand, daily_weather_observations::DailyWeatherObservationDataSource,
        datasource::DataSource, hourly_weather_observations::HourlyWeatherObservationDataSource,
        run_data_source, weather_forecast::WeatherForecastDataSource,
    },
    strategy::{StrategyCommand, run_strategy},
};
use weather::{forecast::model::Model, station::Station};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    DataSource(DataSourceCommand),
    Strategy(StrategyCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::DataSource(subcommand) => run_data_source(subcommand).await?,
        Commands::Strategy(subcommand) => run_strategy(subcommand).await?,
    }

    Ok(())
}
