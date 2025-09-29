use clap::Parser;
use weather::{
    model::Model,
    parser::{ComputeOptions, parse_report_with_opts},
    station::Station,
};

#[derive(Parser)]
#[command()]
struct Cli {
    #[arg(value_enum, short, long)]
    compute_opts: Option<ComputeOptions>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let station = Station::KNYC;
    let model = Model::HRRR;
    let compute_opts = cli.compute_opts.unwrap_or(ComputeOptions::Precomputed);
    let forecast = parse_report_with_opts(&station, &model, 0, compute_opts).await?;
    print!("{:?}", forecast);
    Ok(())
}
