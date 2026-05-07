<script lang="ts">
    import { appState } from '$lib/state/appState.svelte';
    import { invoke } from '@tauri-apps/api/core';

    async function openFile() {
        if (appState.lastSuccessPath) {
            await invoke('shell_open', { filePath: appState.lastSuccessPath });
        }
    }

    async function revealInFolder() {
        if (appState.lastSuccessPath) {
            await invoke('reveal_in_folder', { path: appState.lastSuccessPath });
        }
    }
</script>

{#if appState.statusMessage}
    <div class="fixed bottom-0 left-0 right-0 z-[100] px-4 py-3 flex items-center justify-between transition-all duration-300
        {appState.isError ? 'bg-red-600 text-white' : 'bg-slate-900 dark:bg-blue-600 text-white shadow-lg border-t border-white/10'}">
        
        <div class="flex items-center gap-4">
            {#if appState.isLoading}
                <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            {:else if appState.isError}
                <span class="text-lg">⚠️</span>
            {:else}
                <span class="text-lg">✅</span>
            {/if}
            
            <p class="text-sm font-medium tracking-tight">{appState.statusMessage}</p>
        </div>

        {#if appState.lastSuccessPath && !appState.isLoading && !appState.isError}
            <div class="flex gap-2">
                <button 
                    onclick={openFile}
                    class="px-3 py-1 bg-white/10 hover:bg-white/20 rounded text-xs font-bold transition-colors border border-white/10"
                >
                    Open File
                </button>
                <button 
                    onclick={revealInFolder}
                    class="px-3 py-1 bg-white/10 hover:bg-white/20 rounded text-xs font-bold transition-colors border border-white/10"
                >
                    Show in Folder
                </button>
            </div>
        {/if}
    </div>
{/if}
