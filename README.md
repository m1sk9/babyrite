# babyrite

[![CI](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml)
[![Release babyrite](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml)
[![MIT license](https://img.shields.io/github/license/henrygd/beszel?color=%239944ee)](https://github.com/m1sk9/babyrite/blob/main/LICENSE)

**babyrite** is a lightweight, fast citation message Discord bot.

```shell
# Latest Release
docker pull ghcr.io/m1sk9/babyrite:latest

# Minor Release
docker pull ghcr.io/m1sk9/babyrite:v0

# Specific Release
docker pull ghcr.io/m1sk9/babyrite:v0.16.0
```

[_API Support: requires Discord API v10_](https://discord.com/developers/docs/reference#api-versioning)

## Features

- **Lightweight**: babyrite uses [distroless](https://github.com/GoogleContainerTools/distroless) as its base image and
  consists of a very lightweight Docker Image.
- **Fast**: babyrite is developed in Rust and is very fast!
- **OSS**: babyrite is open-source and licensed under the MIT License. It also provides a way to host it yourself as
  well as publish it as OSS.
- **Easy to Use**: babyrite is very easy to use and can be deployed in seconds.

## Installation

You can install babyrite using Docker. The following command will pull the latest version of babyrite.

```shell
docker pull ghcr.io/m1sk9/babyrite:v0
```

- babyrite is tested on macOS and Linux (major distributions) as recommended environment.
  - Windows is deprecated as it has not been tested.
- ARM64 environments are supported (v0.16.0+)

### Using Docker Compose

It is recommended to use Docker Compose when setting up babyrite. Direct startup using Docker images or binary files is also possible but not recommended.

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v0
    env_file:
      - .env
    restart: always
```

If you are using orchestration tools such as k8s or Docker Swarm, please configure them according to their respective configuration files.

## Configuration

You can customize the behavior of babyrite by using a dedicated configuration file. The settings are written in TOML format. Refer to [`config/config.toml`](./config/config.toml) for configuration items.

You can also start with the default settings without configuring. The following are the default settings used in that case.

```toml
feature_flag = ""
is_mention = true
is_deletable = true
is_allow_nsfw = false
```

| Key             | Description                                                                    | Default Value |
| --------------- | ------------------------------------------------------------------------------ | ------------- |
| `feature_flag`  | Flags to change the behavior of Babyrite. Specify them separated by commas.    | `""` (empty)  |
| `is_mention`    | Specifies whether to mention the request sender when generating a preview.     | `true`        |
| `is_deletable`  | Sets whether to enable the deletion of previews.                               | `true`        |
| `is_allow_nsfw` | Sets whether to allow the generation of messages from channels marked as NSFW. | `false`       |

## Environment Variables

The environment variables used by babyrite are as follows. Note that the only environment variable required for startup is `DISCORD_API_TOKEN`.

| Key                 | Description                                     |
| ------------------- | ----------------------------------------------- |
| `DISCORD_API_TOKEN` | Discord API token                               |
| `CONFIG_FILE`       | Path to the configuration file (recursive path) |

## LICENSE

babyrite is published under [MIT License](./LICENSE).

<sub>
    ® 2023 - 2025 m1sk9
    <br/>
    babyrite is not affiliated with Discord.
</sub>
