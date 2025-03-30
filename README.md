# PDF-Manager
All in one PDF Manager App
A cross-platform, efficient PDF manager built using Rust + Tauri. It can parse, merge, split, and extract pages from PDF files with minimal memory usage and exceptional performance.

## Features
- **Parse PDFs:** Quickly extract text, metadata, and analyze PDF structures.
- **Merge PDFs:** Combine multiple PDF files into one.
- **Split PDFs:** Break PDFs into separate pages or defined page ranges.
- **Extract Pages:** Export individual pages into standalone PDFs.

## Supported Platforms
- ✅ macOS
- ✅ Windows
- ✅ Linux
- ✅ iPadOS (via WebAssembly/Web App deployment)

---

## Prerequisites

Ensure these tools are installed:

- [Rust](https://rustup.rs/) (stable channel recommended)
- [Node.js](https://nodejs.org/) (LTS recommended)
- [pnpm](https://pnpm.io/) or npm (pnpm recommended for speed)

Check installations:
```sh
rustc --version
node -v
pnpm -v  # or npm -v
```

---

## Installation and Setup

Clone the repository and navigate to the directory:
```sh
git clone https://github.com/pbse/PDF-Manager.git
cd PDF-Manager
```

Install frontend dependencies:
```sh
pnpm install  # or npm install
```

Install Rust dependencies (managed automatically by cargo):
```sh
cd src-tauri
cargo build
```

---

## Development

Run the application locally in development mode:
```sh
pnpm tauri dev  # or npm run tauri dev
```

---

## Building and Distributing

Create optimized production builds for your platform:

```sh
pnpm tauri build  # or npm run tauri build
```

The built app will appear in:
- **macOS:** `src-tauri/target/release/bundle/dmg/`
- **Windows:** `src-tauri/target/release/bundle/msi/`
- **Linux:** `src-tauri/target/release/bundle/deb/` or AppImage

---

## Code Structure Overview
- **`src/ui/src`**: User interface built with Svelte (recommended), or alternatively React/Vue.
- **`src-tauri/`**: Tauri app configuration.
- **`src-tauri/src/pdf/`**: Core Rust logic for efficient PDF manipulation.

---

## Dependencies
- **Rust PDF manipulation:** [`lopdf`](https://crates.io/crates/lopdf)
- **Tauri:** [Official Docs](https://tauri.app/)
- **UI Framework:** [Svelte](https://svelte.dev/), [Vue.js](https://vuejs.org/), or [React](https://reactjs.org/) (choose one according to preference)

---

## Contributing
Contributions welcome! Follow standard GitHub workflows:
- Fork repository, make changes, open pull request.

---

## License
This project is open-sourced under the MIT License. See `LICENSE` file for details.

---

## Maintainer
- [Prashant Bansal](mailto:dev@prashantb.me)
- GitHub: [pbse](https://github.com/pbse)

