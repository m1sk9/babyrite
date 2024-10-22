---
title: Cache
editLink: true
---

# Cache

> [!NOTE]
>
> The channel cache feature is available from
> [v0.3.0](https://github.com/m1sk9/babyrite/releases/tag/v0.3.0)

babyrite caches the channel of the message source when a message link is quoted.

If you quote a message from the same channel for the second time or later, the
channel is retrieved from the cache without querying the Discord API.

## Cache Expiration

Entries that have not been accessed or updated for 1 hour are automatically
deleted.
