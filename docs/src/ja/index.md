---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: 'babyrite'
  tagline: A lightweight, fast citation message Discord bot.
  actions:
    - theme: brand
      text: Documentation
      link: /docs/getting-started
    - theme: alt
      text: GitHub
      link: https://github.com/m1sk9/LunaticChat

features:
  - title: 軽量
    details: distroless をベースイメージに採用し，非常に軽量な Docker イメージを実現しています．
    icon: 🪶
  - title: 高速
    details: Rust で開発されており，非常に高速に動作します．
    icon: ⚡
  - title: OSS
    details: Apache License 2.0 のもとでオープンソースとして公開されており，自分でホストしたり，OSS として公開することもできます．
    icon: 📖
  - title: 使いやすい
    details: 非常に使いやすく，数秒でデプロイできます．
    icon: 🚀
---

<hr class="home-divider" />

<!-- Section 1: メッセージプレビュー (text left, media right) -->
<div class="feature-showcase">
  <div class="feature-showcase-text">
    <h2>メッセージプレビューで会話をひと目で</h2>
    <p>
      babyrite はチャット中の Discord メッセージリンクを検出し，埋め込みコンテンツとして展開します．
      チャンネルを移動しなくても，参照先で何が話されているかをその場で確認できます．
    </p>
    <ul>
      <li>Production，PTB，Canary クライアントの URL に対応</li>
      <li>1 メッセージあたり最大 3 件のリンクを展開</li>
      <li>展開前に NSFW チャンネル・権限・プライバシーを検証</li>
    </ul>
  </div>
  <div class="feature-showcase-media">💬</div>
</div>

<hr class="home-divider" />

<!-- Section 2: GitHub パーマリンク展開 (media left, text right) -->
<div class="feature-showcase reverse">
  <div class="feature-showcase-text">
    <h2>GitHub パーマリンクをコードブロックに</h2>
    <p>
      コミット SHA が固定された GitHub パーマリンクを検出し，リンク先ファイルの内容を
      シンタックスハイライト付きのコードブロックとしてそのまま展開します．
    </p>
    <ul>
      <li>行範囲の指定（<code>#L10-L20</code>）に対応</li>
      <li>1 メッセージあたり最大 3 件のリンクを展開</li>
      <li>1MB のファイルサイズ上限，デフォルトで 50 行に切り詰め（変更可能）</li>
    </ul>
  </div>
  <div class="feature-showcase-media">🐙</div>
</div>
