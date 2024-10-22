import { defineConfig } from "vitepress";

export const enConfig = defineConfig({
  lang: "ja-JP",
  themeConfig: {
    nav: [
      { text: "Document", link: "/getting-started" },
      { text: "FAQ", link: "/faq" },
      // { text: "v1", link: "/migration-v1" },
    ],
    sidebar: [
      {
        text: "Getting Started",
        link: "/getting-started",
      },
      // {
      // 	text: "Migration to v1",
      // 	link: "/migration-v1"
      // },
      {
        text: "Settings",
        link: "/settings",
      },
      { text: "FAQ", link: "/faq" },
      {
        text: "Guide",
        collapsed: false,
        items: [
          {
            text: "Preview",
            link: "/guide/preview",
          },
          {
            text: "Cache",
            link: "/guide/cache",
          },
          {
            text: "Hosting",
            link: "/guide/hosting",
          },
        ],
      },
    ],
  },
});
