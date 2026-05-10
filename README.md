# Pinnacle - Local Document Intelligence

[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://pbse.github.io/Pinnacle/)

**Pinnacle** is a professional-grade, local-first document intelligence platform designed for privacy-conscious users and enterprises. Built with Rust and Tauri, it provides exceptional performance for complex PDF operations without ever letting your data leave your device.

Read the full documentation at [pbse.github.io/Pinnacle](https://pbse.github.io/Pinnacle/).

## Why Pinnacle?
- **Privacy First:** All processing happens 100% locally on your machine. No cloud, no tracking.
- **Local AI:** Chat with your documents, generate summaries, and extract insights using local LLMs.
- **High Performance:** Powered by a Rust backend for lightning-fast document manipulation.
- **Intelligence Built-in:** Automated metadata extraction, content analysis, and smart organization.

---

## 🤖 Local AI Setup

Pinnacle uses local Large Language Models (LLMs) to provide document intelligence without compromising your privacy. You can choose between two providers:

### Option 1: Ollama (Recommended)
Ollama provides the best performance and supports a wider range of models. It runs as a separate process on your machine.

1. **Install Ollama:** Download and install from [ollama.com](https://ollama.com/).
2. **Download the Model:** Open your terminal and run:
   ```sh
   ollama run llama3.2:1b
   ```
   *(Pinnacle is optimized for the Llama 3.2 1B model for its balance of speed and intelligence).*
3. **Launch Pinnacle:** The app will automatically detect your local Ollama instance. You can verify the connection in the **Assistant** pane.

### Option 2: Web-LLM (Zero-Install)
If you don't want to install extra software, Pinnacle can run models directly in your browser's WebGPU environment via MLC Web-LLM.

1. **Requirements:** A browser with **WebGPU** support (Chrome 113+, Edge 113+).
2. **Setup:** Simply switch the AI Provider to "Web-LLM" in the app settings.
3. **First Run:** On your first AI task, Pinnacle will download the `Llama-3.2-1B-Instruct` model weights (~1GB) and cache them in your browser. This only happens once.

---

## 🛠 Core Capabilities
- **Assistant:** Chat with single PDFs or your entire library. Ask questions, find citations, and synthesize information.
- **Smart Analysis:** Automatically generate summaries, suggested tags, and BibTeX entries.
- **Document Insights:** Extract entities like dates, organizations, and currency amounts.
- **Precise Manipulation:** Merge, split, reorder, and rotate PDF pages.
- **Secure Annotations:** Highlight, comment, and markup documents natively.
- **Professional Signatures:** Add visual signatures or cryptographic PFX-based digital signatures.
- **Advanced Security:** Flatten annotations, redact sensitive info, and manage PDF encryption.

---

## 🚀 Quick Start for Developers

### Prerequisites
- [Rust](https://rustup.rs/) (Stable)
- [Node.js](https://nodejs.org/) (LTS)
- [npm](https://pnpm.io/) (or pnpm/yarn)

### Setup
1. **Clone & Install**
   ```sh
   git clone https://github.com/pbse/Pinnacle.git
   cd Pinnacle
   npm install
   ```

2. **Run in Development**
   ```sh
   npm run tauri dev
   ```

3. **Run Tests**
   ```sh
   npm run test          # Runs all tests (Backend, UI, E2E)
   npm run test:backend  # Rust unit tests
   npm run test:ui       # Vitest UI tests
   ```

---

## 📱 Supported Platforms
- ✅ **macOS** (Silicon & Intel)
- ✅ **Windows** (10/11)
- ✅ **Linux**
- ✅ **iPadOS/iOS** (Native & WebAssembly)

## 📁 Project Structure
- `src/ui/`: Svelte 5 frontend with Tailwind CSS and Runes for state management.
- `src-tauri/src/pdf/`: High-performance Rust modules for PDF logic.
- `src-tauri/tauri.conf.json`: Application configuration and security allowlists.

## 📄 License
MIT License - See `LICENSE` for details.

---

## Maintainer
[Prashant Bansal](mailto:dev@prashantb.me)
