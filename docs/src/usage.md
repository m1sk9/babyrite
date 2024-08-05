# Usage

<!-- toc -->

## Quoting a Message

When you send a message link to a channel, babyrite will fetch and display the message.

babyrite can quote the following messages:

- Messages from channels that babyrite has access to
- Messages from channels with NSFW settings turned off
- Messages that contain embeds

> [!IMPORTANT]
>
> To prevent spamming, babyrite will quote only the first message link.

## Skipping a Message Quote

By default, babyrite will quote all messages. If there is a message you don't want to be quoted, enclose the link in `<>`.

```markdown
<https://discord.com/channels/1269163236327030815/1269163666528407603/1269164052072890512>
```

## Supported domains

babyrite supports message links at the following domains:

- `discord.com`
- `canary.discord.com`
- `ptb.discord.com`

These are the domains used for the **Official Build (Mobile), Canary Build, PTB Build (Public Test Build), and Development Build**.

> [!IMPORTANT]
>
> Some Windows builds (32bit?) seem to use `discordapp.com` as domain, but babyrite does not plan to support this domain. Please check [m1sk9/babyrite#172](https://github.com/m1sk9/babyrite/issues/172) for details.

## Channel Cache

> [!NOTE]
>
> Channel cache is available from [v0.3.0](https://github.com/m1sk9/babyrite/releases/tag/v0.3.0) ([v0.3.1](https://github.com/m1sk9/babyrite/releases/tag/v0.3.1)) onwards.

babyrite saves the channels of quoted messages as a cache. When there is another request to quote a message from the same channel, it uses the cached channel information to fetch the message.

By using the cache, babyrite does not use the Discord API to fetch the channel, making the message quoting faster.

If there is no cached channel, it fetches the channel using the Discord API (the method before v0.2.1). If there are many channels, quoting might take some time.

### Cache Expiration

If a cached entry is not accessed for 30 minutes, the cache is automatically cleared.

> [!IMPORTANT]
>
> To forcefully clear the cache, restart the active babyrite process.(For Docker Image, you can restart with docker restart.)
>
> However, this operation should generally not be necessary.
