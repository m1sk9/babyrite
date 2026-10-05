---
layout: doc
---

# Getting Started

This page explains how to set up babyrite.

::: tip Before you begin
babyrite needs a Discord bot token to start. If you have not created a bot yet, complete [Setting Up the Discord Bot](./discord-bot) first.
:::

## Requirements

| Item    | Minimum                                                                                       | Recommended (multi-guild) |
| ------- | --------------------------------------------------------------------------------------------- | ------------------------- |
| CPU     | 1 vCPU (shared is fine)                                                                       | 1 vCPU                    |
| Memory  | 128MB                                                                                         | 256MB or more             |
| Disk    | Tens of MB (`config.toml` only)                                                               | Same                      |
| Network | HTTPS outbound to the Discord Gateway (persistent WebSocket) and the Discord API / GitHub API | Same                      |

- babyrite has no external database, and there are no plans to add a feature that would require one.
- When it connects to the Discord API over WebSocket, babyrite caches channel and role information in memory using [moka](https://github.com/moka-rs/moka).
  - The cache stores up to 500 entries each for a guild's channel list, individual channels and role permissions.
  - It's configured with a 1h TTL and a 1h TTI.

::: tip These are implementation-derived estimates, not measured benchmarks
The figures above are estimated from the source code (cache size, the 1MB limit on GitHub raw fetches, etc.), not measured benchmarks. Adjust them to fit your deployment scale.
:::

### Supported OS / Architecture

- Verified to run on macOS and major Linux distributions, which are the recommended environments.
  - It also runs on Windows, but macOS or Linux is recommended (since v0.19.0).
  - Starting with macOS 28, Apple is ending support for Intel-based applications, [so support has also been discontinued in babyrite](https://github.com/m1sk9/babyrite/discussions/701) (v1.4.7+).
- ARM64 environments are supported (since v0.16.0).

### Trade-offs

babyrite's own processing is centered on regex matching and HTTP requests, so it rarely pins the CPU. That said, the following factors scale with the number of guilds and message volume, so running at the "minimum" spec across multiple guilds can lead to memory pressure or increased latency.

- **moka cache memory usage**: the more guilds and channels babyrite participates in, the more channel and role data the cache holds. The cache itself is capped at 500 entries, but the data size per entry depends on each guild's configuration.
- **HTTP connection count**: when Discord message links and GitHub permalinks are expanded concurrently (up to 3 per message each), outbound HTTP connections temporarily increase.
- **GitHub raw fetch buffer**: up to 1MB per file is temporarily loaded into memory, so momentary memory usage can spike when multiple GitHub permalinks are expanded at the same time.

The minimum spec is sufficient for small, single-guild deployments, but if you're running across multiple guilds continuously, it's recommended to leave some headroom at around the recommended spec.

## Installation

babyrite is installed using Docker. You can pull the latest version with the following command.

```shell
docker pull ghcr.io/m1sk9/babyrite:v2
```

### Image tags

Choose a tag that fits your use.

| Tag | Description |
| --- | --- |
| `latest` | The latest release. Major updates are picked up automatically as well. |
| `v2` | The latest v2 release. Major updates with breaking changes are not picked up. (Recommended) |
| `v2.0.0` | Pins a specific release. |

For a major update, read the migration guide (such as the [v2 Migration Guide](./migration-v2)) before updating the tag.

### Using Docker Compose (recommended)

We recommend using Docker Compose to set up babyrite.

If you're using an orchestration tool like k8s or Docker Swarm, configure it to match that tool's own configuration files.

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v2
    env_file:
      - .env
    volumes:
      - ./config/config.toml:/config/config.toml
    restart: always
```

Write the environment variables in `.env`. Set `CONFIG_FILE_PATH` to the path inside the container.

```shell
DISCORD_API_TOKEN=your-token
CONFIG_FILE_PATH=/config/config.toml
```

If you don't use a configuration file, you can omit `volumes` and `CONFIG_FILE_PATH`.

## Configuration

You can customize babyrite's behavior with a dedicated configuration file. Write it in TOML format and point to it with the `CONFIG_FILE_PATH` environment variable. babyrite can also start with the default configuration if no configuration file is provided.

A sample listing every setting is available at [`config/config.toml`](https://github.com/m1sk9/babyrite/blob/main/config/config.toml) in the repository.

See the [Configuration Reference](./configuration) for the full list of settings, their defaults, and logging output details.

## Environment Variables

The environment variables babyrite uses are as follows. `DISCORD_API_TOKEN` is the only one required to start.

| Key                 | Description                                                                    |
| ------------------- | ------------------------------------------------------------------------------ |
| `DISCORD_API_TOKEN` | Discord API token                                                              |
| `CONFIG_FILE_PATH`  | Path to the configuration file (recursive path)                                |
| `RUST_LOG`          | Log level filter. When set, it overrides the configuration file's `log.level`. |
