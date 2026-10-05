---
layout: doc
---

# v2 への移行

babyrite v2.0.0 をリリースしました．ここでは v1 からの移行をサポートするガイドを示します．

## 内部ライブラリの切り替え

v2.0.0 では初期バージョンから Discord API Wrapper として使用してきた [serenity](https://github.com/serenity-rs/serenity) から [twilight](https://github.com/twilight-rs/twilight) に変更を行いました．

Serenity の最新リリース v0.12.5 には脆弱性 (`h2` crate に存在する unbounded empty DATA frames) が存在しますが，修正リリースが行われず，約10ヶ月間ずっとメンテナンスが停滞しています．

babyrite のようなリリースが頻繁に行われているプロジェクトでは使い続けることがリスクなため，今回切り替えすることを決定しました．

## 挙動について

babyrite の機能の多くは変わりません．また設定値が変わったりもしません．そのままアップデートすることができます．

なお，内部ライブラリの変更に伴い一部挙動が変更されています．運用時にはその点に注意してください．

### 変わること

1. **Discord API 呼び出しに 10 秒タイムアウト**が入ります
2. **再接続が指数バックオフ**になります
   - これに伴い長時間障害からの復帰が最大約 4 分遅れうる可能性があります

### 変わらないこと

- 機能
- 引用などの挙動
- シャード構成
- 起動に必要な設定・環境変数・権限

## 移行作業について

babyrite v2 にはそのままアップデートすることができますが，環境により一部作業が必要な場合があります．

### Docker Image のタグ

今回のアップデートはメジャーアップデートになるため，バージョンを固定している場合は `v1` から `v2` へ変更する必要があります．

```diff
services:
  app:
-   image: ghcr.io/m1sk9/babyrite:v1
+   image: ghcr.io/m1sk9/babyrite:v2
    env_file:
      - .env
    volumes:
      - ./config/config.toml:/config/config.toml
    restart: always
```

### Intel Mac のサポート終了 (macOSのみ)

Intel Mac (`x86_64-apple-darwin`) は [v1.4.7](https://github.com/m1sk9/babyrite/releases/tag/babyrite-v1.4.7) でサポートを終了しています．

v2.0.0 向けのイメージは公開されていないため，使用することはできません．引き続き利用するには[ローカルでビルド](./build-from-source)する必要があります (なお，サポートは行われません．)
