---
layout: doc
---

# Getting Started

This page explains how to set up babyrite.

## Requirements

| Item | Minimum | Recommended (multi-guild) |
| --- | --- | --- |
| CPU | 1 vCPU (shared is fine) | 1 vCPU |
| Memory | 128MB | 256MB or more |
| Disk | Tens of MB (`config.toml` only) | Same |
| Network | HTTPS outbound to the Discord Gateway (persistent WebSocket) and the Discord API / GitHub API | Same |

- babyrite has no external database, and there are no plans to add a feature that would require one.
- When it connects to the Discord API over WebSocket, babyrite caches channel information in memory using [moka](https://github.com/moka-rs/moka).
  - The cache stores up to 500 entries each for a guild's channel list and individual channels.
  - It's configured with a 1h TTL and a 1h TTI.

::: tip These are implementation-derived estimates, not measured benchmarks
The figures above are estimated from the source code (cache size, the 1MB limit on GitHub raw fetches, etc.), not measured benchmarks. Adjust them to fit your deployment scale.
:::

### Supported OS / Architecture

- Verified to run on macOS and major Linux distributions, which are the recommended environments.
  - It also runs on Windows, but macOS or Linux is recommended (since v0.19.0).
- ARM64 environments are supported (since v0.16.0).

### Trade-offs

babyrite's own processing is centered on regex matching and HTTP requests, so it rarely pins the CPU. That said, the following factors scale with the number of guilds and message volume, so running at the "minimum" spec across multiple guilds can lead to memory pressure or increased latency.

- **moka cache memory usage**: the more guilds and channels babyrite participates in, the more `GuildChannel` data the cache holds. The cache itself is capped at 500 entries, but the data size per entry depends on each guild's configuration.
- **reqwest connection count**: when Discord message links and GitHub permalinks are expanded concurrently (up to 3 per message each), outbound HTTP connections temporarily increase.
- **GitHub raw fetch buffer**: up to 1MB per file is temporarily loaded into memory, so momentary memory usage can spike when multiple GitHub permalinks are expanded at the same time.

The minimum spec is sufficient for small, single-guild deployments, but if you're running across multiple guilds continuously, it's recommended to leave some headroom at around the recommended spec.

## Installation

babyrite is installed using Docker. You can pull the latest version with the following command.

```shell
docker pull ghcr.io/m1sk9/babyrite:v1
```

### Using Docker Compose (recommended)

We recommend using Docker Compose to set up babyrite.

If you're using an orchestration tool like k8s or Docker Swarm, configure it to match that tool's own configuration files.

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v1
    env_file:
      - .env
    restart: always
```

## Configuration

You can customize babyrite's behavior with a dedicated configuration file. Write it in TOML format and point to it with the `CONFIG_FILE_PATH` environment variable. babyrite can also start with the default configuration if no configuration file is provided.

See the [Configuration Reference](./configuration) for the full list of settings, their defaults, and logging output details.

## Environment Variables

The environment variables babyrite uses are as follows. `DISCORD_API_TOKEN` is the only one required to start.

| Key | Description |
| --- | --- |
| `DISCORD_API_TOKEN` | Discord API token |
| `CONFIG_FILE_PATH` | Path to the configuration file (recursive path) |
| `RUST_LOG` | Log level filter. When set, it overrides the configuration file's `log.level`. |
