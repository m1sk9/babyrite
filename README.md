# babyrite

[![CI](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml)
[![Release babyrite](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml)
[![Docker Image Size](https://ghcr-badge.egpl.dev/m1sk9/babyrite/size)](https://github.com/m1sk9/babyrite/pkgs/container/babyrite/versions)
[![MIT license](https://img.shields.io/github/license/henrygd/beszel?color=%239944ee)](https://github.com/m1sk9/babyrite/blob/main/LICENSE)

**babyrite** is a lightweight, fast citation message Discord bot.

```shell
# Latest Release
docker pull ghcr.io/m1sk9/babyrite:latest

# Minor Release
docker pull ghcr.io/m1sk9/babyrite:v0

# Specific Release
docker pull ghcr.io/m1sk9/babyrite:v0.14.0
```

[*API Support: requires Discord API v10*](https://discord.com/developers/docs/reference#api-versioning)

## Features

- **Lightweight**: babyrite uses [distroless](https://github.com/GoogleContainerTools/distroless) as its base image and
  consists of a very lightweight Docker Image.
- **Fast**: babyrite is developed in Rust and is very fast!
- **OSS**: babyrite is open-source and licensed under the MIT License. It also provides a way to host it yourself as
  well as publish it as OSS.
- **Easy to Use**: babyrite is very easy to use and can be deployed in seconds.

## Installation

It is recommended to use Docker Compose when setting up babyrite. Direct startup using Docker images or binary files is also possible but not recommended.
```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v0.14.0
    env_file:
      - .env
    restart: always
```

If you are using orchestration tools such as k8s or Docker Swarm, please configure them according to their respective configuration files.

## Configuration and Environment Variables

You can customize the behavior of babyrite by using a dedicated configuration file. The settings are written in TOML format. Refer to [`config/config.toml`](./config/config.toml) for configuration items.

You can also start with the default settings without configuring. The following are the default settings used in that case.

```toml
feature_flag = ""
is_mention = true
is_deletable = true
is_allow_nsfw = false
```

The environment variables used by babyrite are as follows. Note that the only environment variable required for startup is DISCORD_API_TOKEN.

| Key                 | Description                                     | 
|---------------------|-------------------------------------------------|
| `DISCORD_API_TOKEN` | Discord API token                               | 
| `CONFIG_FILE`       | Path to the configuration file (recursive path) | 

## LICENSE

babyrite is published under [MIT License](./LICENSE).

<sub>
    ® 2023 - 2025 m1sk9
    <br/>
    babyrite is not affiliated with Discord.
</sub>
