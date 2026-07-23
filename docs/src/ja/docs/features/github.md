---
layout: doc
---

# GitHub パーマリンク展開 <Badge type="tip" text="v1.0.0~" />

コミット SHA が固定された GitHub パーマリンク (Permalink) を展開すると，リンク先ファイルの内容をシンタックスハイライト付きのコードブロックとしてそのまま展開します．

::: tip この機能は feature flag 対象機能です

使用する場合は `config.toml` の `[features]` で対象キーを有効にする必要があります

```toml
[features]
github_permalink = true
```

:::

```md
https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50
```

Permalink は次の手順でコピーできます．

1. GitHub 上でソースコードを開く
2. 開始行を選択する
3. `Shift` を押しながら終了行を選択する
4. `Copy permalink` をクリックする

![](/features/github/github-permalink.gif)

- 1メッセージあたり最大3件までのリンクを展開します．
- 同一 URL の重複は無視されます．

## 対応パターン

対応する Permalink のパターンは次のとおりです．

```md
https://github.com/{owner}/{repo}/blob/{ref}/{path}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{line}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{start}-L{end}
```

- `{ref}` は以下のフォーマット二対応します．
  - Commit SHA ( 4 桁から 40 桁の 16 進数)
  - ブランチ名 ( `main` や `feat/add-github` )
  - タグ名 ( `release-v1.0`, `v2.9.0` )
- クエリ文字列は破棄されます．

## コンテンツ取得の制限

コンテンツを取得には `https://raw.githubusercontent.com/` を使用します．

レスポンスサイズ (`Content-Length`) が 1MB ( `1_048_576` bytes ) を超える場合は取得せずにエラーになります．

### プレビューをキャンセル

プレビューを行いたくない場合はリンクを `<>` で囲むとそのリンクは無視されます．

```md
<https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50>
```
