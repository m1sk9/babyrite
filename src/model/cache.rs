use anyhow::Context as _;
use once_cell::sync::Lazy;
use serenity::all::{ChannelId, GuildChannel, GuildId};
use serenity::client::Context;

pub static CHANNEL_LIST_CACHE: Lazy<moka::future::Cache<ChannelId, GuildChannel>> =
    Lazy::new(|| {
        moka::future::CacheBuilder::new(100)
            .time_to_idle(std::time::Duration::from_secs(60 * 60))
            .build()
    });

pub async fn get_channel_list_cache(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> anyhow::Result<GuildChannel> {
    match CHANNEL_LIST_CACHE.get(&channel_id).await {
        Some(channel) => {
            tracing::debug!("Found channel in cache: {:?}", &channel);
            Ok(channel)
        }
        None => {
            let guild_channels = guild_id
                .channels(&ctx.http)
                .await
                .context("Failed to retrieve channel list.")?;
            let channel = match guild_channels.get(&channel_id) {
                Some(channel) => channel.clone(),
                None => {
                    let guild_threads = guild_id.get_active_threads(&ctx.http).await?;
                    let thread = match guild_threads.threads.iter().find(|c| c.id == channel_id) {
                        Some(channel) => channel.clone(),
                        None => {
                            anyhow::bail!("Channel not found.");
                        }
                    };
                    thread
                }
            };
            tracing::debug!("Inserting channel into cache: {:?}", &channel);
            CHANNEL_LIST_CACHE.insert(channel_id, channel.clone()).await;
            Ok(channel)
        }
    }
}
