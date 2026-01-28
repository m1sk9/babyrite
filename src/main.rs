//! Babyrite - A Discord bot for message link previews.
//!
//! This bot automatically generates previews for Discord message links
//! shared within the same guild.

#![deny(clippy::all)]

mod cache;
mod config;
mod event;
mod preview;

use crate::{
    config::{BabyriteConfig, EnvConfig},
    event::BabyriteEventHandler,
};
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
        GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(BabyriteEventHandler)
    .await
    .expect("Failed to initialize client.");

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
