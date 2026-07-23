import type { DefaultTheme } from 'vitepress';

export const en: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/LunaticChat/edit/main/website/src/:path',
    text: 'Edit this page on GitHub',
  },
  sidebar: {
    '/docs/': [
      {
        items: [
          { text: 'Getting Started', link: '/docs/getting-started' },
          { text: 'Configuration', link: '/docs/configuration' },
        ],
      },
      {
        text: 'Features',
        items: [
          { text: 'Message Preview', link: '/docs/features/citation' },
          { text: 'Caching', link: '/docs/features/cache' },
          { text: 'GitHub Permalink Expansion', link: '/docs/features/github' },
        ],
      },
      {
        text: 'Command Reference',
        items: [
          { text: 'version', link: '/docs/reference/command/version' },
          { text: 'ping', link: '/docs/reference/command/ping' },
          { text: 'help', link: '/docs/reference/command/help' },
          { text: 'config', link: '/docs/reference/command/config' },
          { text: 'debug', link: '/docs/reference/command/debug' },
        ],
      },
    ],
  },
};
