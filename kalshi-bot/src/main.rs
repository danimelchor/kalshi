use anyhow::Result;
use clap::{Parser, Subcommand};
use kalshi_bot::{
    datasource::{
        daily_weather_observations::DailyWeatherObservationDataSource, datasource::DataSource,
        hourly_weather_observations::HourlyWeatherObservationDataSource,
        weather_forecast::WeatherForecastDataSource,
    },
    strategy::{
        dump_if_temp_higher::DumpIfTempHigher, forecast_notifier::ForecastNotifier,
        strategy::Strategy,
    },
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
    WeatherForecastPublisher,
    NwsDailyObservationPublisher,
    NwsHourlyObservationPublisher,
    ForecastNotifier,
    DumpIfTempHigher,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::WeatherForecastPublisher => {
            let mut source = WeatherForecastDataSource::new(Station::KNYC, Model::HRRR);
            source.run().await.unwrap()
        }
        Commands::NwsHourlyObservationPublisher => {
            let mut source = HourlyWeatherObservationDataSource::new(Station::KNYC).await?;
            source.run().await.unwrap()
        }
        Commands::NwsDailyObservationPublisher => {
            let mut source = DailyWeatherObservationDataSource::new(Station::KNYC);
            source.run().await.unwrap()
        }
        Commands::ForecastNotifier => {
            let mut strategy = ForecastNotifier::default();
            strategy.run().await.unwrap()
        }
        Commands::DumpIfTempHigher => {
            let mut strategy = DumpIfTempHigher::default();
            strategy.run().await.unwrap()
        }
    }

    Ok(())
}
