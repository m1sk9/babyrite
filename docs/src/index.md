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
  - title: Lightweight
    details: Built on a distroless base image for a minimal, lightweight Docker image.
    icon: 🪶
  - title: Fast
    details: Developed in Rust for high performance.
    icon: ⚡
  - title: OSS
    details: Open-source under the Apache License 2.0, self-hostable so you can run and publish your own instance.
    icon: 📖
  - title: Easy to Use
    details: Simple to set up and can be deployed in seconds.
    icon: 🚀
---

<hr class="home-divider" />

<!-- Section 1: Message Previews (text left, media right) -->
<div class="feature-showcase">
  <div class="feature-showcase-text">
    <h2>Message Previews at a Glance</h2>
    <p>
      babyrite detects Discord message links in chat and expands them into rich embedded
      content, so readers can see what's being referenced without leaving the channel.
    </p>
    <ul>
      <li>Supports Production, PTB, and Canary client URLs</li>
      <li>Expands up to 3 links per message</li>
      <li>Validates NSFW channels, permissions, and privacy before rendering</li>
    </ul>
  </div>
  <div class="feature-showcase-media">💬</div>
</div>

<hr class="home-divider" />

<!-- Section 2: GitHub Permalink Expansion (media left, text right) -->
<div class="feature-showcase reverse">
  <div class="feature-showcase-text">
    <h2>GitHub Permalinks as Code Blocks</h2>
    <p>
      babyrite detects GitHub permalinks pinned to a commit SHA and expands the linked file's
      content directly as a syntax-highlighted code block.
    </p>
    <ul>
      <li>Supports line range specifications (<code>#L10-L20</code>)</li>
      <li>Expands up to 3 links per message</li>
      <li>1MB file size limit; truncated to 50 lines by default (configurable)</li>
    </ul>
  </div>
  <div class="feature-showcase-media">🐙</div>
</div>
