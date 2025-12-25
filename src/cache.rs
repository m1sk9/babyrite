use anyhow::Context as _;
use moka::future::{Cache, CacheBuilder};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::Channel,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

pub type GuildId = Id<GuildMarker>;
pub type ChannelId = Id<ChannelMarker>;
pub type ChannelMap = HashMap<ChannelId, Channel>;

pub struct CacheArgs {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
}

// Cache for guild channels (channel list)
pub static GUILD_CHANNEL_LIST_CACHE: Lazy<Cache<GuildId, ChannelMap>> =
    Lazy::new(|| {
        CacheBuilder::new(500)
            .name("guild_channel_list_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .time_to_live(std::time::Duration::from_secs(43200))
            .build()
    });

// Cache for guild channel
pub static GUILD_CHANNEL_CACHE: Lazy<Cache<ChannelId, Channel>> = Lazy::new(|| {
    CacheBuilder::new(500)
        .name("guild_channel_cache")
        .time_to_idle(std::time::Duration::from_secs(3600))
        .time_to_live(std::time::Duration::from_secs(43200))
        .build()
});

impl CacheArgs {
    pub async fn get(&self, http: &HttpClient) -> anyhow::Result<Channel> {
        // 1. Try to get from channel cache
        match GUILD_CHANNEL_CACHE.get(&self.channel_id).await {
            // 2-a. If found, return it
            Some(channel) => Ok(channel),
            // 2-b. If not found, try to get from channel list cache.
            None => {
                // 3. Try to get from channel list cache
                if let Some(channels) = GUILD_CHANNEL_LIST_CACHE.get(&self.guild_id).await {
                    // 4. If found, try to get the channel from the list
                    return channels
                        .get(&self.channel_id)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Channel not found in cache"));
                }

                // 5. If not found, fetch from API and update the cache
                let channel_list = self.get_channel_list_from_api(http).await?;
                let channel = match channel_list.get(&self.channel_id).cloned() {
                    Some(c) => c,
                    None => {
                        let data = http
                            .active_threads(self.guild_id)
                            .await
                            .context("Failed to get active threads")?
                            .model()
                            .await
                            .context("Failed to deserialize active threads")?;
                        data.threads
                            .into_iter()
                            .find(|t| t.id == self.channel_id)
                            .ok_or_else(|| anyhow::anyhow!("Channel not found in cache"))?
                    }
                };

                // 6. Update the channel cache
                GUILD_CHANNEL_CACHE
                    .insert(self.channel_id, channel.clone())
                    .await;
                Ok(channel)
            }
        }
    }

    async fn get_channel_list_from_api(
        &self,
        http: &HttpClient,
    ) -> anyhow::Result<ChannelMap> {
        let channels = http
            .guild_channels(self.guild_id)
            .await
            .context("Failed to get guild channels")?
            .model()
            .await
            .context("Failed to deserialize guild channels")?;

        let channel_map: ChannelMap = channels.into_iter().map(|c| (c.id, c)).collect();

        GUILD_CHANNEL_LIST_CACHE
            .insert(self.guild_id, channel_map.clone())
            .await;

        Ok(channel_map)
    }
}
