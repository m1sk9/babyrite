---
title: v1 への移行
editLink: true
---

# v1 への移行

2024/11/01 に v1 がリリースされます. 当リリースをもって babyrite
は安定版となります.

## 移行

Docker Compose の `v0` は最終バージョン `v0.13.0` を指すようになり,
今後アップデートが行われなくなります.

`v1` の Docker Image を使うには次のように変更します:

```yaml
services:
  app:
    image: ghcr.io/m1sk9/babyrite:v0  // [!code --]
    image: ghcr.io/m1sk9/babyrite:v1  // [!code ++]
    env_file:
      - .env
    restart: always
```

## 設定ファイルの移行

設定ファイルのフォーマットが変更されています (変更自体は `v0.12.0`
で行われています)

次のように変更してください:

1. `config.yaml` を `config.toml` にリネームします.

   ```sh
   mv config.yaml config.toml
   ```

2. ファイル内容を書き換えます
   - 各設定項目は [設定](./settings.md#設定項目) を参照してください.

   ```toml
   logger_format='compact'

   [preview]
   is_mention=true
   ```

> [!TIP]
>
> v1 からは設定ファイルが指定されていない場合,
> デフォルトの設定が適用されるようになりました.
