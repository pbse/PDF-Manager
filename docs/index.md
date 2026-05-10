---
layout: home

hero:
  name: "Pinnacle"
  text: "Document Intelligence Redefined."
  tagline: "A blazing fast, local-first, privacy-focused PDF management and intelligence application built with Rust and Svelte 5."
  image:
    src: /logo.png
    alt: Pinnacle Logo
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Features Overview
      link: /guide/features
    - theme: alt
      text: View on GitHub
      link: https://github.com/prashantbansal/Pinnacle

features:
  - title: 🔒 Local-First Privacy
    details: Your documents never leave your machine. No cloud uploads, no subscriptions, just pure local performance using IndexedDB.
    link: /guide/local-first
    linkText: Learn why local-first matters
  - title: ⚡ Rust Powered Performance
    details: Harnessing the raw speed of Rust and Tauri for instant PDF parsing, splitting, merging, and forensic redaction.
  - title: 🧠 Knowledge Graph
    details: Automatically extract entities (Dates, Organizations) and visualize connections between your documents interactively.
    link: /guide/knowledge-graph
    linkText: Explore the Knowledge Graph
  - title: 📝 Advanced Annotation
    details: Highlight, comment, draw, and extract insights seamlessly. Compare versions and track your research natively.
  - title: 🎨 Neubrutalist Aesthetics
    details: A beautiful, modern interface crafted with Svelte 5 Runes and Tailwind CSS, featuring high contrast and glassmorphism.
  - title: 🛡️ Security & Forensics
    details: Secure your PDFs with watermarks, digital signatures, and permanent forensic redaction of sensitive data.
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #3b82f6 30%, #a855f7);
  --vp-home-hero-image-filter: drop-shadow(0 20px 30px rgba(59, 130, 246, 0.3));
}
</style>

