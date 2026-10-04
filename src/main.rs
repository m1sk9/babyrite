//! Babyrite - A Discord bot for message link previews.
//!
//! This bot automatically generates previews for Discord message links
//! shared within the same guild, and expands GitHub permalinks into
//! code blocks.

#![deny(clippy::all)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod cache;
mod config;
mod context;
mod event;
mod expand;
mod reply;
mod utils;

use crate::{
    config::{BabyriteConfig, EnvConfig, LogFormat},
    context::BotContext,
};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use twilight_gateway::{ConfigBuilder, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{Activity, ActivityType, MinimalActivity, Status},
};

/// The gateway events [`event::handle`] acts on.
///
/// Everything else is skipped before deserialization, which matters most for
/// the large guild payloads; `GUILD_CREATE` is kept only to invalidate roles.
const WANTED_EVENTS: EventTypeFlags = EventTypeFlags::READY
    .union(EventTypeFlags::MESSAGE_CREATE)
    .union(EventTypeFlags::CHANNEL_CREATE)
    .union(EventTypeFlags::CHANNEL_UPDATE)
    .union(EventTypeFlags::CHANNEL_DELETE)
    .union(EventTypeFlags::THREAD_CREATE)
    .union(EventTypeFlags::THREAD_UPDATE)
    .union(EventTypeFlags::THREAD_DELETE)
    .union(EventTypeFlags::ROLE_CREATE)
    .union(EventTypeFlags::ROLE_UPDATE)
    .union(EventTypeFlags::ROLE_DELETE)
    .union(EventTypeFlags::GUILD_CREATE);

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

    let token = envs.discord_api_token.clone();
    let ctx = Arc::new(BotContext {
        http: twilight_http::Client::new(token.clone()),
        // The raw-content read caps bytes, not time: without a timeout a stalled
        // server would hold the fetch open indefinitely.
        github: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client."),
    });

    let mut activity = Activity::from(MinimalActivity {
        kind: ActivityType::Custom,
        name: "Custom Status".into(),
        url: None,
    });
    activity.state = Some(format!("Running v{}", env!("CARGO_PKG_VERSION")));
    // Sent with IDENTIFY rather than set once connected, so every re-identify
    // after a lost session restores it without the bot doing anything.
    let presence = UpdatePresencePayload::new(vec![activity], false, None, Status::Online)
        .expect("presence has one activity");

    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
    let shard_config = ConfigBuilder::new(token, intents)
        .presence(presence)
        .build();
    let mut shard = Shard::with_config(ShardId::ONE, shard_config);

    while let Some(item) = shard.next_event(WANTED_EVENTS).await {
        match item {
            Ok(event) => {
                tokio::spawn(event::handle(Arc::clone(&ctx), event));
            }
            // Not fatal: the shard reconnects on its own.
            Err(source) => tracing::warn!(?source, "error receiving gateway event"),
        }
    }

    // The stream ends only on a close code that reconnecting cannot fix (e.g.
    // an invalid token or disallowed intents), so exit non-zero and let the
    // supervisor decide.
    anyhow::bail!("gateway connection closed fatally")
}
