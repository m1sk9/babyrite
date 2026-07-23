---
layout: doc
---

# Configuration Reference

This page summarizes babyrite's configuration options and their default values. For how to point to a configuration file, see [Getting Started](./getting-started).

## Default Configuration

If you don't provide a configuration file, the following defaults are used.

```toml
[log]
level = "babyrite=info"
format = "compact"

[features]
github_permalink = true
commands = true

[github]
max_lines = 50
```

## Configuration Options

| Key | Description | Default Value |
| --- | --- | --- |
| `log.level` | Log level filter (same syntax as `RUST_LOG`). If `RUST_LOG` is set, it takes precedence. | `"babyrite=info"` |
| `log.format` | Log output format: `"compact"` or `"json"`. | `"compact"` |
| `json_logging` | **Deprecated.** Use `log.format = "json"` instead. Only used when `log.format` is unset. | `false` |
| `features.github_permalink` | Toggles GitHub permalink expansion on or off. | `true` |
| `features.commands` | Toggles the mention-prefixed command system (e.g. `@babyrite ping`) on or off. | `true` |
| `github.max_lines` | Maximum number of lines shown without truncation. | `50` |

## Logging

babyrite uses [`tracing`](https://docs.rs/tracing) to write logs to standard output.

- **Log level** is resolved in the following order of precedence: the `RUST_LOG` environment variable takes priority if set; otherwise, `config.toml`'s `log.level` (default `babyrite=info`) is used. If you want detailed debug logs during development — cache hits/misses, external API latency, permission check outcomes, and so on — set `RUST_LOG=babyrite=debug` or `log.level = "babyrite=debug"`.
- **Structured output for Grafana Loki and similar tools**: setting `log.format = "json"` outputs one JSON object per line. Each line includes structured fields — `message_id`, `guild_id`, and `channel_id` — derived from a per-request span, letting you trace a single request end to end. Collect the container's standard output with Promtail or Grafana Alloy and query the fields from LogQL with the `| json` parser.
