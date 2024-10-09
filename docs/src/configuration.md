# Configuration

<!-- toc -->

## Babyrite Configuration

Configuration of babyrite behavior can be written in Toml format.

Babyrite handles configuration files by setting `CONFIG_FILE_PATH` environment variable to a path relative to the configuration file. If not specified, startup will fail. (Example: `/config/config.toml`)

Recommended settings are as follows:

```toml
logger_format='compact'

[preview]
bypass_guild_check=false
is_mention=true
```

### Download configuration file

The default setting for babyrite is `./config/config.toml` in [m1sk9/babyrite](https://github.com/m1sk9/babyrite).

The default configuration can be obtained from `curl` or other sources.

```sh
curl -o config.toml https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.toml
```

### Configurable items

#### `logger_format`

> *Available in [v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)*
>
> *Expected type: string*

Sets the format of the log output. Both will show logs above babyrite's `INFO` level (when using Docker images).

* `compact`: output logs in normal format. (format: `{time} {level} {target} {message}`)
* `json`: Output logs in JSON format. This is the recommended setting if you are using [Grafana Loki](https://grafana.com/oss/loki/) or similar.

> [!TIP]
>
> The following string can also be used: `Compact`, `Json`
>
> ```rs
> #[derive(Debug, Deserialize)]
> pub enum LoggerFormat {
>     #[serde(alias = "compact")]
>     Compact,
>     #[serde(alias = "json")]
>     Json,
> }
> ```

#### `preview.bypass_guild_check`

> *Available in [v0.9.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.0)*
>
> *Expected type: boolean*

Sets whether the guild check process is bypassed when quoting.
If the guild check process is bypassed, messages can be shared and quoted from all guilds in which babyrite is installed.

However, the message will be shared to members who do not belong to that guild. This setting is recommended when babyrite is used by multiple private guilds.

> [!CAUTION]
>
> Note that if you enable `bypass_guilds`, bots may be used for malicious purposes. This setting is only intended for use when babyrite is used among a limited group.
>
> It is recommended that this feature be disabled.

#### `preview.is_mention`

> *Available in [v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)*
>
> *Expected type: boolean*

Whether or not to make a quoted message mentions.
