import type { DefaultTheme } from 'vitepress';

export const en: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/babyrite/edit/main/docs/src/:path',
    text: 'Edit this page on GitHub',
  },
  sidebar: {
    '/docs/': [
      {
        text: 'Introduction',
        items: [
          { text: 'Setting Up the Discord Bot', link: '/docs/discord-bot' },
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
        text: 'Operation',
        items: [
          { text: 'Troubleshooting', link: '/docs/troubleshooting' },
          { text: 'FAQ', link: '/docs/faq' },
          { text: 'v2 Migration Guide', link: '/docs/migration-v2' },
          { text: 'Building from Source', link: '/docs/build-from-source' },
        ],
      },
    ],
  },
};
