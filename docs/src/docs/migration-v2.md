---
layout: doc
---

# Migrating to v2

babyrite v2.0.0 has been released. This guide helps you migrate from v1.

## Switching the underlying library

In v2.0.0, the Discord API wrapper babyrite has used since its first version was changed from [serenity](https://github.com/serenity-rs/serenity) to [twilight](https://github.com/twilight-rs/twilight).

Serenity's latest release, v0.12.5, carries a vulnerability (unbounded empty DATA frames in the `h2` crate), but no fix has been released and maintenance has stalled for about 10 months.

For a project like babyrite that ships releases frequently, continuing to depend on it is a risk, so we decided to switch.

## Behavior

Most of babyrite's features are unchanged, and no configuration values have changed. You can update as-is.

Note, however, that some behavior has changed along with the library. Keep the following in mind when operating babyrite.

### What changes

1. **Discord API calls have a 10 second timeout**
2. **Reconnects use exponential backoff**
   - As a result, recovery from a long outage may be delayed by up to about 4 minutes

### What stays the same

- Features
- Quoting and other behavior
- Shard configuration
- The configuration, environment variables and permissions required to start

## Migration steps

You can update to babyrite v2 as-is, but some environments may need a few steps.

### Docker image tag

This is a major update, so if you pin the version, change it from `v1` to `v2`.

```diff
services:
  app:
-   image: ghcr.io/m1sk9/babyrite:v1
+   image: ghcr.io/m1sk9/babyrite:v2
    env_file:
      - .env
    volumes:
      - ./config/config.toml:/config/config.toml
    restart: always
```

### End of Intel Mac support (macOS only)

Support for Intel Macs (`x86_64-apple-darwin`) ended in [v1.4.7](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v1.4.7).

No image is published for v2.0.0, so it cannot be used. To keep using babyrite, you need to [build it locally](./build-from-source) (note that this is not supported).
