# babyrite

[![Release babyrite](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/release.yaml)
[![Build babyrite](https://github.com/m1sk9/babyrite/actions/workflows/build.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/build.yaml)
[![clippy](https://github.com/m1sk9/babyrite/actions/workflows/clippy.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/clippy.yaml)
[![rustfmt](https://github.com/m1sk9/babyrite/actions/workflows/fmt.yaml/badge.svg)](https://github.com/m1sk9/babyrite/actions/workflows/fmt.yaml)

A citation message Discord bot

## Features

- メッセージリンクのメッセージ内容を展開する

### Todo

- [ ] 添付ファイルのサポート ( [#5](https://github.com/m1sk9/babyrite/issues/5) )
- [ ] チャンネルリストのキャッシュ ( [#6](https://github.com/m1sk9/babyrite/issues/6) )

[babyrite v1.0.0 リリースのためのマイルストーンはこちら](https://github.com/m1sk9/babyrite2/milestone/1)

## Installation

babyrite の Docker Image は ghcr.io から取得することができます.

```shell
# 最新版
docker pull ghcr.io/m1sk9/babyrite:latest

# マイナーバージョン (v0.x)
## 破壊的変更がない限りは互換性があります. こちらから取得することをおすすめします.
docker pull ghcr.io/m1sk9/babyrite:v0

# バージョン指定
docker pull ghcr.io/m1sk9/babyrite:vX.Y.Z
```

取得した Docker Image に対して環境変数を与えると babyrite を起動できます.

## Features Flag

babyrite の各オプション機能は Features Flag で有効化が可能です.

> **Note**
>
> Docker Image で起動する際は一部の Features Flag のみが利用できます. 自由に設定するには自分でビルドする必要があります.

| Flag | Description | Default (Docker Image) |
| ---- | ----------- | ------- |
| `enable_sentry` | Sentry によるエラー報告を有効化します. | `true` |

## Environment Variables

以下は babyrite が利用する環境変数です.

`Required` カラムが `Yes` の Key は指定必須です. 指定しない場合は正しく起動しません.

| Key | Description | Required | Default |
| --- | ----------- | -------- | ------- |
| `DISCORD_API_TOKEN` | Discord API トークン | Yes | --- |

環境変数の設定例は [`.env.example`](.env.example) を参照してください.

```shell
cp .env.example .env
```
