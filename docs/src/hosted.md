# Hosted babyrite

<!-- toc -->

## Overview

- There is no official host for babyrite (in most cases you host it yourself).
- It's a hassle to host it yourself. If you don't know how to do it, we recommend you to use a general citation bot released by other developers (some of them are not OSS and are released as a paid service).
    - babyrite is just one of the major options for those who want to host their own, since it does not require huge resources and there are no useless features.

## How to host babyrite

Hosting a Discord Bot requires a server that can run 24 hours a day. There are many ways to host a Discord Bot, but the most common way is to use a VPS or a cloud service.

### Using Docker Compose

The basic hosting method is Docker Compose, which is relatively easy to use.

Learn more about [Using Docker Compose](installation.md#using-docker-compose).

### Using Kubernetes (k8s)

If you are familiar with Kubernetes, you can use it to host babyrite.

Learn more about [Kubernetes](https://kubernetes.io/docs/home/)

## System requirements

babyrite is a dynamically linked binary compiled with Rust. Unlike discord.js and JDA created bots, it does not require a runtime.

- 1 CPU core
- 512MB RAM

Supports only **Linux and macOS**. [(Windows is not **support**ed)](./faq.md#why-is-windows-not-supported)

> [!NOTE]
> 
> If you need static binaries, you must [compile from source yourself](installation.md#using-binary-build-from-source). Basically, Rust compiles dynamic binaries by default because the standard libraries (e.g. `libc`) depend on the system's shared libraries. If you want to compile static binaries, use `musl`.
>
> ```shell
> rustup target add x86_64-unknown-linux-musl
> cargo build --target x86_64-unknown-linux-musl --release
> ```

## Use hosted babyrite on dev-m1sk9-ubuntu-001

> [!IMPORTANT]
> 
> **Not an official host!**

My friends can use the babyrite I host on their Discord server. If you want to use it, please contact me.

- babyrite always uses the latest version.
  - You can check the version by the activity on Discord or [m1sk9/bot-containers](https://github.com/m1sk9/bot-containers).
- The channel cache is shared by all guilds, so channels that have not been accessed for a while are deleted from the cache.
- `dev-m1sk9-ubuntu-001` is an Ubuntu server hosted at home, and is subject to service outages without notice. Please check [m1sk9's Mastodon account (`mstdn.maud.io/@m1sk9`)](https://mstdn.maud.io/@m1sk9) for the latest information.
