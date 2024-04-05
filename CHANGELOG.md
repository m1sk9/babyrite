# Changelog

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
