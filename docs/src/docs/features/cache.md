---
layout: doc
---

# Caching <Badge type="tip" text="v0.3.0~" />

babyrite uses the concurrent caching library [moka](https://github.com/moka-rs/moka) to cache channel information retrieved from the Discord API, cutting down on the need to refetch it.

::: warning About the caching system's technical constraints

- There is no explicit cache invalidation mechanism.
  - Even after a channel is renamed, deleted, or has its permissions changed, stale data may still be returned until the TTI/TTL expires.
- Cache keys are shared across the entire process.
  - `GUILD_CHANNEL_CACHE` is keyed by `channel_id` alone, so a channel cached for another guild can still be a hit.
  - Every lookup therefore verifies that the channel really belongs to the requested guild. If it does not, the channel is not returned and the lookup fails.

:::

## Supported features

- [Message Preview](./citation.md)

GitHub permalinks are not covered — babyrite fetches directly from `raw.githubusercontent.com` every time.

## Cache layout

babyrite builds the following cache layout in the machine's memory.

| Cache | Key → Value | Purpose |
| ---- | ---- | ---- |
| `GUILD_CHANNEL_LIST_CACHE` | `GuildId` → `HashMap<ChannelId, GuildChannel>` | Per-guild channel list |
| `GUILD_CHANNEL_CACHE` | `ChannelId` → `GuildChannel` | Individual channels |

Both operate with the following shared settings:

- Up to 500 entries
  - The cache is organized on a Least Recently Used (LRU) basis.
  - Entries beyond the size limit are evicted starting with the least recently used data.
- TTI (Time To Idle):
  - Data that hasn't been accessed for 1 hour is automatically removed.
- TTL (Time To Live):
  - Data expires after 12 hours regardless of access frequency.

## Cache lookup flow

### Message preview

1. Look up `GUILD_CHANNEL_CACHE` (the individual channel cache)
    - On a hit, return the channel if it belongs to the requested guild.
    - If it belongs to another guild, fail. (Channel IDs are unique across Discord, so this means the request itself was wrong.)
2. If not found, look up `GUILD_CHANNEL_LIST_CACHE` (the channel list cache)
    - If it's a hit, look for the target channel in that map.
    - If it's a miss, fetch from the Discord API and store the result in the cache.
3. If the target channel isn't found in the channel list, search active threads (threads aren't included in the channel list).
4. The channel that's found is ultimately written into the cache as well.
