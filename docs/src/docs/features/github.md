---
layout: doc
---

# GitHub Permalink Expansion <Badge type="tip" text="v1.0.0~" />

When you send a link to a file on GitHub (a permalink), babyrite expands the linked file's content as a syntax-highlighted code block.

::: tip This feature is gated by a feature flag

It is enabled by default. To disable it, set the corresponding key under `[features]` in `config.toml` to `false`.

```toml
[features]
github_permalink = false
```

:::

```md
https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50
```

You can copy a permalink with the following steps.

1. Open the source code on GitHub
2. Select the starting line
3. Hold `Shift` and select the ending line
4. Click `Copy permalink`

![](/features/github/github-permalink.gif)

- Expands up to 3 links per message.
- Duplicate URLs are ignored.
- Each link's expansion is posted separately as a reply to the original message.

## How expansions are shown

A single line with the file's information is shown above the code block.

```md
`src/main.rs` (L44-L50) - m1sk9/babyrite@52731e2
```

- The file path, line range and `{owner}/{repo}@{ref}` are shown in that order.
- If `{ref}` is a commit SHA, it is shortened to its first 7 characters. Branch and tag names are shown as-is.
- Without a line range, the file is shown from the beginning.
- If the lines to show exceed `github.max_lines` (50 by default), they are truncated to the first `max_lines` lines and marked like `truncated to 50 lines`. The limit can be changed in [Configuration](../configuration).
- If the line range runs past the end of the file, the lines up to the end are shown.

::: info Mentions and special characters
- Expansion replies mention no one. Even if the file contains `@everyone` or user mentions, no one is notified.
- If the file contains ```` ``` ````, it is defused without changing how it looks, so the code block does not close early.
:::

## Supported patterns

The permalink patterns babyrite supports are:

```md
https://github.com/{owner}/{repo}/blob/{ref}/{path}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{line}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{start}-L{end}
```

- `{ref}` supports the following formats.
  - A commit SHA (4 to 40 hex digits)
  - A branch name (e.g. `main` or `feat/add-github`)
  - A tag name (e.g. `release-v1.0`, `v2.9.0`)
- Query strings are discarded.
- Only public repositories are supported. babyrite fetches files without authentication, so files in private repositories cannot be expanded.

## Content fetch limits

Content is fetched from `https://raw.githubusercontent.com/`.

The response body is read incrementally and the transfer is aborted as soon as enough lines for the code block have arrived, so the rest of the file is never downloaded. A file larger than 1MB can therefore still be previewed, as long as the lines that are actually displayed fit within the limit.

If the bytes read up to the last displayed line exceed 1MB (`1_048_576` bytes), the fetch is aborted and an error is returned. The limit is applied to the bytes actually received rather than to `Content-Length`, because `raw.githubusercontent.com` may respond with chunked transfer encoding, in which case that header is absent.

The whole request (connect, response, and body read) is bounded by a 10 second timeout.

### Canceling a preview

If you don't want a link previewed, wrap it in `<>` and it will be ignored.

```md
<https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50>
```

## Supported extensions

The extensions supported for syntax highlighting are listed below.

Note that Discord code blocks use [highlight.js](https://highlightjs.org/) for syntax highlighting, so languages not supported by highlight.js cannot be highlighted regardless of babyrite's configuration.

If syntax highlighting does not work correctly for a supported language, please [open an issue](https://github.com/m1sk9/babyrite/issues/new).

::: tip

For extensions not listed, the extension name is used as-is for the language hint.

:::

::: tip About `.m`

`.m` is used by both Objective-C and Matlab, but babyrite treats it as Objective-C, which is more common on GitHub. Matlab files are highlighted as Objective-C.

:::

::: details Extension table (click to expand)

| Extension                                 | Language     |
| ----------------------------------------- | ------------ |
| `.rs`                                     | Rust         |
| `.py`, `.pyi`, `.pyw`                     | Python       |
| `.js`                                     | JavaScript   |
| `.ts`                                     | TypeScript   |
| `.jsx`                                    | JSX          |
| `.tsx`                                    | TSX          |
| `.rb`                                     | Ruby         |
| `.go`                                     | Go           |
| `.java`                                   | Java         |
| `.kt`, `.kts`                             | Kotlin       |
| `.c`, `.h`                                | C            |
| `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`     | C++          |
| `.cs`                                     | C#           |
| `.swift`                                  | Swift        |
| `.php`                                    | PHP          |
| `.scala`                                  | Scala        |
| `.sh`, `.bash`, `.zsh`, `.fish`           | Bash         |
| `.ps1`, `.psm1`                           | PowerShell   |
| `.html`, `.htm`                           | HTML         |
| `.css`                                    | CSS          |
| `.scss`                                   | SCSS         |
| `.sass`                                   | Sass         |
| `.less`                                   | Less         |
| `.json`                                   | JSON         |
| `.yaml`, `.yml`                           | YAML         |
| `.toml`                                   | TOML         |
| `.xml`                                    | XML          |
| `.sql`                                    | SQL          |
| `.md`, `.markdown`                        | Markdown     |
| `.lua`                                    | Lua          |
| `.r`                                      | R            |
| `.dart`                                   | Dart         |
| `.zig`                                    | Zig          |
| `.nim`                                    | Nim          |
| `.ex`, `.exs`                             | Elixir       |
| `.erl`, `.hrl`                            | Erlang       |
| `.hs`                                     | Haskell      |
| `.ml`, `.mli`                             | OCaml        |
| `.clj`, `.cljs`, `.cljc`                  | Clojure      |
| `.tf`                                     | HCL          |
| `.vue`                                    | Vue          |
| `.svelte`                                 | Svelte       |
| `.graphql`, `.gql`                        | GraphQL      |
| `.proto`                                  | Protobuf     |
| `.jl`                                     | Julia        |
| `.m`, `.mm`                               | Objective-C  |
| `.f`, `.for`, `.f77`                      | Fortran      |
| `.fs`, `.fsx`, `.fsi`                     | F#           |
| `.adb`, `.ads`                            | Ada          |
| `.asm`, `.s`                              | x86 Assembly |
| `.lisp`, `.el`, `.cl`                     | Lisp         |
| `.scm`, `.ss`, `.rkt`                     | Scheme       |
| `.ll`                                     | LLVM IR      |
| `.vhd`                                    | VHDL         |
| `.vert`, `.frag`                          | GLSL         |
| `.j2`                                     | Jinja        |
| `.conf`                                   | Nginx        |
| `.htaccess`, `httpd.conf`, `apache2.conf` | Apache       |
| `.sty`                                    | LaTeX        |
| `.wat`                                    | WebAssembly  |
| `.mk`, `Makefile`                         | Makefile     |
| `Dockerfile`                              | Dockerfile   |
| `CMakeLists.txt`                          | CMake        |

:::
