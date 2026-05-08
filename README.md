# Pinnacle - Local Document Intelligence

**Pinnacle** is a professional-grade, local-first document intelligence platform designed for privacy-conscious users and enterprises. Built with Rust and Tauri, it provides exceptional performance for complex PDF operations without ever letting your data leave your device.

## Why Pinnacle?
- **Privacy First:** All processing happens 100% locally on your machine.
- **High Performance:** Powered by a Rust backend for lightning-fast document manipulation.
- **Intelligence Built-in:** Automated metadata extraction, content analysis, and smart organization.
- **Comprehensive Toolset:** From basic merging to advanced cryptographic signatures.

## Core Capabilities
- **Smart Analysis:** Extract text, metadata, and structural insights automatically.
- **Precise Manipulation:** Merge, split, reorder, and rotate with ease.
- **Secure Annotations:** Highlight, comment, and markup documents natively.
- **Professional Signatures:** Add visual signatures or cryptographic PFX-based digital signatures.
- **Advanced Security:** Flatten annotations, redact sensitive info, and manage PDF encryption.
- **Form Management:** Efficiently fill and extract data from PDF forms.

## Supported Platforms
- ✅ **macOS** (Silicon & Intel)
- ✅ **Windows** (10/11)
- ✅ **Linux**
- ✅ **iPadOS/iOS** (Native & WebAssembly)

---

## Quick Start for Developers

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

3. **Run Backend Tests**
   ```sh
   cd src-tauri
   cargo test
   ```

---

## Project Structure
- `src/ui/`: Svelte 5 frontend with Tailwind CSS and Runes for state management.
- `src-tauri/src/pdf/`: High-performance Rust modules for PDF logic (using `lopdf`).
- `src-tauri/tauri.conf.json`: Application configuration and security allowlists.

## Documentation
- [Backend Architecture](./src-tauri/src/pdf/README.md) (Work in progress)
- [UI State Management](./src/ui/src/lib/state/README.md) (Work in progress)

## License
MIT License - See `LICENSE` for details.

---

## Maintainer
[Prashant Bansal](mailto:dev@prashantb.me)
