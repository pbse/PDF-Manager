import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit()],
    // Prevent Vite copying the public dir -> Tauri handles assets
    publicDir: false,
    // Esbuild target -> .esnext use TailwindCSS
    esbuild: { target: 'esnext' },
    // Tauri prevents Vite clearing screen -> Fix
    clearScreen: false,
    // Tauri needs fixed port for development -> Set
    server: {
        strictPort: true,
        port: 5173, // Default SvelteKit port
        // Make server accessible externally if needed (e.g. mobile testing)
        // host: '0.0.0.0'
    },
    //Env variables prefix -> TAURI_
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
        // Tauri uses esbuild -> set target to latest supported version
        target: ['es2021', 'chrome100', 'safari13'],
        // Don't minify -> Tauri handle minification
        minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
        // Produce sourcemaps even not in dev mode
        sourcemap: !!process.env.TAURI_DEBUG,
    }
});