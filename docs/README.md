# babyrite docs

- [インストール](#インストール)
  - [自分でビルド](#自分でビルド)
  - [Docker Image を利用する](#docker-image-を利用する)
- [babyrite のセットアップ](#babyrite-のセットアップ)
- [使い方](#使い方)
  - [メッセージを引用する](#メッセージを引用する)
  - [メッセージの引用をスキップする](#メッセージの引用をスキップする)
  - [チャンネルのキャッシュ](#チャンネルのキャッシュ)

## インストール

### 自分でビルド

babyrite をビルドするには事前に Rust の開発環境を用意する必要があります.

[公式サイト](https://www.rust-lang.org/ja/learn/get-started) に従ってインストールしてください.

```shell
# 1. babyrite をクローンする
git clone https://github.com/m1sk9/babyrite.git # HTTPS
git clone git@github.com:m1sk9/babyrite.git # SSH

# 2. babyrite をリリースビルドする
cargo build --release
```

> [!NOTE]
> リリースビルドは cargo が最適化を行いコンパイルを行います.
> `--release` オプションを付けない場合は通常のコンパイルとなります. 本番環境で利用する場合は必ずリリースビルドを行ってください.

リリースビルドが完了すると成果物が `./target/release/babyrite` に生成されます.

### Docker Image を利用する

babyrite は ghcr.io にて Docker Image を公開しています.

それぞれ `docker pull` で取得できます. この Docker Image を使用したセットアップ方法は [babyrite のセットアップ](#setup) を参照してください.

```shell
# 最新版
docker pull ghcr.io/m1sk9/babyrite:latest

# マイナーバージョン (v0.x)
docker pull ghcr.io/m1sk9/babyrite:v0

# バージョン指定
docker pull ghcr.io/m1sk9/babyrite:vX.Y.Z
```

> [!WARNING]
> `latest` タグは常に最新版を指します. このため破壊的変更を常に受け入れる可能性があるためこのタグを本番環境で利用することはおすすめしません.

## babyrite のセットアップ

babyrite をセットアップするには以下の環境変数を設定する必要があります.

| Key | Description | Required | Default |
| --- | ----------- | -------- | ------- |
| `DISCORD_API_TOKEN` | Discord API トークン | Yes | --- |

これらの環境変数を `.env` として設定してください. 設定の例は [.env.example](../.env.example) を参照してください.

```shell
cp .env.example .env
```

環境変数を設定したら babyrite を起動します. Docker Compose を利用することを推奨します.

以下のような設定ファイルを `compose.yaml` として作成してください.

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v0.4.0
    env_file:
      - .env
    deploy:
      restart_policy:
        delay: 5s
        max_attempts: 3
```

設定ファイルを作成したら以下のコマンドで babyrite を起動します.

```shell
docker compose up -d
```

バージョン等が表示されたら起動成功です.

```shell
2023-12-02T18:29:18.970884Z  INFO babyrite::event: Connected to m1sk9_debug(ID:1086688781299634216). (Using babyrite v0.3.1).
```

## 使い方

### メッセージを引用する

メッセージリンクをチャンネルに送信すると babyrite はメッセージを取得・展開します.

babyrite が引用できるのは以下のメッセージです.

- babyrite がアクセスできるチャンネルのメッセージ
- NSFW設定がOFFのチャンネルのメッセージ
- 埋め込みが含まれたメッセージ

> [!IMPORTANT]
> 荒らしを防止するため babyrite は最初のメッセージリンク **のみ** を引用します.

### メッセージの引用をスキップする

babyrite は基本すべてのメッセージを引用します. 引用したくないメッセージがある場合は `<>` で囲ってください.

```markdown
<https://discord.com/channels/683939861539192860/683939861539192863/683941506561998848>
```

### チャンネルのキャッシュ

> [!NOTE]
> チャンネルのキャッシュは [v0.3.0](https://github.com/m1sk9/babyrite/releases/tag/v0.3.0) ([v0.3.1](https://github.com/m1sk9/babyrite/releases/tag/v0.3.1)) 以降で利用できます.

babyrite は引用したメッセージのチャンネルをキャッシュとして保存し, 再度そのチャンネルのメッセージに対して引用リクエストがあった場合はキャッシュのチャンネル情報を使用しメッセージを取得します.

チャンネルのキャッシュが存在しない場合は Discord API を使用して取得します. ( v0.2.1 以前の方法) チャンネルの数が多い場合は引用に時間がかかる場合があります.
