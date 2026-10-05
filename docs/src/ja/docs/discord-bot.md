---
layout: doc
---

# Discord Bot の準備

babyrite を起動する前に，Discord Developer Portal で Bot を作成し，サーバーへ招待しておく必要があります．

## Bot を作成する

1. [Discord Developer Portal](https://discord.com/developers/applications) を開き，**New Application** からアプリケーションを作成する
2. 左メニューの **Bot** を開く
3. **Reset Token** を押してトークンを発行し，控えておく
   - このトークンを環境変数 `DISCORD_API_TOKEN` に設定します．
   - トークンは再表示できません．紛失した場合は再発行してください．

::: danger トークンの取り扱い
トークンを知っている人は誰でも Bot を操作できます．リポジトリへのコミットや公開の場への貼り付けは避けてください．漏洩した場合は直ちに再発行してください．
:::

## Message Content Intent を有効にする

babyrite はメッセージ本文からリンクを検出するため，特権 Intent である **Message Content Intent** を必要とします．

1. **Bot** ページの **Privileged Gateway Intents** を開く
2. **Message Content Intent** を有効にして保存する

::: warning 有効にしないと起動できません
Message Content Intent が無効のまま起動すると，Discord から接続を拒否され，babyrite はエラー終了します．
:::

babyrite が要求する Intent は次の 3 つです．特権 Intent は Message Content Intent のみです．

| Intent | 用途 |
| --- | --- |
| `GUILDS` | チャンネル・スレッド・ロールの変更を受け取り，キャッシュを破棄する |
| `GUILD_MESSAGES` | メッセージの投稿を受け取る |
| `MESSAGE_CONTENT` | メッセージ本文からリンクを検出する |

## サーバーへ招待する

1. 左メニューの **OAuth2** → **URL Generator** を開く
2. **Scopes** で `bot` を選択する
3. **Bot Permissions** で以下の権限を選択する
4. 生成された URL を開き，babyrite を導入するサーバーを選ぶ

| 権限 | 用途 |
| --- | --- |
| View Channels (チャンネルを見る) | 引用元・引用先のチャンネルを参照する |
| Send Messages (メッセージを送信) | 展開結果を投稿する |
| Send Messages in Threads (スレッドでメッセージを送信) | スレッド内で展開結果を投稿する |
| Embed Links (埋め込みリンク) | メッセージ引用を埋め込みとして投稿する |
| Read Message History (メッセージ履歴を読む) | 引用先のメッセージを取得し，元のメッセージへ返信する |
| Connect (接続) | ボイスチャンネル・ステージチャンネルのテキストチャットにあるメッセージを引用する |

以下の URL の `CLIENT_ID` を自分のアプリケーション ID に置き換えても招待できます．上記の権限がすべて含まれています．

```md
https://discord.com/oauth2/authorize?client_id=CLIENT_ID&scope=bot&permissions=274879040512
```

::: tip Bot が閲覧できないチャンネルについて
babyrite は自分が閲覧できないチャンネルのメッセージを取得できません．特定のチャンネルを引用対象にしたい場合は，そのチャンネルで babyrite のロールに上記の権限を与えてください．
:::

## 次のステップ

トークンを用意したら，[はじめる](./getting-started) の手順に従って babyrite を起動してください．
