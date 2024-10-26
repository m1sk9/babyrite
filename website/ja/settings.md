---
title: 設定
editLink: true
---

# 設定

babyrite では TOML ファイルを使い, 振る舞いをカスタマイズすることができます.

> [!TIP]
>
> 設定ファイルが存在しない場合は [デフォルト値](./settings.md#設定項目)
> が使用されます. 必ずしも設定を行う必要はありません.

## 設定方法

1. `config.toml` を作成します.
   - GitHub リポジトリにある設定例ファイルをコピーして使用することができます.
   ```sh
   curl https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.toml -o config.toml
   ```

2. `compose.yaml` に設定ファイルへのボリュームマウントを設定します.

   ```yaml
   volumes:
     - ./config.yaml:/config.yaml
   ```

3. `.env` の `CONFIG_FILE_PATH` に設定ファイルのパスを設定します.

   ```sh
   CONFIG_FILE_PATH=/config.yaml
   ```

> [!TIP]
>
> ここで設定するパスはボリュームマウントしたパスを指定してください. (Docker
> Container 内から参照できるパスです)

## 設定項目

デフォルトの設定ファイルは以下の通りです:

```toml
logger_format='compact'

[preview]
is_mention=true
```

### `logger_format`

`string` / _Available in
[v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)_ /
Default: `compact`

ログのフォーマットを指定します.

|    値     |                 説明                 |            フォーマット             |
| :-------: | :----------------------------------: | :---------------------------------: |
| `compact` | 通常のフォーマットでログを出力します | `{time} {level} {target} {message}` |
|  `json`   |      JSON形式でログを出力します      |                 略                  |

### `preview.is_mention`

`boolean` / _Available in
[v0.10.0](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v0.10.0)_ /
Default: `true`

引用時のプレビューにメンションを有効にするかどうかを指定します.
