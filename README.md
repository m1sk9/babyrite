# babyrite

[![CI](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/ci.yaml)
[![Release babyrite](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml)
[![Apache License 2.0](https://img.shields.io/github/license/m1sk9/babyrite?color=%239944ee)](https://github.com/m1sk9/babyrite/blob/main/LICENSE)
[![codecov](https://codecov.io/github/m1sk9/babyrite/graph/badge.svg?token=QA075J11S8)](https://codecov.io/github/m1sk9/babyrite)

**babyrite** is a lightweight, fast citation message Discord bot.

```shell
# Latest Release
docker pull ghcr.io/m1sk9/babyrite:latest

# Minor Release
docker pull ghcr.io/m1sk9/babyrite:v1

# Specific Release
docker pull ghcr.io/m1sk9/babyrite:v1.0.0
```

[_API Support: requires Discord API v10_](https://discord.com/developers/docs/reference#api-versioning)

[babyrite API Documentation](https://babyrite.api.m1sk9.dev)

## Features

- **Message Previews**: babyrite parses Discord message links and displays messages as embedded content.
- **GitHub Permalink Expansion**: babyrite can expand GitHub permalinks and display the content of the linked file.
- **Lightweight**: babyrite uses [distroless](https://github.com/GoogleContainerTools/distroless) as its base image and
  consists of a very lightweight Docker Image.
- **Fast**: babyrite is developed in Rust and is very fast!
- **OSS**: babyrite is open-source and licensed under the MIT License. It also provides a way to host it yourself as
  well as publish it as OSS.
- **Easy to Use**: babyrite is very easy to use and can be deployed in seconds.

### Message Previews

Detects Discord message links (`https://discord.com/channels/...`) and expands them as embedded content.

- Supports Production, PTB, and Canary client URLs
- Expands up to 3 links per message
- Validates NSFW channels, permissions, and privacy

### GitHub Permalink Expansion

Detects GitHub permalinks (blob URLs containing a commit SHA) and expands file content as code blocks.

- Supports line range specifications (`#L10-L20`)
- Expands up to 3 links per message
- 1MB file size limit; display truncated to 50 lines by default (configurable)

<details>
<summary>Supported languages</summary>

| Extension | Language |
| --- | --- |
| `.rs` | Rust |
| `.py` | Python |
| `.js` | JavaScript |
| `.ts` | TypeScript |
| `.jsx` | JSX |
| `.tsx` | TSX |
| `.rb` | Ruby |
| `.go` | Go |
| `.java` | Java |
| `.kt`, `.kts` | Kotlin |
| `.c`, `.h` | C |
| `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx` | C++ |
| `.cs` | C# |
| `.swift` | Swift |
| `.php` | PHP |
| `.scala` | Scala |
| `.sh`, `.bash`, `.zsh`, `.fish` | Bash |
| `.ps1` | PowerShell |
| `.html`, `.htm` | HTML |
| `.css` | CSS |
| `.scss` | SCSS |
| `.sass` | Sass |
| `.less` | Less |
| `.json` | JSON |
| `.yaml`, `.yml` | YAML |
| `.toml` | TOML |
| `.xml` | XML |
| `.sql` | SQL |
| `.md`, `.markdown` | Markdown |
| `.lua` | Lua |
| `.r` | R |
| `.dart` | Dart |
| `.zig` | Zig |
| `.nim` | Nim |
| `.ex`, `.exs` | Elixir |
| `.erl`, `.hrl` | Erlang |
| `.hs` | Haskell |
| `.ml`, `.mli` | OCaml |
| `.clj`, `.cljs` | Clojure |
| `.tf` | HCL |
| `.vue` | Vue |
| `.svelte` | Svelte |
| `.graphql`, `.gql` | GraphQL |
| `.proto` | Protobuf |
| `.mk`, `Makefile` | Makefile |
| `Dockerfile` | Dockerfile |

For extensions not listed above, the extension name is used as-is for the language hint. 

Note that Discord code blocks use [highlight.js](https://highlightjs.org/) for syntax highlighting, so languages not supported by highlight.js cannot be highlighted regardless of babyrite's configuration. If syntax highlighting does not work correctly for a supported language, please [open an issue](https://github.com/m1sk9/babyrite/issues/new).

</details>

## Installation

You can install babyrite using Docker. The following command will pull the latest version of babyrite.

```shell
docker pull ghcr.io/m1sk9/babyrite:v1
```

- babyrite is tested on macOS and Linux (major distributions) as recommended environment.
  - Also compatible with Windows, but we recommend running it on macOS or Linux. (v0.19.0+)
- ARM64 environments are supported (v0.16.0+)

### Using Docker Compose

It is recommended to use Docker Compose when setting up babyrite. Direct startup using Docker images or binary files is also possible but not recommended.

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v1
    env_file:
      - .env
    restart: always
```

If you are using orchestration tools such as k8s or Docker Swarm, please configure them according to their respective configuration files.

## Configuration

You can customize the behavior of babyrite by using a dedicated configuration file. The settings are written in TOML format. Refer to [`config/config.toml`](./config/config.toml) for configuration items.

You can also start with the default settings without configuring. The following are the default settings used in that case.

```toml
json_logging = false

[features]
github_permalink = true

[github]
max_lines = 50
```

| Key                          | Description                                                 | Default Value |
| ---------------------------- | ----------------------------------------------------------- | ------------- |
| `json_logging`               | Sets whether to output logs in JSON format.                 | `false`       |
| `features.github_permalink`  | Enable or disable GitHub Permalink expansion.               | `true`        |
| `github.max_lines`           | Maximum number of lines to display without truncation.      | `50`          |

## Environment Variables

The environment variables used by babyrite are as follows. Note that the only environment variable required for startup is `DISCORD_API_TOKEN`.

| Key                 | Description                                     |
| ------------------- | ----------------------------------------------- |
| `DISCORD_API_TOKEN` | Discord API token                               |
| `CONFIG_FILE`       | Path to the configuration file (recursive path) |

## LICENSE

babyrite is published under [Apache License 2.0](./LICENSE).

<sub>
    ® 2023 - 2026 m1sk9
    <br/>
    babyrite is not affiliated with Discord.
</sub>
