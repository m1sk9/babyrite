use anyhow::Context as _;
use moka::future::{Cache, CacheBuilder};
use once_cell::sync::Lazy;
use serenity::all::{ChannelId, Guild, GuildChannel, GuildId};
use serenity::client::Context;

pub static CHANNEL_CACHE: Lazy<
    Cache<ChannelId, GuildChannel>,
> = {
    Lazy::new(|| {
        moka::future::CacheBuilder::new(1000)
            .name("message_preview_channel_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build()
    })
};

pub static GUILD_CACHE: Lazy<
    Cache<GuildId, Guild>
> = {
    Lazy::new(|| {
        CacheBuilder::new(1000)
            .name("message_preview_guild_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build()
    })
};

pub async fn get_channel_from_cache(
    id: ChannelId,
    guild: GuildId,
    ctx: &Context,
) -> anyhow::Result<GuildChannel> {
    let channel = match CHANNEL_CACHE.get(&id).await {
        Some(c) => Ok(c),
        _ => {
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
        }
    };

    match channel {
        Ok(c) => {
            CHANNEL_CACHE.insert(id, c.clone()).await;
            Ok(c)
        }
        _ => channel,
    }
}
