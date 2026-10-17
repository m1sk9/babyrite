import { inBrowser, type Theme } from 'vitepress';
import DefaultTheme from 'vitepress/theme';
import CopyOrDownloadAsMarkdownButtons from 'vitepress-plugin-llms/vitepress-components/CopyOrDownloadAsMarkdownButtons.vue';
import { enhanceAppWithTabs } from 'vitepress-plugin-tabs/client';
import { h } from 'vue';
import './custom.css';

const LOCALE_REDIRECT_FLAG = 'babyrite-locale-redirected';

function redirectByBrowserLanguage() {
  if (!inBrowser) return;
  if (localStorage.getItem(LOCALE_REDIRECT_FLAG)) return;
  localStorage.setItem(LOCALE_REDIRECT_FLAG, '1');

  const { pathname, search, hash } = window.location;
  const isJaPath = pathname === '/ja' || pathname.startsWith('/ja/');
  const prefersJa = navigator.language.toLowerCase().startsWith('ja');

  if (!isJaPath && prefersJa) {
    window.location.replace(`/ja${pathname}${search}${hash}`);
  }
}

export default {
  extends: DefaultTheme,
  Layout: () =>
    h(DefaultTheme.Layout, null, {
      'doc-before': () => h(CopyOrDownloadAsMarkdownButtons),
    }),
  enhanceApp({ app }) {
    redirectByBrowserLanguage();
    enhanceAppWithTabs(app);
  },
} satisfies Theme;
