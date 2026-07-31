import { execSync } from 'node:child_process';
import { defineConfig } from 'vitepress';
import llmstxt from 'vitepress-plugin-llms';
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs';
import { en } from './config/en';
import { ja } from './config/ja';

const gitRoot = execSync('git rev-parse --show-toplevel').toString().trim();
const commitHash = execSync(`git log -1 --format=%H -- ${gitRoot}/docs/`)
  .toString()
  .trim()
  .slice(0, 7);

export default defineConfig({
  cleanUrls: true,
  description: 'A lightweight, fast citation message Discord bot.',
  head: [['link', { href: '/favicon.ico', rel: 'icon' }]],
  outDir: './dist',
  srcDir: 'src',
  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      themeConfig: {
        ...en,
        footer: {
          copyright: 'Copyright © 2026 m1sk9',
          message: `<a href="https://github.com/m1sk9/babyrite/commit/${commitHash}">babyrite/docs@${commitHash}</a>`,
        },
      },
    },
    ja: {
      label: '日本語',
      lang: 'ja-JP',
      link: '/ja/',
      themeConfig: {
        ...ja,
        footer: {
          copyright: 'Copyright © 2026 m1sk9',
          message: `<a href="https://github.com/m1sk9/babyrite/commit/${commitHash}">babyrite/docs@${commitHash}</a>`,
        },
      },
    },
  },
  title: 'babyrite',
  titleTemplate: 'babyrite',
  vite: {
    plugins: [llmstxt()],
  },
  themeConfig: {
    socialLinks: [
      { icon: 'github', link: 'https://github.com/m1sk9/babyrite' },
    ],
  },
  markdown: {
    config(md) {
      md.use(tabsMarkdownPlugin);
    },
  },
});
