use anyhow::Context as _;
use once_cell::sync::Lazy;
use serenity::all::{Channel, ChannelId, Guild, GuildChannel, GuildId};
use serenity::client::Context;

pub static MESSAGE_PREVIEW_CHANNEL_CACHE: Lazy<moka::future::Cache<ChannelId, GuildChannel>> = {
    Lazy::new(|| {
        moka::future::CacheBuilder::new(1000)
            .name("message_preview_channel_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build()
    })
};

pub async fn get_channel_from_cache(
    id: ChannelId,
    guild: GuildId,
    ctx: &Context,
) -> anyhow::Result<GuildChannel> {
    let channel = if let Some(c) = MESSAGE_PREVIEW_CHANNEL_CACHE.get(&id).await {
        Ok(c)
    } else {
        let channels = guild
            .channels(&ctx.http)
            .await
            .context("Failed to get channels.")?;
        if let Some(channel) = channels.get(&id) {
            Ok(channel.clone())
        } else {
            let guild_threads = guild
                .get_active_threads(&ctx.http)
                .await
                .context("Failed to get active threads.")?;
            guild_threads
                .threads
                .iter()
                .find(|c| c.id == id)
                .cloned()
                .context("Failed to find channel.")
        }
    };

    if let Ok(c) = channel {
        MESSAGE_PREVIEW_CHANNEL_CACHE.insert(id, c.clone()).await;
        Ok(c)
    } else {
        channel
    }
}
