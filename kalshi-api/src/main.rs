use anyhow::Result;
use clap::Parser;
use kalshi_api::{
    client::{BaseUrl, KalshiApiClient, SafeSend},
    keys::{ApiKey, PrivateKey},
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

    let ticker: String = cli.event_ticker.into();
    let api_key = ApiKey::from_env()?;
    let private_key = PrivateKey::from_file(cli.private_key).await?;
    let client = KalshiApiClient::new(api_key, private_key, BaseUrl::Prod);

    let response: EventResponse = client
        .get(&format!("/trade-api/v2/events/{ticker}"))?
        .safe_send()
        .await?;
    println!("Response: {:?}", response);

    Ok(())
}
