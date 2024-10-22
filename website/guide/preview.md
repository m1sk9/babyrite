---
title: Quoting Feature
editLink: true
---

# Quoting Feature

To quote a message, send the message link in a channel where babyrite can view
and send messages.

```md
https://discord.com/channels/123456789012345678/123456789012345678/123456789012345678
```

Messages that meet the following conditions can be quoted:

- The channel is one that babyrite can view.
- The source channel of the message is not set to NSFW (Not Safe For Work).
- The message is not an embed-only message.
- If the message is from a private channel, the channel where the quote is being
  expanded must grant viewing permissions to both babyrite and `@everyone`.

If the message does not meet these conditions, babyrite will ignore it and not
generate a preview.

## Skipping Quotes

If you want to send **only the message link** without quoting, enclose the
message link in `<>`.

```md
<https://discord.com/channels/123456789012345678/123456789012345678/123456789012345678>
```

## What Can Be Generated in the Preview

The following can be generated in the preview by babyrite:

- Content of the message
- Sender of the message
- Date and time the message was sent
- Attachments (images, GIFs)

> [!WARNING]
>
> - If the attachment of the message is invalid for any reason, it will not be
>   displayed in the preview.
> - Attachments displayed in the preview are not stored on the execution server
>   but are retrieved from Discord's CDN.

## Supported Message Links

The message links currently supported by babyrite are as follows:

- `discord.com`
- `canary.discord.com`
- `ptb.discord.com`

Other message links are ignored for security reasons.

> [!IMPORTANT]
>
> `discordapp.com` is not supported. For more details, refer to
> [m1sk9/babyrite#172](https://github.com/m1sk9/babyrite/issues/172).

## Channel Cache

When a message link is quoted, babyrite caches the channel from which the
message was sent.

For subsequent quotes from the same channel, the channel is retrieved from the
cache instead of querying the Discord API.

For more details, refer to [Cache](./cache.md).
