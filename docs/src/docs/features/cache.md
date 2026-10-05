---
layout: doc
---

# Caching <Badge type="tip" text="v0.3.0~" />

babyrite uses the concurrent caching library [moka](https://github.com/moka-rs/moka) to cache channel and role information retrieved from the Discord API, cutting down on the need to refetch it.

::: warning About the caching system's technical constraints

- When Discord reports that a channel was created, updated or deleted (threads included), the cache for that channel and its guild is discarded.
  - Making a channel private therefore takes effect immediately.
  - An event missed across a gateway reconnect is the exception; the TTL resolves that within the hour.
- When Discord reports that a role was created, updated or deleted, the role cache for that guild is discarded. It is also discarded when the guild is (re)announced with `GUILD_CREATE` on connect.
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
| `GUILD_CHANNEL_LIST_CACHE` | Guild ID → channel map | Per-guild channel list |
| `GUILD_CHANNEL_CACHE` | Channel ID → channel | Individual channels |
| `GUILD_ROLE_CACHE` | Guild ID → role permissions | Per-guild role permissions used for visibility checks |

All operate with the following shared settings:

- Up to 500 entries
  - The cache is organized on a Least Recently Used (LRU) basis.
  - Entries beyond the size limit are evicted starting with the least recently used data.
- TTI (Time To Idle):
  - Data that hasn't been accessed for 1 hour is automatically removed.
- TTL (Time To Live):
  - Data expires after 1 hour regardless of access frequency.
  - Cached channel and role data feeds permission decisions, so the TTL is not just a memory bound — it also bounds how long a missed event can stay in effect.

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
5. When a visibility check is needed, the guild's role permissions are taken from `GUILD_ROLE_CACHE` the same way (fetched from the Discord API on a miss).
6. If some members are granted or denied access individually, their information is fetched from the Discord API every time.
    - Member role changes cannot be observed without the privileged `GUILD_MEMBERS` intent, so member information is not cached.
