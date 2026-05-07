# Pinnacle - Project Instructions

## Project Overview
Pinnacle is a local-first, privacy-focused document intelligence application. It leverages Rust for performance-critical PDF manipulation and Svelte 5 (Runes) for a reactive, modern UI.

## Architecture Guidelines
- **Local-First:** Never add features that require sending document content to a remote server unless explicitly requested and clearly marked as "Cloud-Optional".
- **Performance:** Complex PDF logic must reside in the Rust backend (`src-tauri/src/pdf`).
- **State Management:** Use Svelte 5 Runes (`$state`, `$derived`, `$effect`) located in `src/ui/src/lib/state/`.
- **Styling:** Use Tailwind CSS. Follow the existing "Neubrutalist" meets "Glassmorphism" aesthetic (high contrast, sharp rounded corners, subtle blurs).

## Development Workflow
- **Prerequisites:** Rust (Stable), Node.js (LTS).
- **Commands:**
  - `npm run tauri dev`: Start the application in development mode.
  - `cargo test`: Run backend unit tests (in `src-tauri`).
  - `npm run lint`: (If configured) Run linting checks.

## Tech Stack
- **Backend:** Rust, Tauri v2, `lopdf`.
- **Frontend:** Svelte 5, Tailwind CSS, Vite.
- **Testing:** Rust unit tests for core logic, Vitest for UI logic.

## Onboarding Checklist
1. Install Rust and Node.js.
2. Run `npm install` at the root.
3. Run `npm run tauri dev` to verify the environment.
4. Review `src-tauri/src/pdf/mod.rs` for available backend commands.
5. Review `src/ui/src/lib/state/appState.svelte.ts` for global UI state.
