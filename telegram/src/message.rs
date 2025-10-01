use anyhow::{Context, Result};
use protocol::protocol;
use serde::{Deserialize, Serialize};
use std::env;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
use tokio::{net::UnixStream, try_join};

#[derive(Serialize, Deserialize, Debug)]
pub struct TelegramMessage {
    title: String,
    message: String,
    extra_data: Option<String>,
}

impl TelegramMessage {
    pub fn to_telegram_text(&self) -> String {
        let mut text = format!("*{}*\n\n{}", self.title, self.message);
        if let Some(extra) = &self.extra_data {
            text.push_str(&format!("\n\n```\n{}\n```", extra));
        }
        text
    }
}

#[derive(Default)]
struct TelegramMessageBuilder {
    title: Option<String>,
    message: Option<String>,
    extra_data: Option<String>,
}

impl TelegramMessageBuilder {
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn extra_data(mut self, extra_data: &str) -> Self {
        self.extra_data = Some(extra_data.to_string());
        self
    }

    pub fn build(self) -> TelegramMessage {
        TelegramMessage {
            title: self.title.unwrap_or_default(),
            message: self.message.unwrap_or_default(),
            extra_data: self.extra_data,
        }
    }
}
