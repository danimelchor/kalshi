use anyhow::Result;
use chrono::{TimeDelta, Utc};
use chrono_tz::Tz;
use clap::{Parser, Subcommand};
use futures::StreamExt;
use std::pin::pin;
use weather::{
    forecast::{
        fetcher::ForecastFetcher,
        model::{ComputeOptions, Model},
    },
    observations::{daily::NWSDailyObservationFetcher, hourly::NWSHourlyObservationsScraper},
    station::Station,
};

#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, value_enum, default_value_t=Station::KNYC)]
    station: Station,
}

#[derive(Subcommand)]
enum Commands {
    ModelForecast {
        #[arg(value_enum, short, long)]
        compute_opts: Option<ComputeOptions>,

        #[arg(long, value_enum, default_value_t=Model::HRRR)]
        model: Model,
    },
    NWSHourlyObservation,
    NWSDailyObservation,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ModelForecast {
            compute_opts,
            model,
        } => {
            let compute_opts = compute_opts.unwrap_or(ComputeOptions::Precomputed);
            let ts = Utc::now().with_timezone(&Tz::America__New_York) - TimeDelta::hours(1);
            let mut fetcher = ForecastFetcher::new(cli.station, model);
            let mut result = pin!(fetcher.fetch());
            while let Some(forecast) = result.next().await {
                println!("{:?}", forecast);
            }
        }
        Commands::NWSHourlyObservation => {
            let mut scraper = NWSHourlyObservationsScraper::new(cli.station, None).await?;
            let temperatures = scraper.scrape().await?.0;
            for temp in temperatures {
                println!("{:?}", temp);
            }
            scraper.close().await?
        }
        Commands::NWSDailyObservation => {
            let mut fetcher = NWSDailyObservationFetcher::new(cli.station, None);
            let report = fetcher.fetch(1, false).await?;
            println!("{:?}", report);
        }
    }
    Ok(())
}
