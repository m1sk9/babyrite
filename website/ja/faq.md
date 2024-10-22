---
title: FAQ
editLink: true
---

# FAQ

## [Sharding](https://discord.com/developers/docs/topics/gateway#sharding)で運用できますか？

**出来ません**. babyrite は設計上複数プロセスでの運用をサポートしていません.
そのため, 2500ギルド以上での運用は推奨されません.

このような運用を希望される場合は, babyriteを利用せず,
ご自身でbotを作成されるか公開されているbotをご利用ください.

## なぜWindowsはサポートされていないのですか？

babyriteは開発者が使用している macOS(`aarch64-apple-darwin`) と, Docker
Imageで使用している Debian(`x86_64-unknown-linux-gnu`)
でのコンパイルと実行がテストされています.

Windows は GitHub Actions の実行時間が長すぎるなどの理由でテストされていません.
サポートする意味もないので, 今後の予定はありません.

## BabyriteはShuttleと一緒に使えますか？

**使えません.** 互換バージョンをリリースする予定はありません.
