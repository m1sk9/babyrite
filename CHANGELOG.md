# Changelog

## [0.8.8](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.7...babyrite-v0.8.8) (2024-08-08)


### Bug Fixes

* **deps:** update rust crate serde to v1.0.205 ([#176](https://github.com/m1sk9/babyrite/issues/176)) ([73291a9](https://github.com/m1sk9/babyrite/commit/73291a924d7c0b13baeaaaa9c64144d3787b53b7))

## [0.8.7](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.6...babyrite-v0.8.7) (2024-08-03)


### Bug Fixes

* **deps:** bump time to `0.3.36` ([#166](https://github.com/m1sk9/babyrite/issues/166)) ([6f8c069](https://github.com/m1sk9/babyrite/commit/6f8c0691f4b8616f2a5633703ac216e0eac6bc86))
* **deps:** update rust crate regex to v1.10.6 ([#164](https://github.com/m1sk9/babyrite/issues/164)) ([f2e2716](https://github.com/m1sk9/babyrite/commit/f2e271698aae8cef10440b524b9b2966ea7c317b))
* **deps:** update rust crate tokio to v1.39.2 ([#162](https://github.com/m1sk9/babyrite/issues/162)) ([6819c35](https://github.com/m1sk9/babyrite/commit/6819c35e6545a17ae0f7eff15a0529c62b18be80))

## [0.8.6](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.5...babyrite-v0.8.6) (2024-07-23)


### Bug Fixes

* **deps:** update rust crate tokio to v1.39.1 ([#159](https://github.com/m1sk9/babyrite/issues/159)) ([69dbed0](https://github.com/m1sk9/babyrite/commit/69dbed0e012aa3b43385e3ed0abf3bb843bd8f65))

## [0.8.5](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.4...babyrite-v0.8.5) (2024-07-16)


### Bug Fixes

* **deps:** update rust crate tokio to v1.38.1 ([#156](https://github.com/m1sk9/babyrite/issues/156)) ([09a2686](https://github.com/m1sk9/babyrite/commit/09a2686e4fe1f2ff9431e1b5134814d87f5914a0))

## [0.8.4](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.3...babyrite-v0.8.4) (2024-07-14)


### Bug Fixes

* **deps:** update rust crate typed-builder to 0.19.0 ([#154](https://github.com/m1sk9/babyrite/issues/154)) ([5de5624](https://github.com/m1sk9/babyrite/commit/5de5624e9db58f9eaa5dd664118950aa70037381))

## [0.8.3](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.2...babyrite-v0.8.3) (2024-07-08)


### Bug Fixes

* **deps:** update rust crate moka to v0.12.8 ([#150](https://github.com/m1sk9/babyrite/issues/150)) ([a77ddc0](https://github.com/m1sk9/babyrite/commit/a77ddc00d3f19cd4fb84520624d34d5333cc8187))

## [0.8.2](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.1...babyrite-v0.8.2) (2024-07-07)


### Bug Fixes

* **deps:** update rust crate serde to v1.0.204 ([#148](https://github.com/m1sk9/babyrite/issues/148)) ([bfb0400](https://github.com/m1sk9/babyrite/commit/bfb040013ae2c3d51ce692d99f23771fcabaacb1))

## [0.8.1](https://github.com/m1sk9/babyrite/compare/babyrite-v0.8.0...babyrite-v0.8.1) (2024-06-13)


### Bug Fixes

babyrite が使用している Discord API ライブラリ [Serenity v0.12.2](https://github.com/serenity-rs/serenity/releases/tag/v0.12.2) で非推奨になったメソッドを削除・置き換えを行いました.

* 非推奨メソッド `is_nsfw()` を `nsfw()` に置き換え ([#143](https://github.com/m1sk9/babyrite/issues/143)) ([6bd4047](https://github.com/m1sk9/babyrite/commit/6bd4047084a0d18a20ddf69e40ef2529cab1e317))
* 非推奨メソッド `is_private()` を削除し, DM かの判定を ID チェックのみに ([#141](https://github.com/m1sk9/babyrite/issues/141)) ([6b26a52](https://github.com/m1sk9/babyrite/commit/6b26a527a985174179b9a2f6b720af67c00832ae))
  * この変更により, babyrite は ID 取り出し処理による判定のみで DM かどうかを判断します.

## [0.8.0](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.6...babyrite-v0.8.0) (2024-05-19)


### Breaking Changes

- Sentry のサポートを削除 ([#134](https://github.com/m1sk9/babyrite/issues/134))
 - v0.8.0 から Sentry が利用できなくなります. Sentry を使用したエラー監視を行いたい場合は自前でのビルドを推奨します.

### Features

- ベースイメージを Debian 12 (bookworm) に更新 ([#131](https://github.com/m1sk9/babyrite/issues/131))

## [0.7.6](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.5...babyrite-v0.7.6) (2024-05-14)


### Bug Fixes

* release-please-action のワークフローポイントを修正 ([#125](https://github.com/m1sk9/babyrite/issues/125)) ([70afcce](https://github.com/m1sk9/babyrite/commit/70afcce2174c390ac65fa3b918037f1648b91aa1))

## [0.7.5](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.4...babyrite-v0.7.5) (2024-04-20)


### Bug Fixes

* **deps:** bump rustls from 0.21.6 to 0.21.11 ([#118](https://github.com/m1sk9/babyrite/issues/118)) ([d3e16ba](https://github.com/m1sk9/babyrite/commit/d3e16ba7bf334b16f5a928f51c0db0f16cdfaf96))

## [0.7.4](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.3...babyrite-v0.7.4) (2024-04-05)


### Bug Fixes

* **deps:** bump h2 from 0.3.24 to 0.3.26 ([#107](https://github.com/m1sk9/babyrite/issues/107)) ([3eb5f14](https://github.com/m1sk9/babyrite/commit/3eb5f148d0fe50dc204a29c82db315b53a08368b))

## [0.7.3](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.2...babyrite-v0.7.3) (2024-03-06)


### Bug Fixes

* 埋め込みありのメッセージを引用できるように ([#100](https://github.com/m1sk9/babyrite/issues/100)) ([4ab5a2b](https://github.com/m1sk9/babyrite/commit/4ab5a2bcf1e9b491993d97722e4937189e818f2b))

## [0.7.2](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.1...babyrite-v0.7.2) (2024-02-22)


### Bug Fixes

* 埋め込みの Author 欄がリクエスト送信者になる問題の修正 ([#95](https://github.com/m1sk9/babyrite/issues/95)) ([23c4e55](https://github.com/m1sk9/babyrite/commit/23c4e55d0998c92b4c9ee38b884de8aaf0c336b9))

## [0.7.1](https://github.com/m1sk9/babyrite/compare/babyrite-v0.7.0...babyrite-v0.7.1) (2024-02-21)


### Bug Fixes

* Moka のキャッシュ戦略の修正 ([#92](https://github.com/m1sk9/babyrite/issues/92)) ([589ea1f](https://github.com/m1sk9/babyrite/commit/589ea1f3e49407da00cf15aee93b702218edcc01))

## [0.7.0](https://github.com/m1sk9/babyrite/compare/babyrite-v0.6.2...babyrite-v0.7.0) (2024-02-20)


### Features

* sentry のサポート ([#84](https://github.com/m1sk9/babyrite/issues/84)) ([614fc0f](https://github.com/m1sk9/babyrite/commit/614fc0f2855a1f866e8efb6e80c8e91d8430db12))
  * Babyrite に Sentry を導入できるようになりました. `SENTRY_DSN` 環境変数を設定することで, エラーが発生した際に Sentry に通知されます.

### Performance Improvements

* 引用機能の再実装 ([#83](https://github.com/m1sk9/babyrite/issues/83)) ([cd81400](https://github.com/m1sk9/babyrite/commit/cd81400ff96e0e0aa3b27dbc7fa838fae4a50133))
  * 引用機能を再実装し, コード全体のリファクタリングを行いました.
  * キャッシュのシステムを変更しました. TTL の設定を削除し, キャッシュのサイズを制限するようにしました.

## [0.6.2](https://github.com/m1sk9/babyrite/compare/babyrite-v0.6.1...babyrite-v0.6.2) (2024-02-17)


### Bug Fixes

* **bot:** 通常メッセージ以外のメッセージを引用してしまう不具合の修正 ([#81](https://github.com/m1sk9/babyrite/issues/81)) ([7e34746](https://github.com/m1sk9/babyrite/commit/7e34746187804e81ca4bb3a87f520eb31dfe7057))

## [0.6.1](https://github.com/m1sk9/babyrite/compare/babyrite-v0.6.0...babyrite-v0.6.1) (2024-01-19)


### Bug Fixes

* **deps:** bump h2 from 0.3.21 to 0.3.24 ([#75](https://github.com/m1sk9/babyrite/issues/75)) ([1651003](https://github.com/m1sk9/babyrite/commit/165100315bc4aa85cb79069eee58c990fe48ab20))

## [0.6.0](https://github.com/m1sk9/babyrite/compare/babyrite-v0.5.0...babyrite-v0.6.0) (2024-01-03)


### Features

* スタンプのサポート ([#74](https://github.com/m1sk9/babyrite/issues/74)) ([4e07e5f](https://github.com/m1sk9/babyrite/commit/4e07e5fcda53f3415f67f4a71cc67551096155d9))


### Bug Fixes

* 一部の埋め込みメッセージが引用できない不具合を修正 ([#71](https://github.com/m1sk9/babyrite/issues/71)) ([eb6ff62](https://github.com/m1sk9/babyrite/commit/eb6ff62c112824f5c7fec5f38dc9d7d46fb12857))

## [0.5.0](https://github.com/m1sk9/babyrite/compare/babyrite-v0.4.0...babyrite-v0.5.0) (2023-12-10)


### Features

* メッセージ引用時の計測時間を表示するように ([#65](https://github.com/m1sk9/babyrite/issues/65)) ([c324926](https://github.com/m1sk9/babyrite/commit/c324926cdc4271353893c74016fff1be92890e43))
  * メッセージ取得〜埋め込み返信までの計測時間がコンソールに表示されるようになりました. これにより, メッセージ引用時のパフォーマンスを測定できます.
* ロギングシステムの実装 ([#61](https://github.com/m1sk9/babyrite/issues/61)) ([c303560](https://github.com/m1sk9/babyrite/commit/c30356087e5ada988bb7ae1b7cfac0138fd0dc73))
  * ロギングシステムを実装しました. `RUST_LOG` の設定状況により, ログレベルを変更できます. 詳しくは [ドキュメント](https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging) を参照してください.
  * この変更により, デフォルトでは Serenity など babyrite 内部クレートのログは表示されなくなりました.

### Performance Improvements

* 自動でキャッシュをアイドル・解放するように ([#66](https://github.com/m1sk9/babyrite/issues/66)) ([71c7313](https://github.com/m1sk9/babyrite/commit/71c73138e37ed8678cdc32a2119963209ae83d5a))
  * `Time to live (TTL)`: 最初のキャッシュ保存 (挿入) から1時間経過したキャッシュは自動で解放されます.
  * `Time to idle (TTI)`: キャッシュされたエントリーが30分間アクセスされなかった場合, キャッシュは自動で解放されます. ただし, アクセスがあっても `TTL` により最大1時間で解放されます.
  * この変更によりキャッシュの最大サイズ(キャパシティ)を調整しています. 最大キャパシティに達した場合, 古いキャッシュから自動で解放されます. この場合 `TTL`, `TTI` の設定は無視されます.
* Docker Image のサイズを最適化 ([#159](https://github.com/m1sk9/babyrite/pull/68)) ([17a36e5](https://github.com/m1sk9/babyrite/commit/17a36e57f6c2a0fc0c88815f2d93605e4e2e8146))
  * 不必要な apt パッケージを削除し, Docker Image のサイズを最適化しました.

| Image | Size |
| --- | --- |
| [`m1sk9/babyrite:v0.5.0`](#050-2023-12-10) | 98.93MB |
| [`m1sk9/babyrite:v0.4.0`](#040-2023-12-02) | 132MB |

## [0.4.0](https://github.com/m1sk9/babyrite/compare/babyrite-v0.3.1...babyrite-v0.4.0) (2023-12-02)


### Features

* Support Serenity v0.12.0 ([#45](https://github.com/m1sk9/babyrite/issues/45)) ([4a0b95d](https://github.com/m1sk9/babyrite/commit/4a0b95da39b0e4440ceaf8a714f09c7de5021c52))
  * Serenity v0.12.0 では、すべての ID タイプの内部表現が非公開になりました。提供される新しい表現に置き換えられました。
  * Serenity v0.12.0 では，`Activity` が削除され、新しく `ActivityData` が実装されました。ステータス更新ロジックが置き換えられました。
  * Serenity v0.12.0 では、埋め込みとメッセージのロジックに破壊的変更が加えられました。これにより、babyrite ビルドインの埋め込み添付ファイルの実装が壊れたため, Serenity 独自の Builder に置き換えられました。
    * これにより、アバターのないユーザーのメッセージを引用する際の動作が変更されました：アバターのないユーザーを引用する場合、アバターフィールドはデフォルトアバターの画像になります。


## [0.3.1](https://github.com/m1sk9/babyrite/compare/v0.3.0...v0.3.1) (2023-10-28)


### Performance Improvements

* チャンネルキャッシュのロジックを改善 ([#37](https://github.com/m1sk9/babyrite/issues/37)) ([1c229f1](https://github.com/m1sk9/babyrite/commit/1c229f1bc042e0064ff884213a643d638cb6f815))

## [0.3.0](https://github.com/m1sk9/babyrite/compare/v0.2.1...v0.3.0) (2023-10-28)


### Features

* Bot のメッセージ引用時埋め込みにフラグを表示するように ([#32](https://github.com/m1sk9/babyrite/issues/32)) ([36d7524](https://github.com/m1sk9/babyrite/commit/36d7524001f59f3fe048ec4019df591d4089d680))
* チャンネルリストのキャッシュを実装 ([#34](https://github.com/m1sk9/babyrite/issues/34)) ([37cc6c6](https://github.com/m1sk9/babyrite/commit/37cc6c63fd52ed55fec2335b4156b1e7884575fc))

## [0.2.1](https://github.com/m1sk9/babyrite/compare/v0.2.0...v0.2.1) (2023-09-14)


### Performance Improvements

* 埋め込み生成時のエラーパフォーマンスを改善 ([#22](https://github.com/m1sk9/babyrite/issues/22)) ([d893ca3](https://github.com/m1sk9/babyrite/commit/d893ca37862680dab84c462b3c810097a4ca9e77))

## [0.2.0](https://github.com/m1sk9/babyrite/compare/v0.1.1...v0.2.0) (2023-09-09)


### Features

* 添付ファイルのサポート ([#13](https://github.com/m1sk9/babyrite/issues/13)) ([453931d](https://github.com/m1sk9/babyrite/commit/453931d174503be30d10f109ba0925d791f3b725))


### Bug Fixes

* スレッド内のメッセージを引用できない問題の修正 ([#17](https://github.com/m1sk9/babyrite/issues/17)) ([316217a](https://github.com/m1sk9/babyrite/commit/316217a36fa84794b2ad26e2ac4ffd6ee535adf1))
* 埋め込みメッセージをメッセージ文字列なしで引用してしまう不具合を修正 ([#16](https://github.com/m1sk9/babyrite/issues/16)) ([8cac699](https://github.com/m1sk9/babyrite/commit/8cac6991d9b4aac82151737afeeef7ff0aeb1758))


### Performance Improvements

* エラーハンドリングの改善 ([#15](https://github.com/m1sk9/babyrite/issues/15)) ([ed79084](https://github.com/m1sk9/babyrite/commit/ed790842ddceaaf9cccb808a629adfefb48df93b))

## [0.1.1](https://github.com/m1sk9/babyrite/compare/v0.1.0...v0.1.1) (2023-08-30)


### Bug Fixes

* ビルドエラーの修正 ([9f88c10](https://github.com/m1sk9/babyrite/commit/9f88c1062fdc5b2e81097e9c963fc120461f36ba))

## 0.1.0 (2023-08-30)


### Features

* Discord API のログインロジックを追加 ([#2](https://github.com/m1sk9/babyrite/issues/2)) ([7e455fb](https://github.com/m1sk9/babyrite/commit/7e455fb557d770fbaf265a41a17ab98f30aab3a9))
* Docker のサポート ([#4](https://github.com/m1sk9/babyrite/issues/4)) ([84ebfc1](https://github.com/m1sk9/babyrite/commit/84ebfc17986d25f0749f88862a1e82cc7de0bdf2))
* 引用機能の追加 ([#7](https://github.com/m1sk9/babyrite/issues/7)) ([5651ef5](https://github.com/m1sk9/babyrite/commit/5651ef5c6c0d3ac5f43e9ac62432125e7a62dfbe))


### Bug Fixes

* サーバーが一致しなかった際引用しないように ([#8](https://github.com/m1sk9/babyrite/issues/8)) ([631414e](https://github.com/m1sk9/babyrite/commit/631414edde6770e31bc79c8b652e9fa5e4f3e482))
