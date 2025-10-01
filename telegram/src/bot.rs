use anyhow::{Context, Result};
use protocol::protocol;
use std::env;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
use tokio::{net::UnixStream, try_join};

use crate::message::TelegramMessage;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "display the status of the service.")]
    Status(String),
}

pub struct TelegramBot {
    bot: Bot,
    chat_id: ChatId,
}

impl Default for TelegramBot {
    fn default() -> Self {
        let bot = Bot::from_env();
        let chat_id: i64 = env::var("TELOXIDE_CHAT_ID")
            .expect("TELOXIDE_CHAT_ID env var is not set")
            .parse()
            .expect("Chat id is not a number");
        Self {
            bot,
            chat_id: ChatId(chat_id),
        }
    }
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Status(service) => {
            bot.send_message(msg.chat.id, format!("The service is {service}."))
                .await?
        }
    };

    Ok(())
}

impl TelegramBot {
    async fn start_command_bot(&self) -> Result<()> {
        let chat_id = self.chat_id;
        let handler = Update::filter_message()
            .filter(move |msg: Message| msg.chat.id == chat_id)
            .filter_command::<Command>()
            .endpoint(answer);

        Dispatcher::builder(self.bot.clone(), handler)
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    async fn log(&self, message: TelegramMessage) {
        let chat_id = self.chat_id;
        let text = message.to_telegram_text();
        if let Err(err) = self
            .bot
            .send_message(chat_id, &text)
            .parse_mode(ParseMode::MarkdownV2)
            .await
        {
            eprintln!(
                "Error logging message to telegram: {} with error {}",
                text, err
            )
        }
    }

    async fn handle_client(&self, mut stream: UnixStream) {
        loop {
            match protocol::read::<TelegramMessage>(&mut stream).await {
                Ok(event) => self.log(event.message).await,
                Err(err) => eprintln!("Error reading from unix stream: {}", err),
            }
        }
    }

    async fn start_logger(&self) -> Result<()> {
        let bind = protocol::create_unix_bind(protocol::ServiceName::Telegram).await?;
        println!("Listening to messages");
        loop {
            let (stream, _) = bind.accept().await?;
            self.handle_client(stream).await;
        }
    }

    pub async fn run(&self) -> Result<()> {
        let command_bot = self.start_command_bot();
        let logger = self.start_logger();
        let _ = try_join!(command_bot, logger);
        Ok(())
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

    pub async fn send_message(&mut self, message: &TelegramMessage) -> Result<()> {
        let buf = bitcode::serialize(message).context("Serializing telegram message")?;
        protocol::write(&buf, &mut self.stream).await?;
        Ok(())
    }
}
