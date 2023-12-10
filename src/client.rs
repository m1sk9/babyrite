use anyhow::Context;
use serenity::{prelude::GatewayIntents, Client};
use tracing::debug;

use crate::event::EvHandler;

pub async fn discord_client(token: &str) -> anyhow::Result<()> {
    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(EvHandler)
        .await
        .context("Failed to create client")?;
    debug!("Discord client created.");

    client.start().await.context("Failed to start client")
}
