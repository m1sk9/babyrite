---
title: Settings
editLink: true
---

# Settings

In babyrite, you can customize behavior using TOML files.

> [!TIP]
>
> If the configuration file does not exist, the
> [default values](./settings.md#settings-items) will be used. It is not
> necessary to configure it.

## How to Configure

1. Create `config.toml`.

- You can copy and use the example configuration file from the GitHub
  repository.

```sh
curl https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.toml -o config.toml
```

2. Set the volume mount to the configuration file in `compose.yaml`.

   ```yaml
   volumes:
     - ./config/config.yaml:/config/config.yaml
   ```

3. Set the path to the configuration file in `CONFIG_FILE_PATH` in `.env`.

   ```sh
   CONFIG_FILE_PATH=/config/config.yaml
   ```

> [!TIP]
>
> Specify the path set here as the volume-mounted path. (This is the path that
> can be referenced from within the Docker Container)

## Configuration Items

The default configuration file is as follows:

```toml
logger_format='compact'

[preview]
is_mention=true
```

### `logger_format`

`string` / _Available in
[v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)_ /
Default: `compact`

Specifies the log format.

|   Value   |           Description            |               Format                |
| :-------: | :------------------------------: | :---------------------------------: |
| `compact` | Outputs logs in the usual format | `{time} {level} {target} {message}` |
|  `json`   |   Outputs logs in JSON format    |                 N/A                 |

### `preview.is_mention`

`boolean` / _Available in
[v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)_ /
Default: `true`

Specifies whether to enable mentions in previews when quoting.
