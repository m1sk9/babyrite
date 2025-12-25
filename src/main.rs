#![deny(clippy::all)]

mod cache;
mod config;
mod event;
mod preview;

use std::sync::Arc;

use crate::config::{BabyriteConfig, EnvConfig};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    payload::outgoing::UpdatePresence,
    presence::{ActivityType, MinimalActivity, Status},
};

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

    // Create HTTP client
    let http = Arc::new(HttpClient::new(envs.discord_api_token.clone()));

    // Create gateway shard with intents
    let intents = Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES;
    let mut shard = Shard::new(ShardId::ONE, envs.discord_api_token.clone(), intents);

    // Event loop
    loop {
        let event = match shard.next_event(EventTypeFlags::all()).await {
            Some(Ok(event)) => event,
            Some(Err(source)) => {
                tracing::warn!(?source, "error receiving event");
                // Continue on recoverable errors
                continue;
            }
            None => break,
        };

        match &event {
            Event::Ready(ready) => {
                event::on_ready(ready);

                // Set presence/activity
                let activity = MinimalActivity {
                    kind: ActivityType::Custom,
                    name: format!("Running v{}", env!("CARGO_PKG_VERSION")),
                    url: None,
                };

                let presence = UpdatePresence::new(vec![activity.into()], false, None, Status::Online)?;

                shard.command(&presence);
            }
            Event::MessageCreate(msg) => {
                let http = Arc::clone(&http);
                let message = msg.0.clone();
                tokio::spawn(async move {
                    event::on_message(http, message).await;
                });
            }
            _ => {}
        }
    }

    Ok(())
}
