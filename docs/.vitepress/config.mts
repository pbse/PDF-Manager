import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Pinnacle",
  description: "Local-first, privacy-focused document intelligence",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' }
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Features', link: '/guide/features' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/prashantbansal/Pinnacle' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026-present Prashant Bansal'
    }
  }
})
