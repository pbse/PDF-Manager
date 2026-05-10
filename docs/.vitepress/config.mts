import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Pinnacle",
  description: "Local-first, privacy-focused document intelligence",
  base: '/Pinnacle/',
  head: [['link', { rel: 'icon', href: '/Pinnacle/logo.png' }]],
  themeConfig: {
    logo: '/logo.png',
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Features', link: '/guide/features' }
    ],

    search: {
      provider: 'local'
    },

    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Features overview', link: '/guide/features' }
        ]
      },
      {
        text: 'Core Concepts',
        items: [
          { text: 'Local-First Architecture', link: '/guide/local-first' },
          { text: 'AI & Ollama', link: '/guide/ai-ollama' },
          { text: 'Knowledge Graph', link: '/guide/knowledge-graph' }
        ]
      },
      {
        text: 'Tools & Features',
        items: [
          { text: 'PDF Manipulation', link: '/guide/pdf-tools' }
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
