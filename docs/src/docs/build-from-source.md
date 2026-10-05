---
layout: doc
---

# Building from Source

This page explains how to build babyrite from source instead of using the published Docker image.

::: warning About support for source builds

Building from source is meant for environments without a published image (such as Intel Macs). babyrite run this way is not supported. Use the Docker image described in [Getting Started](./getting-started) whenever possible.

:::

## Prerequisites

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install) (stable)

::: tip

If you use `rustup`, the required toolchain is installed automatically.

:::

## Build the binary

```shell
git clone https://github.com/m1sk9/babyrite.git
cd babyrite
git checkout babyrite-v2.0.0
cargo build --release
```

Once the build finishes, the binary is generated at `target/release/babyrite`.

Check out a release tag with `git checkout`. The `main` branch may contain unreleased changes. See [Releases](https://github.com/m1sk9/babyrite/releases) for the list of tags.

## Run it

Run the binary with the environment variables set. See [Getting Started](./getting-started#environment-variables) for details on the environment variables.

```shell
DISCORD_API_TOKEN=your-token CONFIG_FILE_PATH=./config/config.toml ./target/release/babyrite
```

If `CONFIG_FILE_PATH` is omitted, babyrite starts with the default configuration.

## Build the Docker image

You can also build a Docker image locally with the same setup as the published one. Run this from the repository root.

```shell
docker build -f docker/Dockerfile -t babyrite:local .
```

To use the image you built, replace `image` in your Docker Compose file with `babyrite:local`.
