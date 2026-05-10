# Local-First Architecture

Pinnacle is fundamentally designed as a **local-first** application. This means your data belongs to you, and it never leaves your machine unless you explicitly choose to share it.

## Why Local-First?

In an era of cloud-based document processing, your sensitive PDFs (contracts, bank statements, personal notes) are often uploaded to third-party servers. Pinnacle flips this model:

1. **Absolute Privacy:** We don't have servers. We don't have databases. Your documents are stored entirely on your device using IndexedDB and the local file system.
2. **Instant Performance:** No waiting for files to upload or download. Processing happens instantly using the power of Rust on your own machine.
3. **Offline Access:** You can read, annotate, search, and analyze your documents even when you are completely offline.

## How it Works

Pinnacle achieves this through a carefully chosen tech stack:

- **Tauri & Rust:** The backend is written in Rust, utilizing libraries like `lopdf` to manipulate PDF files directly on your disk at native speeds.
- **Svelte 5:** The UI is reactive and snappy, running in a lightweight WebView.
- **Local AI (Coming Soon):** We are exploring ways to bring small, efficient LLMs (like `web-llm`) directly to your device so that features like the Insight Pane can remain 100% private.
