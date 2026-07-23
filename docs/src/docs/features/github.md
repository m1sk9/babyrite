---
layout: doc
---

# GitHub Permalink Expansion <Badge type="tip" text="v1.0.0~" />

When you expand a GitHub permalink pinned to a commit SHA, babyrite expands the linked file's content directly as a syntax-highlighted code block.

::: tip This feature is gated by a feature flag

To use it, you need to enable the corresponding key under `[features]` in `config.toml`.

```toml
[features]
github_permalink = true
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

## Content fetch limits

Content is fetched from `https://raw.githubusercontent.com/`.

If the response size (`Content-Length`) exceeds 1MB (`1_048_576` bytes), the fetch is aborted and an error is returned.

### Canceling a preview

If you don't want a link previewed, wrap it in `<>` and it will be ignored.

```md
<https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50>
```
