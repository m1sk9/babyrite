---
title: Hosting
editLink: true
---

# Hosting

## 概要

- babyriteの公式ホストはありません (ほとんどの場合, 自分でホストします)
- 自分でホストするのは面倒です.やり方がわからない場合は,
  他の開発者が公開している一般的な引用ボットを利用することをお勧めします
  (中にはOSSではなく, 有料サービスとして公開されているものもあります)
  - babyriteは, 莫大なリソースを必要とせず, 無駄な機能がないため,
    自分でホスティングしたい方にとって主要な選択肢の一つに過ぎません.

## babyriteをホストする方法

24時間稼働するサーバーが必要です.
最も一般的な方法はVPSやクラウドサービスを利用することです.

### Docker Compose を使う

基本的なホスティング方法は Docker Compose で, 比較的簡単に使用できます.

- Docker Compose については [こちら](https://docs.docker.com/compose/install/)
- Docker Compose によるホストの方法は [こちら](../getting-started.md#はじめる)

### Kubernetes (k8s) を使う

Kubernetes に慣れていれば, babyrite をホストするために Kubernetes
を使うことができます.

- Kubernetes については [こちら](https://kubernetes.io/docs/home/)

## システム要件

babyrite は Rust でコンパイルされた動的リンクバイナリです. discord.js や JDA
が作成したボットとは異なりランタイムを必要としません.

- 1 CPU Core
- 512MB RAM

サポートは **LinuxとmacOS** のみです.

> [!TIP]
>
> 静的バイナリが必要な場合は,
> [自分でソースからコンパイル](../getting-started.md#テスト版を使用する)
> してください.
>
> Rustは標準ライブラリ(`libc`など)はシステムの共有ライブラリに依存しているため,
> デフォルトで動的バイナリをコンパイルします.
>
> 静的バイナリをコンパイルしたい場合は, `musl` を使用します.
>
> ```sh
> rustup target add x86_64-unknown-linux-musl
> cargo build --target x86_64-unknown-linux-musl --release
> ```

## dev-m1sk9-s1 でホストされたbabyriteを使う

> [!IMPORTANT]
>
> 公式ホストではありません. このサービスは予告なく停止される可能性があります.

私の知り合いであれば, 私が自分でホストしている babyrite
を自分でDiscordサーバーで使用することができます. 詳しくは私まで連絡ください.

- babyriteは常に最新版を使用します.
- チャンネルキャッシュは全ギルドで共有されているため,
  しばらくアクセスされていないチャンネルはキャッシュから削除されます.
- `dev-m1sk9-001`
  は自宅でホストされているUbuntuサーバーで予告なくサービスが停止したり,
  メンテナンスが行われることがあります.
  最新情報は[m1sk9の Mastodon アカウント(`mstdn.maud.io/@m1sk9`)](https://mstdn.maud.io/@m1sk9)
  をご確認ください.
