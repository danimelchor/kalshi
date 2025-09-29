use chrono::{TimeDelta, Utc};
use clap::{Parser, Subcommand};
use weather::{
    forecast::{
        model::Model,
        parser::{ComputeOptions, parse_report_with_opts},
    },
    observations::hourly::NWSHourlyObservationsScraper,
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

        #[arg(long, default_value_t=Model::HRRR)]
        model: Model,
    },
    NWSHourlyObservation,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ModelForecast {
            compute_opts,
            model,
        } => {
            let compute_opts = compute_opts.unwrap_or(ComputeOptions::Precomputed);
            let ts = Utc::now() - TimeDelta::hours(3);
            let forecast =
                parse_report_with_opts(&cli.station, &model, ts, 0, compute_opts).await?;
            println!("{:?}", forecast);
        }
        Commands::NWSHourlyObservation => {
            let mut scraper = NWSHourlyObservationsScraper::new(cli.station, None).await?;
            let temperatures = scraper.scrape().await?.0;
            for temp in temperatures {
                println!("{:?}", temp);
            }
            scraper.close().await?
        }
    }
    Ok(())
}
