use std::env;

use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};

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
    chat_id: String,
}

impl Default for TelegramBot {
    fn default() -> Self {
        let bot = Bot::from_env();
        let chat_id = env::var("TELOXIDE_CHAT_ID").expect("TELOXIDE_CHAT_ID env var is not set");
        Self { bot, chat_id }
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
    pub async fn run(&self) -> Result<()> {
        self.bot.send_message(self.chat_id.clone(), "foo").await?;

        let chat_id = self.chat_id.clone();
        let handler = Update::filter_message()
            .filter(move |msg: Message| msg.chat.id.to_string() == chat_id)
            .filter_command::<Command>()
            .endpoint(answer);

        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
        Ok(())
    }
}
