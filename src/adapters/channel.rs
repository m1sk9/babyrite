use anyhow::Context;
use serenity::{
    http::Http,
    model::prelude::{ChannelId, GuildChannel, GuildId},
};
use std::sync::Arc;
use tracing::debug;
use tracing::info;

pub async fn get_channel(
    channel_id: ChannelId,
    guild_id: GuildId,
    http: &Arc<Http>,
) -> anyhow::Result<GuildChannel> {
    let guild_channels = guild_id
        .channels(&http)
        .await
        .context("Failed to retrieve channel list.")?;

    let channel = match guild_channels.get(&channel_id) {
        Some(channel) => {
            debug!("Channel found from Discord API: {:?}.", channel);
            channel.clone()
        }
        None => {
            let guild_threads = guild_id.get_active_threads(http).await?;
            let thread = match guild_threads.threads.iter().find(|c| c.id == channel_id) {
                Some(channel) => {
                    debug!("Thread found from Discord API: {:?}.", channel);
                    channel.clone()
                }
                None => {
                    return Err(anyhow::anyhow!("Channel not found."));
                }
            };
            thread
        }
    };

    info!("--- Channel acquisition is complete.");
    Ok(channel)
}
