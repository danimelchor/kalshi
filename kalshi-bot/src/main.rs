use clap::{Parser, Subcommand};
use kalshi_bot::{
    datasource::{datasource::DataSource, weather_forecast::WeatherForecastDataSource},
    strategy::{example::ExampleStrategy, strategy::Strategy},
};
use weather::{model::Model, station::Station};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Publisher,
    Client,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Publisher => {
            let mut source = WeatherForecastDataSource::new(Station::KNYC, Model::HRRR);
            source.run().await.unwrap()
        }
        Commands::Client => {
            let mut strategy = ExampleStrategy::default();
            strategy.run().await.unwrap()
        }
    }
}
