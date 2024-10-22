---
title: FAQ
editLink: true
---

# FAQ

## Can I operate with [Sharding](https://discord.com/developers/docs/topics/gateway#sharding)?

**No.** babyrite is not designed to support multiple processes. Therefore, it is
not recommended to operate with more than 2500 guilds.

If you wish to operate in such a manner, please either create your own bot or
use a publicly available bot instead of using babyrite.

## Why is Windows not supported?

babyrite is tested for compilation and execution on macOS
(`aarch64-apple-darwin`), which is used by the developers, and on Debian
(`x86_64-unknown-linux-gnu`), which is used in the Docker Image.

Windows is not tested due to reasons such as the long execution time on GitHub
Actions. There is no point in supporting it, so there are no plans to do so in
the future.

## Can Babyrite be used with Shuttle?

**No.** There are no plans to release a compatible version.
