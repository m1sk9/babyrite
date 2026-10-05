---
layout: doc
---

# FAQ

If something is not working, see [Troubleshooting](./troubleshooting).

## Setup and operation

### Q. Do I need to host it myself?

Yes.

babyrite is a self-hosted bot. There is no public instance.

Follow [Setting Up the Discord Bot](./discord-bot) and [Getting Started](./getting-started) to run it in your own environment.

### Q. Can one babyrite serve multiple servers?

Yes. A single process handles every server the bot has joined.

However, babyrite uses only one shard, so it cannot run beyond the scale at which Discord requires sharding (2,500 servers).

### Q. Are there slash commands?

No. babyrite only detects links in messages and expands them automatically; it provides no commands to operate it.

### Q. Does babyrite store message or file contents?

No. babyrite has no database and does not store quoted messages or GitHub file contents anywhere.

The only things cached in memory are the channel information and role permissions used for visibility checks. The cache expires within an hour at most and is gone when the process exits. See [Caching](./features/cache) for details.

::: info What the logs contain

The logs include the message ID, server ID, channel ID and sender name of messages containing links. Message content is not logged.

:::

## Message preview

### Q. Can I keep a specific link from being quoted?

Wrap the link in `<>` and it will not be expanded.

```md
<https://discord.com/channels/1390203929182339092/1496887099536838706/1497066171390886000>
```

### Q. Can I disable message previews?

Not through the configuration.

Message preview is babyrite's core feature, so it has no feature flag.

If you do not want links quoted in a specific channel, remove permissions such as "View Channel" from babyrite's role in that channel.

### Q. Why can't messages from another server be quoted?

Discord permissions are defined per server, so babyrite cannot tell whether members of the source channel can read a message in another server. To avoid leaking messages to people without access, message links to another server are not expanded.

### Q. If the linked message is edited or deleted, does the preview change?

No.

The preview is posted with the content at the time it was quoted, and later edits or deletions are not reflected. To remove a preview, have a member with permission to manage messages delete it.

## GitHub permalink expansion

### Q. Can I disable only GitHub permalink expansion?

Yes. Set the following in `config.toml`.

```toml
[features]
github_permalink = false
```

### Q. Are GitLab or Gist links supported?

No. Only links of the form `https://github.com/{owner}/{repo}/blob/...` are expanded.

### Q. Can files in private repositories be expanded?

No.

babyrite fetches files from `raw.githubusercontent.com` without authentication, so only files in public repositories can be expanded.

### Q. Can I show more lines?

Change `github.max_lines` in `config.toml`. The default is 50 lines.

```toml
[github]
max_lines = 100
```

Note that Discord messages have a character limit, so setting too many lines can make posting fail.
