# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Babyrite is a Discord bot that automatically generates previews for Discord message links and expands GitHub permalinks into code blocks. Built with Rust (edition 2024), using Serenity for the Discord API and Tokio for async runtime.

## Common Commands

```bash
cargo build                                    # Debug build
cargo test --verbose                           # Run all tests
cargo fmt --all -- --check                     # Check formatting
cargo clippy --all-targets --all-features      # Lint
cargo llvm-cov --all-features --workspace      # Code coverage
```

## Architecture

The bot listens for Discord messages containing links and expands them into embeds or code blocks.

**Entry flow:** `main.rs` → loads config → initializes logging → starts Serenity client → `event.rs` handles messages

**Key modules:**

- **`config.rs`** — Singleton config via `OnceLock`. Loads from TOML file (`CONFIG_FILE_PATH` env var) or defaults. `EnvConfig` handles env vars (`DISCORD_API_TOKEN`).
- **`event.rs`** — Implements `serenity::EventHandler`. Filters bot/non-guild messages, parses up to 3 Discord links and 3 GitHub permalinks per message, sends expanded content as replies.
- **`cache.rs`** — Two `moka::future::Cache` instances for guild channel lists and individual channels (500 entries, 12h TTL, 1h TTI). Lookup cascade: channel cache → guild list → active threads → API.
- **`expand.rs`** — `ExpandedContent` enum: `Embed` for Discord previews, `CodeBlock` for GitHub files. `ExpandError` unifies error types.
- **`expand/discord.rs`** — Regex-based parsing of Discord message URLs (production/PTB/Canary). Validates NSFW, permissions, privacy before generating embeds.
- **`expand/github.rs`** — Parses GitHub permalinks with commit SHAs and optional line ranges (`#L10-L20`). Fetches raw content with 1MB size limit, truncates to `max_lines` (default 50).
- **`utils.rs`** — `language_from_extension()` maps file extensions to syntax highlighting language names.

## Patterns & Conventions

- **Error handling:** `thiserror::Error` for domain enums (`PreviewError`, `GitHubExpandError`, `BabyriteConfigError`), `anyhow::Result` + `anyhow::Context` for propagation.
- **Statics:** `OnceLock` for config, `LazyLock` for compiled regexes and moka caches.
- **Lint enforcement:** `#![deny(clippy::all)]` in `main.rs`.
- **Logging:** `tracing` macros (`info!`, `error!`, `debug!`). JSON or compact format based on config.
- **Testing:** Unit tests in `#[cfg(test)]` modules within each source file. No integration tests. Tests use pure data structures without mocking.
- **Commits:** Conventional commits (`feat:`, `fix:`, `chore:`, `ci:`, `refactor:`, `docs:`). Release-please automates versioning and CHANGELOG.
- **Docker:** Multi-stage build with cargo-chef for layer caching, distroless runtime image (`gcr.io/distroless/cc-debian12`).

## Environment Variables

- `DISCORD_API_TOKEN` (required) — Bot authentication token
- `CONFIG_FILE_PATH` (optional) — Path to `config.toml`
- `RUST_LOG` — Log level filter (default in Docker: `babyrite=info`)
