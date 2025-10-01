use anyhow::Result;
use telegram::bot::TelegramBot;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let bot = TelegramBot::default();
    bot.run().await?;
    Ok(())
}
