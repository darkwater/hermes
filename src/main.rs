use anyhow::{bail, Context, Result};
use clap::Parser;
use teloxide::{
    dispatching::dialogue::GetChatId,
    payloads::{GetUpdatesSetters, SendMessageSetters},
    requests::{Request, Requester},
    types::{AllowedUpdate, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, UpdateKind::CallbackQuery},
    Bot,
};
use std::process::ExitCode;

use crate::config::load_config;

mod config;

#[derive(Clone, Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Path to the config file. If not specified, it will default to /etc/hermes/config.toml
    #[clap(short, long)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Parser)]
pub enum Command {
    /// Send a single message to the configured chat
    Send {
        /// Message to send
        message: String,
    },

    /// Show a prompt and wait for a button press
    Wait {
        /// Message to send
        message: String,

        /// Text on the buttons
        buttons: Vec<String>,

        #[clap(short, long, default_value = "3600")]
        timeout: u32,
    },
}

#[tokio::main]
async fn main() -> Result<ExitCode> {
    inner()
    .await
    .or(Ok(ExitCode::from(u8::MAX)))
}
async fn inner() -> Result<ExitCode> {
    pretty_env_logger::init();

    let args = Args::parse();

    let config = load_config(&args).await.context("Failed to load config")?;

    let bot = Bot::new(config.token);

    match args.command {
        Command::Send { message } => {
            bot.send_message(ChatId(config.chat_id), message)
                .send()
                .await
                .context("Failed to send message")?;
        }
        Command::Wait { message, buttons, timeout } => {
            let mut keyboard = Vec::new();//[[InlineKeyboardButton::callback(button, "button")]];
            for (i, button) in buttons.iter().enumerate() {
                keyboard.push([InlineKeyboardButton::callback(button, i.to_string())]);
            }

            if let Some(update) = bot
                .get_updates()
                .allowed_updates(vec![AllowedUpdate::CallbackQuery])
                .send()
                .await
                .context("Failed to get updates to clear queue in advance")?
                .into_iter()
                .last()
            {
                bot.get_updates()
                    .offset(update.id + 1)
                    .send()
                    .await
                    .context("Failed to clear queue in advance")?;
            }

            let sent_message = bot
                .send_message(ChatId(config.chat_id), message.clone())
                .reply_markup(InlineKeyboardMarkup::new(keyboard))
                .send()
                .await
                .context("Failed to send message")?;

            let update = bot
                .get_updates()
                .timeout(timeout)
                .allowed_updates(vec![AllowedUpdate::CallbackQuery])
                .send()
                .await
                .context("Failed to get updates")?
                .into_iter()
                .find(|update| {
                    matches!(update.kind, teloxide::types::UpdateKind::CallbackQuery(_))
                });

            let Some(update) = update else {
                bot.edit_message_text(sent_message.chat.id, sent_message.id, message + "\n\n(expired)")
                    .send()
                    .await
                    .context("Failed to edit message after press")?;

                bail!("Timed out waiting for button press")
            };


            bot.get_updates()
                .offset(update.id + 1)
                .allowed_updates(vec![AllowedUpdate::CallbackQuery])
                .send()
                .await
                .context("Failed to acknowledge button press")?;

            bot.edit_message_text(sent_message.chat.id, sent_message.id, message + "\n\n(pressed)")
                .send()
                .await
                .context("Failed to edit message after press")?;

            let CallbackQuery(cb_data) = update.kind else {
                unreachable!();
            };

            let return_code = cb_data.data.map(|i| i.parse::<u8>().ok().unwrap_or(u8::MAX) ).unwrap();

            return Ok(ExitCode::from(return_code))
        }
    }

    Ok(ExitCode::from(0))
}
