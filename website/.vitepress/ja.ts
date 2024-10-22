import { defineConfig } from "vitepress";

const ja = "/ja";

export const jaConfig = defineConfig({
  lang: "ja-JP",
  themeConfig: {
    nav: [
      { text: "ドキュメント", link: ja + "/getting-started" },
      { text: "FAQ", link: ja + "/faq" },
      // { text: "v1", link: ja + "/migration-v1" },
    ],
    sidebar: [
      {
        text: "はじめる",
        link: ja + "/getting-started",
      },
      // {
      // 	text: "v1 への移行",
      // 	link: ja + "/released-v1"
      // },
      {
        text: "設定",
        link: ja + "/settings",
      },
      { text: "FAQ", link: ja + "/faq" },
      {
        text: "ガイド",
        collapsed: false,
        items: [
          {
            text: "引用する",
            link: ja + "/guide/preview",
          },
          {
            text: "キャッシュ",
            link: ja + "/guide/cache",
          },
          {
            text: "Hosting",
            link: ja + "/guide/hosting",
          },
        ],
      },
    ],
  },
});
