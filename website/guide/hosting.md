---
title: Hosting
editLink: true
---

# Hosting

## Overview

- There is no official host for babyrite (in most cases, you will host it
  yourself).
- Hosting it yourself can be cumbersome. If you don't know how, it is
  recommended to use a general-purpose quote bot published by other developers
  (some of which are not OSS and are available as paid services).
  - babyrite does not require massive resources and has no unnecessary features,
    making it just one of the main options for those who want to host it
    themselves.

## How to Host babyrite

You need a server that runs 24/7. The most common way is to use a VPS or cloud
service.

### Using Docker Compose

The basic hosting method is Docker Compose, which is relatively easy to use.

- For Docker Compose, see [here](https://docs.docker.com/compose/install/)
- For hosting with Docker Compose, see
  [here](../getting-started.md#getting-started)

### Using Kubernetes (k8s)

If you are familiar with Kubernetes, you can use Kubernetes to host babyrite.

- For Kubernetes, see [here](https://kubernetes.io/docs/home/)

## System Requirements

babyrite is a dynamically linked binary compiled in Rust. Unlike bots created
with discord.js or JDA, it does not require a runtime.

- 1 CPU Core
- 512MB RAM

Support is **Linux and macOS** only.

> [!TIP]
>
> If you need a static binary,
> [compile from source](../getting-started.md#using-the-test-version) yourself.
>
> Rust compiles dynamic binaries by default because the standard library (such
> as `libc`) depends on the system's shared libraries.
>
> To compile a static binary, use `musl`.
>
> ```sh
> rustup target add x86_64-unknown-linux-musl
> cargo build --target x86_64-unknown-linux-musl --release
> ```

## Using babyrite hosted on dev-m1sk9-s1

> [!IMPORTANT]
>
> This is not an official host. This service may be stopped without notice.

If you are an acquaintance of mine, you can use the babyrite I host myself on
your Discord server. Please contact me for more details.

- babyrite always uses the latest version.
- Channel cache is shared across all guilds, so channels that have not been
  accessed for a while will be removed from the cache.
- `dev-m1sk9-001` is an Ubuntu server hosted at home, and the service may be
  stopped or maintained without notice. For the latest information, please check
  [m1sk9's Mastodon account (`mstdn.maud.io/@m1sk9`)](https://mstdn.maud.io/@m1sk9).
