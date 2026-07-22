import { DefaultTheme } from "vitepress";

export const ja: DefaultTheme.Config = {
  editLink: {
    pattern: 'https://github.com/m1sk9/LunaticChat/edit/main/website/src/:path',
    text: 'GitHub で編集',
  },
  sidebar: {
    '/docs/': []
  }
}
