import { defineConfig } from "vitepress";
import { jaConfig } from "./ja.ts";
import { enConfig } from "./en.ts";

export default defineConfig({
  title: "🦀 babyrite",
  description: "babyrite docs",
  themeConfig: {
    socialLinks: [
      { icon: "github", link: "https://github.com/m1sk9/babyrite" },
      { icon: "discord", link: "https://run.m1sk9.dev/discord" },
    ],
    footer: {
      message:
        "babyrite is published under MIT LIcense - Not affiliated with Discord.",
      copyright: "© 2023-2024 m1sk9",
    },
    editLink: {
      pattern: "https://github.com/m1sk9/babyrite/edit/main/docs/:path",
      text: "Edit this page on GitHub",
    },
    search: {
      provider: "local",
    },
  },
  locales: {
    root: {
      label: "English",
      lang: "en-US",
      ...enConfig,
    },
    ja: {
      label: "日本語",
      lang: "ja-JP",
      link: "/ja/",
      ...jaConfig,
    },
  },
});
