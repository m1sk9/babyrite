# Changelog

The full version should be checked on [GitHub](https://github.com/m1sk9/babyrite/releases).

----

<!-- toc -->

## v0.9.2 (Backport: v0.9.1)

[released on 2024/09/02](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.2)

**Fixed**

- [#198](https://github.com/m1sk9/babyrite/pull/198): Fixed a bug that prevented babyrite from starting in the development environment.
  - The `CONFIG_FILE_PATH` environment variable must now be set to a path relative to the configuration file.

## v0.9.0

[released on 2024/08/22](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.9.0)

**The entire code base has been refactored**. Please report bugs on GitHub.

**Added**

- [Guild checks can now be bypassed](configuration.md#babyrite-configuration). Messages between guilds with babyrite are shared.
  - This setting is `false` by default. To enable it, set it to `true` in the [configuration](configuration.md#babyrite-configuration) file.

**Changed**

- The logging system has been enhanced and errors have been fleshed out.
- By default, logs for all libraries are displayed, including Serenity. To see logs for babyrite only, set `RUST_LOG` to `babyrite=info`.

**Dependencies**

- Updated serde and tokio to the latest versions.
