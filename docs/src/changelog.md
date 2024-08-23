# Changelog

Updates are available for major versions only.

The full version should be checked on [GitHub](https://github.com/m1sk9/babyrite/releases).

----

<!-- toc -->

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
