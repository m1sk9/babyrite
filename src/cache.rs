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
/// 500 entries, TTL 1 hour, TTI 1 hour.
///
/// The time-to-live is not just a memory bound: a cached [`GuildChannel`] carries
/// the permission overwrites that decide whether a link may be expanded, so a
/// stale entry keeps authorizing against permissions that no longer exist.
/// [`invalidate_channel`] drops entries as soon as Discord reports a change, and
/// this bounds how long a change that never reached us — a missed event across a
/// gateway session it could not resume — can stay in effect.
fn channel_cache<K, V>(name: &str) -> Cache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    CacheBuilder::new(500)
        .name(name)
        .time_to_idle(std::time::Duration::from_secs(3600))
        .time_to_live(std::time::Duration::from_secs(3600))
        .build()
}

/// Cache for guild channel lists, mapping guild IDs to their channel lists.
pub static GUILD_CHANNEL_LIST_CACHE: LazyLock<Cache<GuildId, HashMap<ChannelId, GuildChannel>>> =
    LazyLock::new(|| channel_cache("guild_channel_list_cache"));

/// Cache for individual guild channels, mapping channel IDs to their channel data.
pub static GUILD_CHANNEL_CACHE: LazyLock<Cache<ChannelId, GuildChannel>> =
    LazyLock::new(|| channel_cache("guild_channel_cache"));

/// Returns `true` when a channel resolved from cache is really in `guild_id`.
fn belongs_to_guild(channel: &GuildChannel, guild_id: GuildId) -> bool {
    channel.guild_id == guild_id
}

/// Drops every cached view of `channel_id`.
///
/// Both caches hold permission overwrites, and those decide whether a linked
/// channel may be expanded. Serving them after Discord has changed them means
/// authorizing against permissions that no longer exist — a channel made private
/// would keep being treated as public. Callers invoke this from the gateway
/// events that report such a change.
///
/// The whole guild's channel list goes too, not just the one entry: the list is a
/// single cached value holding every channel's overwrites, so there is no way to
/// replace one member of it.
pub async fn invalidate_channel(guild_id: GuildId, channel_id: ChannelId) {
    GUILD_CHANNEL_CACHE.invalidate(&channel_id).await;
    GUILD_CHANNEL_LIST_CACHE.invalidate(&guild_id).await;
    tracing::debug!(%guild_id, %channel_id, "invalidated channel caches");
}

impl CacheArgs {
    /// Retrieves a guild channel from cache or fetches it from the API.
    ///
    /// The returned channel is always in [`Self::guild_id`]. [`GUILD_CHANNEL_CACHE`]
    /// is keyed by channel id alone, so a hit is verified against the requested
    /// guild before it is returned; the remaining steps are already scoped to the
    /// guild.
    ///
    /// The result is only as fresh as the cache. [`invalidate_channel`] drops
    /// entries when Discord reports a change and nothing survives the hour
    /// regardless, but permission overwrites read from here can still lag a change
    /// whose event never reached the bot.
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
            // Channel ids are globally unique snowflakes, so a hit for another
            // guild is not a stale entry to refresh — the caller asked for a
            // channel that is not in the guild it named. Refuse rather than
            // return it: the result feeds visibility checks whose role data is
            // guild-local and would silently compare against the wrong guild.
            if !belongs_to_guild(&channel, self.guild_id) {
                tracing::warn!(
                    cached_guild_id = %channel.guild_id,
                    "channel cache hit belongs to another guild"
                );
                anyhow::bail!("Channel does not belong to the requested guild");
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    // `GuildChannel` is `#[non_exhaustive]`, so it cannot be built with a struct
    // literal outside serenity — hence `default()` plus assignment.
    fn channel_in(guild_id: GuildId) -> GuildChannel {
        let mut channel = GuildChannel::default();
        channel.guild_id = guild_id;
        channel
    }

    #[test]
    fn channel_from_the_requested_guild_is_accepted() {
        let guild = GuildId::new(1);
        assert!(belongs_to_guild(&channel_in(guild), guild));
    }

    #[test]
    fn channel_from_another_guild_is_rejected() {
        assert!(!belongs_to_guild(
            &channel_in(GuildId::new(2)),
            GuildId::new(1)
        ));
    }

    // The caches are process-wide statics, so each async test claims ids of its
    // own rather than relying on ordering or isolation between tests.

    #[tokio::test]
    async fn invalidating_drops_the_individual_channel() {
        let guild = GuildId::new(900);
        let channel = ChannelId::new(901);
        GUILD_CHANNEL_CACHE.insert(channel, channel_in(guild)).await;

        invalidate_channel(guild, channel).await;

        assert!(GUILD_CHANNEL_CACHE.get(&channel).await.is_none());
    }

    #[tokio::test]
    async fn invalidating_drops_the_whole_guild_channel_list() {
        let guild = GuildId::new(910);
        let channel = ChannelId::new(911);
        let other = ChannelId::new(912);
        // The list is one cached value covering every channel in the guild, so a
        // change to one of them has to discard all of it.
        let mut list = HashMap::new();
        list.insert(channel, channel_in(guild));
        list.insert(other, channel_in(guild));
        GUILD_CHANNEL_LIST_CACHE.insert(guild, list).await;

        invalidate_channel(guild, channel).await;

        assert!(GUILD_CHANNEL_LIST_CACHE.get(&guild).await.is_none());
    }

    #[tokio::test]
    async fn invalidating_leaves_other_guilds_alone() {
        let target = GuildId::new(920);
        let bystander = GuildId::new(921);
        let bystander_channel = ChannelId::new(922);
        GUILD_CHANNEL_LIST_CACHE
            .insert(bystander, HashMap::new())
            .await;
        GUILD_CHANNEL_CACHE
            .insert(bystander_channel, channel_in(bystander))
            .await;

        invalidate_channel(target, ChannelId::new(923)).await;

        assert!(GUILD_CHANNEL_LIST_CACHE.get(&bystander).await.is_some());
        assert!(GUILD_CHANNEL_CACHE.get(&bystander_channel).await.is_some());
    }
}
