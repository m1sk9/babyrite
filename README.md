# babyrite

[![Release babyrite](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml)
[![Build](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml)

A citation message Discord bot

![babyrite example](docs/src/public/example.gif)

## Installation

Can get babyrite Docker Image from ghcr.io:

```shell
# latest version
docker pull ghcr.io/m1sk9/babyrite:latest

# Minor version (v0.x) - recommended
docker pull ghcr.io/m1sk9/babyrite:v0
```

You can start babyrite by providing environment variables for the Docker Image you have obtained.

See [babyrite docs](. /docs/README.md) for detailed setup instructions.

## Environment Variables

The following are environment variables used by babyrite.

Key whose `Required` column is `Yes` must be specified. If not specified, babyrite will not start correctly.

| Key | Description | Required | Default |
| --- | ----------- | -------- | ------- |
| `DISCORD_API_TOKEN` | Discord API Token | Yes | --- |
