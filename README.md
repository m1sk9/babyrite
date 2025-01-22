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

- **Lightweight**: babyrite uses [distroless](https://github.com/GoogleContainerTools/distroless) as its base image and consists of a very lightweight Docker Image.
- **Fast**: babyrite is developed in Rust and is very fast!
- **OSS**: babyrite is open-source and licensed under the MIT License. It also provides a way to host it yourself as well as publish it as OSS.
- **Easy to Use**: babyrite is very easy to use and can be deployed in seconds.

## Usage

- Send a message link in a channel where babyrite can view it, and it will send a preview of the message's content
- babyrite can also show attached images and GIF files as preview. These are sent through Discord's CDN and are never stored on the server side.

## LICENSE

babyrite is published under [MIT License](./LICENSE).

<sub>
    ® 2023 - 2025 m1sk9
    <br/>
    babyrite is not affiliated with Discord.
</sub>
