---
layout: doc
---

# ソースからビルド

公開されている Docker イメージを使わず，ソースコードから babyrite をビルドする手順です．

::: warning ソースビルドによるサポートについて

ソースからのビルドは，公開イメージが提供されていない環境 (Intel Mac など) 向けの手段です．この方法で動かした babyrite はサポート対象外です．通常は [はじめる](./getting-started) の Docker イメージを使用してください．

:::

## 必要なもの

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install) (stable)

::: tip

`rustup` を使っていれば必要なツールチェインが自動で導入されます．

:::

## バイナリをビルドする

```shell
git clone https://github.com/m1sk9/babyrite.git
cd babyrite
git checkout babyrite-v2.0.0
cargo build --release
```

ビルドが完了すると `target/release/babyrite` にバイナリが生成されます．

`git checkout` ではリリースタグを指定してください．`main` ブランチは未リリースの変更を含むことがあります．タグの一覧は [Releases](https://github.com/m1sk9/babyrite/releases) で確認できます．

## 起動する

環境変数を指定してバイナリを実行します．環境変数の詳細は [はじめる](./getting-started#環境変数) を参照してください．

```shell
DISCORD_API_TOKEN=your-token CONFIG_FILE_PATH=./config/config.toml ./target/release/babyrite
```

`CONFIG_FILE_PATH` を省略した場合はデフォルト設定で起動します．

## Docker イメージをビルドする

公開イメージと同じ構成の Docker イメージを手元でビルドすることもできます．リポジトリのルートで実行してください．

```shell
docker build -f docker/Dockerfile -t babyrite:local .
```

ビルドしたイメージは，Docker Compose の `image` を `babyrite:local` に置き換えて使用します．
