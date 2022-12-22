use anyhow::{Context, Result};
use clap::Parser;
use teloxide::{
    requests::{Request, Requester},
    types::ChatId,
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
        /// Message to send
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
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
    }

    Ok(())
}
