//! Babyrite - A Discord bot for message link previews.
//!
//! This bot automatically generates previews for Discord message links
//! shared within the same guild, and expands GitHub permalinks into
//! code blocks.

#![deny(clippy::all)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod cache;
mod config;
mod event;
mod expand;
mod utils;

use crate::{
    config::{BabyriteConfig, EnvConfig, LogFormat},
    event::{BabyriteEventHandler, HttpClient},
};
use serenity::all::GatewayIntents;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    BabyriteConfig::init()?;
    let envs = EnvConfig::get();
    let config = BabyriteConfig::get();

    // `RUST_LOG` takes precedence; otherwise fall back to the `[log] level`
    // configured in `config.toml` (defaults to `babyrite=info`).
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log.level.clone()));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match config.resolved_log_format() {
        LogFormat::Json => builder.json().init(),
        LogFormat::Compact => builder.compact().init(),
    }
    tracing::debug!("Config: {:?}", config);

    let mut client = serenity::Client::builder(
        &envs.discord_api_token,
        GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS,
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
