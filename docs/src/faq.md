# FAQ

<!-- toc -->

### Can babyrite be operated with [Sharding](https://discord.com/developers/docs/topics/gateway#sharding)?

No, it cannot. babyrite does not support operation with multiple processes using Sharding. Therefore, it is not recommended to operate in more than 2500 guilds.

If you wish to operate in this manner, you should either create your own bot instead of using babyrite, or use a publicly available bot.

### Why is Windows not supported?

Strictly speaking, it is untested. babyrite has been tested for compilation and execution on macOS (`aarch64-apple-darwin`), which is used by the developer, and on Debian (`x86_64-unknown-linux-gnu`), used in the Docker Image. These are the supported OS architectures.

Windows is untested due to reasons like excessively long GitHub Actions execution times. Therefore, it is considered an unsupported OS, and there are no plans to support it in the future.

### Can babyrite be used with Shuttle?

No, it cannot be used. There are no plans to release a compatible version.

### Can I quote poll messages?

As of [v0.9.0](./changelog.md#v090), quoting poll messages is not possible because Serenity does not support it.
