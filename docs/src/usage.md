# Usage

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
