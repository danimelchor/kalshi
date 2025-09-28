use clap::{Parser, Subcommand};
use kalshi::{
    datasource::{datasource::DataSource, example::ExampleComDataSource},
    strategy::{example::ExampleStrategy, strategy::Strategy},
};

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
            let mut source = ExampleComDataSource::default();
            source.run().await.unwrap()
        }
        Commands::Client => {
            let mut strategy = ExampleStrategy::default();
            strategy.run().await.unwrap()
        }
    }
}
