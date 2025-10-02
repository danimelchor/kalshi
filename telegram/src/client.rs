use ::protocol::protocol::Event;
use anyhow::Result;
use protocol::protocol;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TelegramMessage {
    title: Option<String>,
    body: Option<String>,
    extra_data: Option<String>,
}

fn escape_markdown_v2(text: &str) -> String {
    let chars_to_escape = r#"_*[]()~`>#+-=|{}.!"#;
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if chars_to_escape.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

impl TelegramMessage {
    pub fn to_telegram_text(&self) -> String {
        let title = escape_markdown_v2(self.title.as_deref().unwrap_or(""));
        let body = escape_markdown_v2(self.body.as_deref().unwrap_or(""));
        let mut text = format!("*{}*\n\n{}", title, body);
        if let Some(extra) = &self.extra_data {
            let extra = escape_markdown_v2(extra);
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
    id: u32,
}

impl TelegramClient {
    pub async fn start() -> Result<Self> {
        let stream = protocol::create_unix_stream(protocol::ServiceName::Telegram).await?;
        Ok(Self { stream, id: 0 })
    }

    async fn send_message(&mut self, message: TelegramMessage) -> Result<()> {
        let event = Event::new(self.id, message);
        protocol::write_one(&event, &mut self.stream).await?;
        self.id += 1;
        Ok(())
    }

    pub fn message(&mut self) -> WIPMessage<'_> {
        WIPMessage::new(self)
    }
}
