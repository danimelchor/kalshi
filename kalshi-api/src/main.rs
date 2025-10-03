use anyhow::Result;
use clap::Parser;
use kalshi_api::{
    client::BaseUrl,
    keys::{ApiKey, PrivateKey},
    markets::MarketsApiClient,
};
use kalshi_api_spec::{event::EventResponse, ticker::EventTicker};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    private_key: PathBuf,

    #[arg(long)]
    event_ticker: EventTicker,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dotenvy::dotenv()?;

    let api_key = ApiKey::from_env()?;
    let private_key = PrivateKey::from_file(cli.private_key).await?;
    let client = MarketsApiClient::new(api_key, private_key, BaseUrl::Prod);
    let response: EventResponse = client.get_event(&cli.event_ticker).await?;
    let as_json = serde_json::to_string_pretty(&response)?;
    eprintln!("Response:");
    println!("\n{}", as_json);

    Ok(())
}
