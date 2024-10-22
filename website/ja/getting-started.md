---
title: はじめる
editLink: true
---

# はじめる

> [!NOTE]
>
> 前提条件:
>
> - [Docker](https://docs.docker.com/get-docker/),
>   [Docker Compose](https://docs.docker.com/compose/install/)
>   がインストールされていること
> - [Discord Developer Portal](https://discord.com/developers/applications) で
>   Bot を作成し, トークンを取得していること

> [!WARNING]
>
> babyrite には **Message Content Intent** が必要です.
>
> `Bot` タブから `Privileged Gateway Intents` の `Message Content Intent`
> を有効にしてください.

1. ディレクトリに `.env` ファイルを作成し, 環境変数を設定します.
   - `.env` ファイルのテンプレートは `curl` コマンドを使って取得できます.

   ```sh
   curl https://raw.githubusercontent.com/m1sk9/babyrite/main/.env.example -o .env
   ```

2. ディレクトリに `compose.yaml` ファイルを作成します.

   ```yaml
   services:
     app:
       image: ghcr.io/m1sk9/babyrite:v1
       env_file:
         - .env
       restart: always
   ```

3. コマンドを実行し, babyrite を起動します.

   ```sh
   docker compose up -d
   ```

セットアップ後は以下のチュートリアルを参照してください.

- [カスタマイズ (設定)](./settings.md)
- [使い方](./guide/preview.md)

## テスト版を使用する

babyrite ではテスト版を Docker Image では配布していないため,
自分の環境でコンパイルする必要があります. なおこの方法では
[Rust](https://www.rust-lang.org/tools/install)
をインストールしておく必要があります.

> [!IMPORTANT]
>
> テスト版では予期せぬ不具合が発生したり,
> 予告なく仕様が変更される可能性があります.
>
> 自己責任でご利用のうえ, ぜひフィードバックをお寄せください.

1. babyrite のリポジトリをクローンする

   ```sh
   git clone https://github.com/m1sk9/babyrite.git
   ```

2. ディレクトリに移動し, ビルドを行う

   ```sh
   cd babyrite
   cargo build --release
   ```

3. ディレクトリに `.env` ファイルを作成し, 環境変数を設定します.
4. babyrite の実行ファイルを実行します.

   ```sh
   ./target/release/babyrite
   ```
