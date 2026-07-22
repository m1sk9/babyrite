import { DefaultTheme } from "vitepress";

export const en: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/LunaticChat/edit/main/website/src/:path',
    text: 'Edit this page on GitHub',
  },
  sidebar: {
    '/docs/': []
  }
}
