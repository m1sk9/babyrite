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

/// Builds a cache with the shared tuning for both channel caches:
/// 500 entries, TTL 12 hours, TTI 1 hour.
fn channel_cache<K, V>(name: &str) -> Cache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    CacheBuilder::new(500)
        .name(name)
        .time_to_idle(std::time::Duration::from_secs(3600))
        .time_to_live(std::time::Duration::from_secs(43200))
        .build()
}

/// Cache for guild channel lists, mapping guild IDs to their channel lists.
pub static GUILD_CHANNEL_LIST_CACHE: LazyLock<Cache<GuildId, HashMap<ChannelId, GuildChannel>>> =
    LazyLock::new(|| channel_cache("guild_channel_list_cache"));

/// Cache for individual guild channels, mapping channel IDs to their channel data.
pub static GUILD_CHANNEL_CACHE: LazyLock<Cache<ChannelId, GuildChannel>> =
    LazyLock::new(|| channel_cache("guild_channel_cache"));

impl CacheArgs {
    /// Retrieves a guild channel from cache or fetches it from the API.
    ///
    /// The lookup order is:
    /// 1. Individual channel cache
    /// 2. Guild channel list cache
    /// 3. Discord API (with cache update)
    #[tracing::instrument(
        skip(self, ctx),
        fields(guild_id = %self.guild_id, channel_id = %self.channel_id)
    )]
    pub async fn get(&self, ctx: &Context) -> anyhow::Result<GuildChannel> {
        if let Some(channel) = GUILD_CHANNEL_CACHE.get(&self.channel_id).await {
            tracing::debug!("channel cache hit");
            return Ok(channel);
        }
        tracing::debug!("channel cache miss");

        // `try_get_with` coalesces concurrent misses for the same guild into a
        // single fetch, so `join_all`-ing several link expansions no longer
        // fires one identical `channels` request per link.
        let channel_list = GUILD_CHANNEL_LIST_CACHE
            .try_get_with(self.guild_id, self.get_channel_list_from_api(ctx))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get channel list: {e}"))?;

        let channel = match channel_list.get(&self.channel_id).cloned() {
            Some(c) => c,
            None => {
                // Not in the channel list — it may be an active thread,
                // which is not returned by the guild channels endpoint.
                tracing::debug!("channel not in list, searching active threads");
                let data = self
                    .guild_id
                    .get_active_threads(&ctx.http)
                    .await
                    .context("Failed to get active threads")?;
                data.threads
                    .iter()
                    .find(|t| t.id == self.channel_id)
                    .cloned()
                    .ok_or_else(|| {
                        tracing::debug!("channel not found in guild or active threads");
                        anyhow::anyhow!("Channel not found in cache")
                    })?
            }
        };

        GUILD_CHANNEL_CACHE
            .insert(self.channel_id, channel.clone())
            .await;
        tracing::trace!("inserted channel into cache");
        Ok(channel)
    }

    /// Fetches the channel list from the Discord API.
    ///
    /// The caller inserts the result into [`GUILD_CHANNEL_LIST_CACHE`] via
    /// `try_get_with`, so this does not touch the cache itself.
    #[tracing::instrument(skip(self, ctx), fields(guild_id = %self.guild_id))]
    async fn get_channel_list_from_api(
        &self,
        ctx: &Context,
    ) -> anyhow::Result<HashMap<ChannelId, GuildChannel>> {
        tracing::debug!("fetching channel list from Discord API");
        let started = std::time::Instant::now();
        let channels = self
            .guild_id
            .channels(&ctx.http)
            .await
            .context("Failed to get channel list")?;

        tracing::debug!(
            channels = channels.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "fetched channel list from Discord API"
        );

        Ok(channels)
    }
}
