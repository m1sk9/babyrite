---
title: Migration to v1
editLink: true
---

# Migration to v1

v1 will be released on 2024/11/01. With this release, babyrite will become a
stable version.

## Migration

Docker Compose `v0` will refer to the final version `v0.13.0` and will no longer
be updated.

To use the Docker Image of `v1`, change it as follows:

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v0  // [!code --]
    image: ghcr.io/m1sk9/babyrite:v1  // [!code ++]
    env_file:
      - .env
    restart: always
```

## Migration of Configuration Files

The format of the configuration file has been changed (the change itself was
made in `v0.12.0`).

Please change it as follows:

1. Rename `config.yaml` to `config.toml`.

   ```sh
   mv config.yaml config.toml
   ```

2. Rewrite the file content
   - Refer to [Settings](./settings.md#settings) for each configuration item.

   ```toml
   logger_format='compact'

   [preview]
   is_mention=true
   ```

> [!TIP]
>
> From v1, if no configuration file is specified, the default settings will be
> applied.
