---
layout: doc
---

# Setting Up the Discord Bot

Before starting babyrite, you need to create a bot in the Discord Developer Portal and invite it to your server.

## Create the bot

1. Open the [Discord Developer Portal](https://discord.com/developers/applications) and create an application with **New Application**
2. Open **Bot** in the left menu
3. Click **Reset Token** to issue a token and keep it somewhere safe
   - Set this token as the `DISCORD_API_TOKEN` environment variable.
   - The token cannot be shown again. If you lose it, reset it.

::: danger Handling the token
Anyone who knows the token can control the bot. Never commit it to a repository or paste it anywhere public. If it leaks, reset it immediately.
:::

## Enable the Message Content Intent

babyrite detects links in message content, so it requires the **Message Content Intent**, a privileged intent.

1. Open **Privileged Gateway Intents** on the **Bot** page
2. Enable **Message Content Intent** and save

::: warning babyrite will not start without it
If the Message Content Intent is disabled, Discord refuses the connection and babyrite exits with an error.
:::

babyrite requests the following three intents. The Message Content Intent is the only privileged one.

| Intent | Purpose |
| --- | --- |
| `GUILDS` | Receive channel, thread and role changes to discard the cache |
| `GUILD_MESSAGES` | Receive posted messages |
| `MESSAGE_CONTENT` | Detect links in message content |

## Invite the bot to your server

1. Open **OAuth2** → **URL Generator** in the left menu
2. Select `bot` under **Scopes**
3. Select the permissions below under **Bot Permissions**
4. Open the generated URL and choose the server to add babyrite to

| Permission | Purpose |
| --- | --- |
| View Channels | Look up the source and linked channels |
| Send Messages | Post expansions |
| Send Messages in Threads | Post expansions inside threads |
| Embed Links | Post message previews as embeds |
| Read Message History | Fetch the linked message and reply to the original message |
| Connect | Quote messages from the text chat of voice and stage channels |

You can also invite the bot by replacing `CLIENT_ID` in the URL below with your application ID. It includes all of the permissions above.

```md
https://discord.com/oauth2/authorize?client_id=CLIENT_ID&scope=bot&permissions=274879040512
```

::: tip Channels the bot cannot view
babyrite cannot fetch messages from channels it cannot view. If you want a channel to be quotable, grant babyrite's role the permissions above in that channel.
:::

## Next steps

Once you have the token, follow [Getting Started](./getting-started) to start babyrite.
