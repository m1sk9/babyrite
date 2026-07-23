---
layout: doc
---

# 設定

babyrite の設定項目とデフォルト値についてまとめます．設定ファイルの指定方法については [はじめる](./getting-started) を参照してください．

## デフォルト設定

設定ファイルを用意しない場合，以下のデフォルト設定が使用されます．

```toml
[log]
level = "babyrite=info"
format = "compact"

[features]
github_permalink = true
commands = true

[github]
max_lines = 50
```

## 設定項目

| Key | Description | Default Value |
| --- | --- | --- |
| `log.level` | ログレベルのフィルタ(`RUST_LOG` と同じ構文)．`RUST_LOG` が設定されている場合はそちらが優先されます． | `"babyrite=info"` |
| `log.format` | ログ出力形式：`"compact"` または `"json"`． | `"compact"` |
| `json_logging` | **非推奨．** `log.format = "json"` を使用してください．`log.format` が未設定の場合のみ使用されます． | `false` |
| `features.github_permalink` | GitHub パーマリンク展開機能の有効/無効を切り替えます． | `true` |
| `features.commands` | メンション接頭辞コマンドシステム(例：`@babyrite ping`)の有効/無効を切り替えます． | `true` |
| `github.max_lines` | 省略せずに表示する最大行数． | `50` |

## ロギング

babyrite は [`tracing`](https://docs.rs/tracing) を使用し，ログを標準出力へ出力します．

- **ログレベル**は次の優先順位で解決されます：`RUST_LOG` 環境変数が設定されていればそちらが優先され，未設定の場合は `config.toml` の `log.level`(デフォルト `babyrite=info`)が使用されます．開発時にキャッシュのヒット/ミス，外部 API のレイテンシ，権限チェックの判定などの詳細なデバッグログを見たい場合は，`RUST_LOG=babyrite=debug` または `log.level = "babyrite=debug"` を設定してください．
- **Grafana Loki 等向けの構造化出力**：`log.format = "json"` を設定すると，1 行につき 1 つの JSON オブジェクトとして出力されます．各行にはリクエストごとの span に由来する `message_id`，`guild_id`，`channel_id` を含む構造化フィールドが含まれており，1 つのリクエストをエンドツーエンドで追跡できます．Promtail や Grafana Alloy でコンテナの標準出力を収集し，`| json` パーサーで LogQL からフィールドをクエリしてください．
