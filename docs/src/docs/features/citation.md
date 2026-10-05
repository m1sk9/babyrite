---
layout: doc
---

# Message Preview <Badge type="tip" text="v0.1.0~" />

When you send a message link, babyrite expands that message's content as embedded content.

```md
https://discord.com/channels/1390203929182339092/1496887099536838706/1497066171390886000
```

Follow these steps in your client to copy a message link.

::: tabs
== Desktop / Browser

1. Right-click the message
2. Click **Copy Message Link** from the context menu

![](/features/citation/citation-desktop.gif)

== Mobile / Tablet

1. Long-press the message
2. Tap **Copy Message Link** from the menu

![](/features/citation/citation-mobile.gif)

:::

- Expands up to 3 links per message.
- Content in NSFW channels is not expanded. The same applies to threads under an NSFW channel.

### Canceling a preview

If you don't want a link expanded, wrap the message link in `<>` and it will be ignored.

```md
<https://discord.com/channels/1390203929182339092/1496887099536838706/1497066171390886000>
```

## Behavior with channel permissions

babyrite behaves as follows with respect to channel permissions.

- Message links pointing to another server (guild) are not expanded.
  - Permissions are defined per server, so babyrite cannot judge whether a channel in another server is viewable.
- Messages in private threads and DMs are not expanded.
- The linked message is only expanded if every member who can view the source channel can also read it.
  - Reading the linked message requires both "View Channel" and "Read Message History" in its channel.
  - In voice and stage channels, "Connect" is required as well.
  - Permissions are evaluated the way Discord does, including members who hold several roles at once.
  - It is not expanded if even one member could view the source channel but not read the linked message.
- Public threads are judged based on their parent channel's permissions.
- Members granted or denied access individually are looked up and judged on their actual roles.
  - If more than 10 such members would need to be looked up for one link, it is not expanded, as a safety measure.
- If the target is the same channel as the source, the permission checks above are skipped and the link is expanded as-is.

## Supported message links

The message links babyrite currently supports are:

- `discord.com`
- `canary.discord.com`
- `ptb.discord.com`

Any other message link is ignored for safety reasons.

::: warning About the domain used by some Windows clients

The `discordapp.com` domain, used by some Windows clients, is a legacy format. Support for this domain was dropped in a past version.

See the following issue for details.

[Support discordapp.com - m1sk9/babyrite #172](https://github.com/m1sk9/babyrite/issues/172)

:::

## Caching

When babyrite expands a message link, it caches the guild's channel list and the source channel of the message.

From the second preview of a message in the same channel onward, the channel is retrieved from the cache instead of querying the Discord API.

See [Caching](./cache.md) for details.
