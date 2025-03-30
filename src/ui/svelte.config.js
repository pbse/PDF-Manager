import adapter from '@sveltejs/adapter-static'; // <-- Import adapter-static
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    // Consult https://kit.svelte.dev/docs/integrations#preprocessors
    // for more information about preprocessors
    preprocess: vitePreprocess(),

    kit: {
        // Use adapter-static
        adapter: adapter({
            // Default options are fine for Tauri SPA:
            pages: 'build',       // Output directory relative to SvelteKit project (src/ui/)
            assets: 'build',      // Output directory relative to SvelteKit project (src/ui/)
            fallback: 'index.html', // <-- IMPORTANT: Fallback for SPA routing
            precompress: false,   // Tauri doesn't need this
            strict: true          // Recommended by adapter-static docs
        }),
        // Ensure no base path is set unless you have specific needs
        paths: {
          base: ''
        },
        // Usually disable prerendering for a full SPA unless you have purely static pages
         prerender: {
            entries: []
        }
    }
};

export default config;