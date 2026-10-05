---
layout: doc
---

# GitHub パーマリンク展開 <Badge type="tip" text="v1.0.0~" />

GitHub 上のファイルへのリンク (パーマリンク) を送信すると，リンク先ファイルの内容をシンタックスハイライト付きのコードブロックとして展開します．

::: tip この機能は feature flag 対象機能です

デフォルトで有効です．無効にする場合は `config.toml` の `[features]` で対象キーを `false` にしてください．

```toml
[features]
github_permalink = false
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
- 展開結果はリンクごとに 1 件ずつ，元のメッセージへの返信として投稿されます．

## 展開結果の表示

コードブロックの上に，ファイルの情報が 1 行で表示されます．

```md
`src/main.rs` (L44-L50) - m1sk9/babyrite@52731e2
```

- ファイルのパス，行範囲，`{owner}/{repo}@{ref}` の順に表示されます．
- `{ref}` がコミット SHA の場合は先頭 7 文字に短縮されます．ブランチ名・タグ名はそのまま表示されます．
- 行範囲を指定しない場合は，ファイルの先頭から表示します．
- 表示する行数が `github.max_lines` (デフォルト 50 行) を超える場合は，先頭から `max_lines` 行までに切り詰め，`truncated to 50 lines` のように表示します．上限は [設定](../configuration) で変更できます．
- 行範囲がファイルの末尾を超えている場合は，末尾までを表示します．

::: info メンションと記号の扱い
- 展開結果の返信では誰にもメンションしません．ファイル内に `@everyone` やユーザーメンションが含まれていても通知は飛びません．
- ファイル内に ```` ``` ```` が含まれている場合，コードブロックが途中で閉じないよう，見た目を変えずに無害化して表示します．
:::

## 対応パターン

対応する Permalink のパターンは次のとおりです．

```md
https://github.com/{owner}/{repo}/blob/{ref}/{path}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{line}
https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{start}-L{end}
```

- `{ref}` は以下のフォーマットに対応します．
  - Commit SHA ( 4 桁から 40 桁の 16 進数)
  - ブランチ名 ( `main` や `feat/add-github` )
  - タグ名 ( `release-v1.0`, `v2.9.0` )
- クエリ文字列は破棄されます．
- 公開リポジトリのみ対応しています．babyrite は認証なしでファイルを取得するため，プライベートリポジトリのファイルは展開できません．

## コンテンツ取得の制限

コンテンツの取得には `https://raw.githubusercontent.com/` を使用します．

レスポンスボディは逐次読み込まれ，コードブロックの生成に必要な行が揃った時点で転送を打ち切ります．そのためファイルの残りはダウンロードされず，1MB を超えるファイルであっても実際に表示する行が上限に収まっていればプレビューできます．

最後に表示する行までに読み込んだバイト数が 1MB ( `1_048_576` bytes ) を超えた場合は取得を中断してエラーになります．上限は `Content-Length` ではなく実際に受信したバイト数に対して適用されます．`raw.githubusercontent.com` は chunked transfer encoding で応答することがあり，その場合このヘッダーが存在しないためです．

リクエスト全体 (接続・レスポンス・ボディ読み込み) には 10 秒のタイムアウトが設定されています．

### プレビューをキャンセル

プレビューを行いたくない場合はリンクを `<>` で囲むとそのリンクは無視されます．

```md
<https://github.com/m1sk9/babyrite/blob/52731e2a66c5647b3bd30b429af87c5e181e3434/src/main.rs#L44-L50>
```

## サポートされている拡張子

シンタックスハイライトに対応している拡張子は次の通りです．

なお，Discordのコードブロックは構文ハイライトに [highlight.js](https://highlightjs.org/) を使用しているため，highlight.js がサポートしていない言語は babyrite の設定に関わらずハイライト表示されません．

サポートされている言語であるにもかかわらず構文ハイライトが正しく機能しない場合は，[新しく Issue を作成してください](https://github.com/m1sk9/babyrite/issues/new)．

::: tip

未サポート拡張子は，拡張子の名前がそのまま言語ヒントとして使用されます．

:::

::: tip `.m` について

`.m` は Objective-C と Matlab の両方で使われる拡張子ですが，GitHub 上でより一般的な Objective-C として扱います．Matlab のファイルは Objective-C としてハイライトされます．

:::

::: details 対応表 (クリックで開きます)

| 拡張子                                    | 言語         |
| ----------------------------------------- | ------------ |
| `.rs`                                     | Rust         |
| `.py`, `.pyi`, `.pyw`                     | Python       |
| `.js`                                     | JavaScript   |
| `.ts`                                     | TypeScript   |
| `.jsx`                                    | JSX          |
| `.tsx`                                    | TSX          |
| `.rb`                                     | Ruby         |
| `.go`                                     | Go           |
| `.java`                                   | Java         |
| `.kt`, `.kts`                             | Kotlin       |
| `.c`, `.h`                                | C            |
| `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`     | C++          |
| `.cs`                                     | C#           |
| `.swift`                                  | Swift        |
| `.php`                                    | PHP          |
| `.scala`                                  | Scala        |
| `.sh`, `.bash`, `.zsh`, `.fish`           | Bash         |
| `.ps1`, `.psm1`                           | PowerShell   |
| `.html`, `.htm`                           | HTML         |
| `.css`                                    | CSS          |
| `.scss`                                   | SCSS         |
| `.sass`                                   | Sass         |
| `.less`                                   | Less         |
| `.json`                                   | JSON         |
| `.yaml`, `.yml`                           | YAML         |
| `.toml`                                   | TOML         |
| `.xml`                                    | XML          |
| `.sql`                                    | SQL          |
| `.md`, `.markdown`                        | Markdown     |
| `.lua`                                    | Lua          |
| `.r`                                      | R            |
| `.dart`                                   | Dart         |
| `.zig`                                    | Zig          |
| `.nim`                                    | Nim          |
| `.ex`, `.exs`                             | Elixir       |
| `.erl`, `.hrl`                            | Erlang       |
| `.hs`                                     | Haskell      |
| `.ml`, `.mli`                             | OCaml        |
| `.clj`, `.cljs`, `.cljc`                  | Clojure      |
| `.tf`                                     | HCL          |
| `.vue`                                    | Vue          |
| `.svelte`                                 | Svelte       |
| `.graphql`, `.gql`                        | GraphQL      |
| `.proto`                                  | Protobuf     |
| `.jl`                                     | Julia        |
| `.m`, `.mm`                               | Objective-C  |
| `.f`, `.for`, `.f77`                      | Fortran      |
| `.fs`, `.fsx`, `.fsi`                     | F#           |
| `.adb`, `.ads`                            | Ada          |
| `.asm`, `.s`                              | x86 Assembly |
| `.lisp`, `.el`, `.cl`                     | Lisp         |
| `.scm`, `.ss`, `.rkt`                     | Scheme       |
| `.ll`                                     | LLVM IR      |
| `.vhd`                                    | VHDL         |
| `.vert`, `.frag`                          | GLSL         |
| `.j2`                                     | Jinja        |
| `.conf`                                   | Nginx        |
| `.htaccess`, `httpd.conf`, `apache2.conf` | Apache       |
| `.sty`                                    | LaTeX        |
| `.wat`                                    | WebAssembly  |
| `.mk`, `Makefile`                         | Makefile     |
| `Dockerfile`                              | Dockerfile   |
| `CMakeLists.txt`                          | CMake        |

:::
