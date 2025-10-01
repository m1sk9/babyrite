use anyhow::Context as _;
use moka::future::{Cache, CacheBuilder};
use once_cell::sync::Lazy;
use serenity::all::{ChannelId, GuildChannel, GuildId};
use serenity::client::Context;
use std::collections::HashMap;
use thiserror::Error;

pub struct CacheArgs {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
}

#[derive(Error, Debug)]
enum CacheError {
    #[error("Entity not found in cache")]
    CacheNotFound,
}

// Cache for guild channels (channel list)
pub static GUILD_CHANNEL_LIST_CACHE: Lazy<Cache<GuildId, HashMap<ChannelId, GuildChannel>>> = {
    Lazy::new(|| {
        CacheBuilder::new(500)
            .name("guild_channel_list_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .time_to_live(std::time::Duration::from_secs(43200))
            .build()
    })
};

// Cache for guild channel
pub static GUILD_CHANNEL_CACHE: Lazy<Cache<ChannelId, GuildChannel>> = {
    Lazy::new(|| {
        CacheBuilder::new(500)
            .name("guild_channel_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .time_to_live(std::time::Duration::from_secs(43200))
            .build()
    })
};

impl CacheArgs {
    pub async fn get_channel_from_cache(&self, ctx: &Context) -> anyhow::Result<GuildChannel> {
        // 1. Try to get from channel cache
        match GUILD_CHANNEL_CACHE.get(&self.channel_id).await {
            // 2-a. If found, return it
            Some(channel) => Ok(channel),
            // 2-b. If not found, try to get from channel list cache.
            None => {
                // 3. Try to get from channel list cache
                if let Some(channels) = GUILD_CHANNEL_LIST_CACHE.get(&self.guild_id).await {
                    // 4. If found, try to get the channel from the list
                    channels
                        .get(&self.channel_id)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Channel not found in cache"))?;
                }

                // 5. If not found, fetch from API and update the cache
                let channel_list = self.get_channel_list_from_api(ctx).await?;
                let channel = match channel_list.get(&self.channel_id).cloned() {
                    Some(c) => c,
                    None => {
                        let data = self
                            .guild_id
                            .get_active_threads(&ctx.http)
                            .await
                            .context("Failed to get active threads")?;
                        data.threads
                            .iter()
                            .find(|t| t.id == self.channel_id)
                            .cloned()
                            .ok_or(CacheError::CacheNotFound)?
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
        ctx: &Context,
    ) -> anyhow::Result<HashMap<ChannelId, GuildChannel>> {
        let guild = ctx
            .http
            .get_guild(self.guild_id)
            .await
            .context("Failed to get guild")?;
        let channels = guild
            .channels(&ctx)
            .await
            .context("Failed to get channel list")?;

        GUILD_CHANNEL_LIST_CACHE
            .insert(self.guild_id, channels.clone())
            .await;

        Ok(channels)
    }
}
