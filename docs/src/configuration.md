# Configuration

<!-- toc -->

## Babyrite Configuration

Configuration of babyrite behavior can be written in YAML format.

Babyrite handles configuration files by setting `CONFIG_FILE_PATH` environment variable to a path relative to the configuration file. If not specified, startup will fail. (Example: `/config/config.yaml`)

Recommended settings are as follows:

```yaml
bypass_guilds: false
citation_mention: false
```

### Download configuration file

The default setting for babyrite is `. /config/config.yaml` in m1sk9/babyrite.

The default configuration can be obtained from `curl` or other sources.

```sh
curl -o config.yaml https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.yaml
```

### Configurable items

#### `bypass_guilds`

> *Available in [v0.9.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.0)*

Sets whether the guild check process is bypassed when quoting.
If the guild check process is bypassed, messages can be shared and quoted from all guilds in which babyrite is installed.

However, the message will be shared to members who do not belong to that guild. This setting is recommended when babyrite is used by multiple private guilds.

> [!CAUTION]
>
> Note that if you enable `bypass_guilds`, bots may be used for malicious purposes. This setting is only intended for use when babyrite is used among a limited group.
>
> It is recommended that this feature be disabled.

#### `citation_mention`

> *Available in [v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)*

Whether or not to make a quoted message mentions.

## Change the displayed logs

Since babyrite uses [tracing, traching-subscriber](https://github.com/tokio-rs/tracing) for logging, you can change the displayed logs by setting the environment variable `RUST_LOG` at startup.

```shell
RUST_LOG=babyrite=info cargo run
RUST_LOG=babyrite=debug cargo run
RUST_LOG=babyrite=info ./target/release/babyrite
```

When `RUST_LOG` is not set, running cargo run or the binary directly will display babyrite logs including debug events. (default)

If you want to change the type of log displayed, pass `RUST_LOG` as an environment variable.

> [!NOTE]
>
> `babyrite=` specifies the logs for the babyrite. Therefore, if you specify a crate other than babyrite in RUST_LOG (e.g. `RUST_LOG=serenity=info`), the logs for that crate will be displayed.
