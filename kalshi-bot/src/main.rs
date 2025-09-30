use anyhow::Result;
use clap::{Parser, Subcommand};
use kalshi_bot::{
    datasource::{DataSourceCommand, run_data_source},
    strategy::{StrategyCommand, run_strategy},
    system::start_system,
};

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
    System,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::DataSource(subcommand) => run_data_source(subcommand).await?,
        Commands::Strategy(subcommand) => run_strategy(subcommand).await?,
        Commands::System => start_system().await?,
    }

    Ok(())
}
