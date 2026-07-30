import type { DefaultTheme } from 'vitepress';

export const ja: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/LunaticChat/edit/main/website/src/:path',
    text: 'GitHub で編集',
  },
  sidebar: {
    '/ja/docs/': [
      {
        items: [
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
    ],
  },
};
