use anyhow::Result;
use serde::Deserialize;
use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::{ChatId, ParseMode},
    Bot,
};

use clap::Parser;

#[derive(Deserialize)]
pub struct Config {
    /// Telegram bot token
    pub token: String,

    /// Telegram chat id
    pub chat_id: i64,
}

#[derive(Clone, Parser)]
pub struct Args {
    /// Path to the config file. If not specified, it will default to /etc/hermes/config.toml
    #[clap(short, long)]
    pub config_file: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Parser)]
pub enum Command {
    Send { message: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();

    let config = config::Config::builder()
        .add_source(
            config::File::with_name(
                &args
                    .config_file
                    .unwrap_or("/etc/hermes/config.toml".to_string()),
            )
            .required(false),
        )
        .add_source(config::Environment::with_prefix("HERMES"))
        .build()?
        .try_deserialize::<Config>()?;

    let bot = Bot::new(config.token);

    match args.command {
        Command::Send { message } => {
            bot.send_message(ChatId(config.chat_id), message)
                .parse_mode(ParseMode::MarkdownV2)
                .send()
                .await?;
        }
    }

    Ok(())
}
