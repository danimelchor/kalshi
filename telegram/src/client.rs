use anyhow::{Context, Result};
use protocol::protocol;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TelegramMessage {
    title: Option<String>,
    body: Option<String>,
    extra_data: Option<String>,
}

impl TelegramMessage {
    pub fn to_telegram_text(&self) -> String {
        let mut text = format!(
            "*{}*\n\n{}",
            self.title.as_deref().unwrap_or(""),
            self.body.as_deref().unwrap_or("")
        );
        if let Some(extra) = &self.extra_data {
            text.push_str(&format!("\n\n```\n{}\n```", extra));
        }
        text
    }
}

pub struct WIPMessage<'a> {
    message: TelegramMessage,
    client: &'a mut TelegramClient,
}

impl<'a> WIPMessage<'a> {
    pub fn new(client: &'a mut TelegramClient) -> Self {
        Self {
            client,
            message: TelegramMessage::default(),
        }
    }

    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.message.title = Some(title.into());
        self
    }

    pub fn with_body<T: Into<String>>(mut self, body: T) -> Self {
        self.message.body = Some(body.into());
        self
    }

    pub fn with_extra_data<T: Into<String>>(mut self, extra_data: T) -> Self {
        self.message.extra_data = Some(extra_data.into());
        self
    }

    pub async fn send(self) -> Result<()> {
        self.client.send_message(self.message).await
    }
}

pub struct TelegramClient {
    stream: UnixStream,
}

impl TelegramClient {
    pub async fn start() -> Result<Self> {
        let stream = protocol::create_unix_stream(protocol::ServiceName::Telegram).await?;
        Ok(Self { stream })
    }

    async fn send_message(&mut self, message: TelegramMessage) -> Result<()> {
        let buf = bitcode::serialize(&message).context("Serializing telegram message")?;
        protocol::write(&buf, &mut self.stream).await?;
        Ok(())
    }

    pub fn message(&mut self) -> WIPMessage<'_> {
        WIPMessage::new(self)
    }
}
