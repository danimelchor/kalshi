use anyhow::Result;
use clap::{Parser, Subcommand};
use futures::StreamExt;
use std::pin::pin;
use weather::{
    forecast::{
        fetcher::ForecastFetcher,
        model::{ComputeOptions, Model},
    },
    observations::{
        nws_daily_report::NWSDailyReportFetcher, nws_hourly_table::NWSHourlyTableFetcher,
        nws_hourly_timeseries::NWSHourlyTimeseriesScraper,
    },
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
    NWSHourlyTimeseries,
    NWSHourlyTable,
    NWSDailyReport,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ModelForecast {
            compute_opts,
            model,
        } => {
            let mut fetcher = ForecastFetcher::new(cli.station, model, 12, compute_opts);
            let mut result = pin!(fetcher.fetch());
            while let Some(forecast) = result.next().await {
                println!("{:?}", forecast);
            }
        }
        Commands::NWSHourlyTimeseries => {
            let mut scraper = NWSHourlyTimeseriesScraper::new(cli.station, None).await?;
            let result = scraper.scrape().await?.0;
            let last_temp = result.last().unwrap();
            println!("{:?}", last_temp);
            scraper.close().await?
        }
        Commands::NWSDailyReport => {
            let mut fetcher = NWSDailyReportFetcher::new(cli.station, None);
            let report = fetcher.fetch(1, false).await?;
            println!("{:?}", report);
        }
        Commands::NWSHourlyTable => {
            let mut fetcher = NWSHourlyTableFetcher::new(cli.station, None);
            let result = fetcher.fetch().await?.0;
            let last_temp = result.last().unwrap();
            println!("{:?}", last_temp);
        }
    }
    Ok(())
}
