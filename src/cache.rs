//! Cache module for guild channels.
//!
//! This module provides caching functionality for guild channels using moka cache.
//! It includes two caches:
//! - [`GUILD_CHANNEL_LIST_CACHE`]: Caches the list of channels for each guild.
//! - [`GUILD_CHANNEL_CACHE`]: Caches individual guild channels.
//!
//! The [`CacheArgs`] struct is used to retrieve channels from the cache or fetch them from the API if not found.

use anyhow::Context as _;
use moka::future::{Cache, CacheBuilder};
use serenity::all::{ChannelId, GuildChannel, GuildId};
use serenity::client::Context;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Arguments for cache operations.
pub struct CacheArgs {
    /// The ID of the guild.
    pub guild_id: GuildId,
    /// The ID of the channel.
    pub channel_id: ChannelId,
}

/// Cache for guild channel lists.
///
/// Maps guild IDs to their channel lists. TTL: 12 hours, TTI: 1 hour.
pub static GUILD_CHANNEL_LIST_CACHE: LazyLock<Cache<GuildId, HashMap<ChannelId, GuildChannel>>> = {
    LazyLock::new(|| {
        CacheBuilder::new(500)
            .name("guild_channel_list_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .time_to_live(std::time::Duration::from_secs(43200))
            .build()
    })
};

/// Cache for individual guild channels.
///
/// Maps channel IDs to their channel data. TTL: 12 hours, TTI: 1 hour.
pub static GUILD_CHANNEL_CACHE: LazyLock<Cache<ChannelId, GuildChannel>> = {
    LazyLock::new(|| {
        CacheBuilder::new(500)
            .name("guild_channel_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .time_to_live(std::time::Duration::from_secs(43200))
            .build()
    })
};

impl CacheArgs {
    /// Retrieves a guild channel from cache or fetches it from the API.
    ///
    /// The lookup order is:
    /// 1. Individual channel cache
    /// 2. Guild channel list cache
    /// 3. Discord API (with cache update)
    pub async fn get(&self, ctx: &Context) -> anyhow::Result<GuildChannel> {
        match GUILD_CHANNEL_CACHE.get(&self.channel_id).await {
            Some(channel) => Ok(channel),
            None => {
                if let Some(channels) = GUILD_CHANNEL_LIST_CACHE.get(&self.guild_id).await {
                    return channels
                        .get(&self.channel_id)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Channel not found in cache"));
                }

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
                            .ok_or_else(|| anyhow::anyhow!("Channel not found in cache"))?
                    }
                };

                GUILD_CHANNEL_CACHE
                    .insert(self.channel_id, channel.clone())
                    .await;
                Ok(channel)
            }
        }
    }

    /// Fetches the channel list from the Discord API and updates the cache.
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
