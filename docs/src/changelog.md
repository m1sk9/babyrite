# Changelog

The full version should be checked on [GitHub](https://github.com/m1sk9/babyrite/releases).

----

<!-- toc -->

## v0.12.0

*[Released on 2024/10/09](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.12.0)*

```shell
docker pull ghcr.io/m1sk9/babyrite:v0.12.0
```

> [!IMPORTANT]
>
> The following versions are no longer supported due to `serde_yaml` deprecation
>
> - v0.11
> - v0.10
> - v0.9

**Breaking Changes**

- [#255](https://github.com/m1sk9/babyrite/pull/225): The `YAML` format has been removed from the config file format and only the `TOML` format is supported.
  - The values in the configuration file have been changed accordingly. See [configuration](configuration.md) for details.

## v0.11.0

*[Released on 2024/10/04](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.11.0)*

```shell
docker pull ghcr.io/m1sk9/babyrite:v0.11.0
```

> [!IMPORTANT]
> 
> The following versions are no longer supported.
> 
> - v0.8
>   - v0.8.0
>   - v0.8.1
>   - v0.8.2
>   - v0.8.3
>   - v0.8.4
>   - v0.8.5
>   - v0.8.6
>   - v0.8.7
>   - v0.8.8
>   - v0.8.9

**Added**

- [#218](https://github.com/m1sk9/babyrite/pull/218): Added private channel hiding feature
  - Channels without `View Channel` permission for `@everyone` are now ignored and not quoted
- [#219](https://github.com/m1sk9/babyrite/pull/219): Account color is now used for embed color
  - If the account color is not set (typically for non-Nitro subscribers), the default color is used

**Performance**

- [#218](https://github.com/m1sk9/babyrite/pull/218): Improved performance when quoting messages

## v0.10.0

*[Released on 2024/09/06](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.2)*

**Added**

- [#207](https://github.com/m1sk9/babyrite/pull/207): JSON style logs are now available!
  - You can now output logs in [JSON format by setting `logger_format` to `json`](./configuration.md#logger_format).
  - If you are using a logging matrix such as Grafana Loki, this setting will help improve readability.

  ```json
  {"timestamp":"2024-09-06T08:09:31.752334Z","level":"INFO","fields":{"message":"Configuration: BabyriteConfig { bypass_guilds: false, citation_mention: false, logger_format: Json }"},"target":"babyrite"}
  {"timestamp":"2024-09-06T08:09:31.859359Z","level":"DEBUG","fields":{"message":"Initializing shard info: 0 - 1/1"},"target":"serenity::client","span":{"end_shard":0, "start_shard":0,"total_shards":1,"name":"start_connection"},"spans":[{"name":"start"},{"end_shard":0,"start_shard":0,"total_shards":1,"name":"start_connection"}]}
  ```

- [#205](https://github.com/m1sk9/babyrite/pull/205): Mentions in quoted messages can now be disabled.
  - [The setting `citation_mention`](./configuration.md#citation_mention) allows you to change whether babyrite adds or does not add mentions to the embed sent when quoting a message.

**Fixed**

- [#206](https://github.com/m1sk9/babyrite/pull/206): Fixed a bug that prevented quoting a message that did not contain an embed.
- [Build - #208](https://github.com/m1sk9/babyrite/pull/208): Fixed a debug log output bug with Docker images published on `ghcr.io/m1sk9/babyrite`.
- [CI/CD - #203](https://github.com/m1sk9/babyrite/pull/203): Fixed rewinding of google-release-please.
  - A pull request was created in Release CI that rewound a previous release, so we rewound it by specifying the last release commit.

**Changed (... and development change)**

- **Development**: Serenity debug logs are now displayed when `cargo run` is executed.
  - This behavior can be changed with `RUST_LOG`.
- **Development**: Added `x` scripts for building documentation, manipulating babyrite itself, etc.

## v0.9.2 (Backport: v0.9.1)

*[Released on 2024/09/02](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.2)*

**Fixed**

- [#198](https://github.com/m1sk9/babyrite/pull/198): Fixed a bug that prevented babyrite from starting in the development environment.
  - The `CONFIG_FILE_PATH` environment variable must now be set to a path relative to the configuration file.

## v0.9.0

*[Released on 2024/08/22](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.0)*

**The entire code base has been refactored**. Please report bugs on GitHub.

**Added**

- [Guild checks can now be bypassed](configuration.md#babyrite-configuration). Messages between guilds with babyrite are shared.
  - This setting is `false` by default. To enable it, set it to `true` in the [configuration](configuration.md#babyrite-configuration) file.

**Changed**

- The logging system has been enhanced and errors have been fleshed out.
- By default, logs for all libraries are displayed, including Serenity. To see logs for babyrite only, set `RUST_LOG` to `babyrite=info`.

**Dependencies**

- Updated serde and tokio to the latest versions.
