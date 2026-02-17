//! Babyrite - A Discord bot for message link previews.
//!
//! This bot automatically generates previews for Discord message links
//! shared within the same guild, and expands GitHub permalinks into
//! code blocks.

#![deny(clippy::all)]

mod cache;
mod config;
mod event;
mod expand;
mod utils;

use crate::{
    config::{BabyriteConfig, EnvConfig},
    event::{BabyriteEventHandler, HttpClient},
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

    // Register the shared HTTP client for GitHub API requests
    {
        let mut data = client.data.write().await;
        data.insert::<HttpClient>(reqwest::Client::new());
    }

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
