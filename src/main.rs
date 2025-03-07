use anyhow::{bail, Context, Result};
use clap::Parser;
use teloxide::payloads::{SendMediaGroupSetters as _, SendPhotoSetters};
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto};
use teloxide::RequestError::Network;
use teloxide::{
    payloads::{GetUpdatesSetters, SendMessageSetters},
    requests::{Request, Requester},
    types::{AllowedUpdate, ChatId, InlineKeyboardButton, InlineKeyboardMarkup},
    Bot,
};

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
        /// Sends the message silently. Users will receive a notification with no sound
        #[clap(short, long)]
        silent: bool,

        /// Optional path to an image to attach. Can be specified multiple times
        #[clap(short, long)]
        image: Vec<String>,

        /// Message to send
        message: Option<String>,
    },

    /// Show a prompt and wait for a button press
    Wait {
        /// Sends the message silently. Users will receive a notification with no sound
        #[clap(short, long)]
        silent: bool,

        /// Message to send
        message: String,

        /// Text on the button
        button: Vec<String>,

        #[clap(short, long, default_value = "3600")]
        timeout: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();

    let config = load_config(&args).await.context("Failed to load config")?;

    let bot = Bot::new(config.token);

    match args.command {
        Command::Send { silent, image, mut message } => match image.as_slice() {
            [] => {
                if let Some(message) = message {
                    bot.send_message(ChatId(config.chat_id), message)
                        .disable_notification(silent)
                        .send()
                        .await
                        .context("Failed to send message")?;
                } else {
                    bail!("Nothing to send")
                }
            }
            [image] => {
                bot.send_photo(ChatId(config.chat_id), InputFile::file(image))
                    .disable_notification(silent)
                    .caption(message.unwrap_or_default()) // XXX: is empty string the same as None?
                    .send()
                    .await
                    .context("Failed to send images")?;
            }
            images @ [..] => {
                bot.send_media_group(
                    ChatId(config.chat_id),
                    images.iter().map(|image| {
                        let mut photo = InputMediaPhoto::new(InputFile::file(image));
                        photo.caption = message.take();
                        InputMedia::Photo(photo)
                    }),
                )
                .disable_notification(silent)
                .send()
                .await
                .context("Failed to send images")?;
            }
        },
        Command::Wait { silent, message, button, timeout } => {
            let keyboard = button
                .iter()
                .enumerate()
                .map(|(i, button): (usize, &String)| {
                    [InlineKeyboardButton::callback(button, format!("{}", i))]
                });

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
                .disable_notification(silent)
                .send()
                .await
                .context("Failed to send message")?;

            let update = loop {
                let update_tmp = bot
                    .get_updates()
                    .timeout(timeout)
                    .allowed_updates(vec![AllowedUpdate::CallbackQuery])
                    .send()
                    .await;
                match update_tmp {
                    Err(Network(ref e)) if e.is_timeout() => continue,
                    _ => break update_tmp,
                };
            }
            .context("Failed to get updates")?
            .into_iter()
            .find(|update| match &update.kind {
                teloxide::types::UpdateKind::CallbackQuery(q) => {
                    println!("{}", q.data.clone().expect("How?").as_str());
                    true
                }
                _ => false,
            });

            let Some(update) = update else {
                bot.edit_message_text(
                    sent_message.chat.id,
                    sent_message.id,
                    message + "\n\n(expired)",
                )
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
        }
    }

    Ok(())
}
