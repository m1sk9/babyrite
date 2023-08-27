use anyhow::Context;
use serenity::{prelude::GatewayIntents, Client};

use crate::event::EvHandler;

pub async fn discord_client(token: String) -> anyhow::Result<()> {
    let intents = GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(EvHandler)
        .await
        .context("Failed to create client")?;

    client.start().await.context("Failed to start client")
}
