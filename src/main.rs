#![deny(clippy::all)]

mod config;
mod event;
mod message;
mod utils;

use crate::config::{BabyriteConfig, EnvConfig};
use serenity::all::GatewayIntents;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    BabyriteConfig::init()?;
    let envs = EnvConfig::get();

    match BabyriteConfig::get().json_logging {
        true => tracing_subscriber::fmt().json().init(),
        false => tracing_subscriber::fmt().compact().init(),
    }
    tracing::debug!("Config: {:?}", BabyriteConfig::get());

    let mut client = serenity::Client::builder(
        &envs.discord_api_token,
        GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILD_MESSAGE_REACTIONS,
    )
    .event_handler(event::preview::PreviewHandler)
    .event_handler(event::ready::ReadyHandler)
    .await
    .expect("Failed to initialize client.");

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
