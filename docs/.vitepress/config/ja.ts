import type { DefaultTheme } from 'vitepress';

export const ja: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/babyrite/edit/main/docs/src/:path',
    text: 'GitHub で編集',
  },
  sidebar: {
    '/ja/docs/': [
      {
        text: '導入',
        items: [
          { text: 'Discord Bot の準備', link: '/ja/docs/discord-bot' },
          { text: 'はじめる', link: '/ja/docs/getting-started' },
          { text: '設定', link: '/ja/docs/configuration' },
        ],
      },
      {
        text: '機能',
        items: [
          { text: 'メッセージ引用', link: '/ja/docs/features/citation' },
          { text: 'キャッシュシステム', link: '/ja/docs/features/cache' },
          { text: 'GitHub パーマリンク展開', link: '/ja/docs/features/github' },
        ],
      },
      {
        text: '運用',
        items: [
          { text: 'トラブルシューティング', link: '/ja/docs/troubleshooting' },
          { text: 'よくある質問', link: '/ja/docs/faq' },
          { text: 'v2 移行ガイド', link: '/ja/docs/migration-v2' },
          { text: 'ソースからビルド', link: '/ja/docs/build-from-source' },
        ],
      },
    ],
  },
};
