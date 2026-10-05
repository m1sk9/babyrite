---
layout: doc
---

# Troubleshooting

This page lists common problems and what to check.

When investigating, enable debug logs first. They show why a link was not expanded (a link to another server, NSFW, permissions, and so on).

```shell
RUST_LOG=babyrite=debug
```

To set it in the configuration file, use `log.level = "babyrite=debug"`. See [Configuration](./configuration#logging) for details.

## babyrite exits right after starting

| What to check | Details |
| --- | --- |
| Is `DISCORD_API_TOKEN` correct? | The connection is refused if the token is wrong or was invalidated by a reset. |
| Is the Message Content Intent enabled? | The connection is refused if it is disabled. See [Setting Up the Discord Bot](./discord-bot#enable-the-message-content-intent). |
| Can the configuration file be read? | If the file at `CONFIG_FILE_PATH` does not exist or is not valid TOML, babyrite exits with `Failed to read configuration file.` / `Failed to parse configuration file.` |

## The configuration is not applied

- The configuration file path is set with the `CONFIG_FILE_PATH` environment variable. If it is not set, babyrite starts with the default configuration.
- With Docker, mount the configuration file into the container and set `CONFIG_FILE_PATH` to its path inside the container.
- The configuration file is only read at startup. Restart babyrite after changing it.

## Message links are not expanded

| What to check | Details |
| --- | --- |
| Is the link wrapped in `<>`? | Wrapped links are ignored. |
| Does the message contain 4 or more links? | Only the first 3 are expanded. |
| Is the domain supported? | Only `discord.com` / `canary.discord.com` / `ptb.discord.com` are supported. `discordapp.com` is not. |
| Is the message in the same server? | Message links to another server are not expanded. |
| Is it an NSFW channel? | NSFW channels and threads under them are not expanded. |
| Can every member of the source channel read the target? | Every member who can view the source channel must have "View Channel" and "Read Message History" in the target channel. Voice and stage channels also require "Connect". |
| Are there too many members with individual permissions? | If more than 10 of the members granted or denied access individually need to be checked for one link, it is not expanded. |
| Is it a private thread? | Private threads are not expanded. |
| Can babyrite view the target? | babyrite cannot fetch messages from channels where it lacks View Channels and Read Message History (plus Connect in voice and stage channels). |

See [Message Preview](./features/citation#behavior-with-channel-permissions) for details on how permissions are handled.

::: tip If you just changed channel permissions

babyrite discards its cache when Discord reports a channel or role change, so changes usually take effect immediately. Even if a change is missed during a reconnect, it takes effect within an hour. See [Caching](./features/cache) for details.

:::

## GitHub permalinks are not expanded

| What to check | Details |
| --- | --- |
| Is the feature enabled? | Check that `features.github_permalink = false` is not set in `config.toml`. |
| Does the URL contain `blob`? | Only URLs of the form `https://github.com/{owner}/{repo}/blob/{ref}/{path}` are supported. Repository top pages and directory URLs are not expanded. |
| Is it a public repository? | babyrite fetches files without authentication, so files in private repositories cannot be expanded. |
| Is the file too large? | It is not expanded if more than 1MB is read before the last displayed line. |
| Does the message contain 4 or more links? | Only the first 3 are expanded. |

See [GitHub Permalink Expansion](./features/github) for details.

## Syntax highlighting does not work

Discord highlights code blocks with [highlight.js](https://highlightjs.org/), so languages highlight.js does not support are not highlighted.

See [GitHub Permalink Expansion](./features/github#supported-extensions) for babyrite's extension table. If a language that should be supported is not highlighted, please [open an issue](https://github.com/m1sk9/babyrite/issues/new).
