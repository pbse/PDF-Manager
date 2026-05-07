<script lang="ts">
  import { chatState } from "$lib/state/chatState.svelte";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let ollamaUrl = $state("http://localhost:11434");
  let syncPath = $state("");

  async function handleSelectSyncPath() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false, directory: true });
    if (result && result.length > 0) {
      syncPath = result[0];
      await invoke("start_folder_watcher", { path: syncPath });
      appState.showStatus(`Syncing with: ${syncPath}`, false);
    }
  }

  const accents = [
    { name: "Blue", color: "#3b82f6" },
    { name: "Indigo", color: "#6366f1" },
    { name: "Rose", color: "#f43f5e" },
    { name: "Emerald", color: "#10b981" },
    { name: "Amber", color: "#f59e0b" },
  ];

  async function handleBackup() {
    const json = await historyState.exportLibrary();
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "pdf_manager_backup.json" });
    if (outputPath) {
      await invoke("write_text_file", { path: outputPath, contents: json });
      appState.showStatus("Library backup saved.", false);
    }
  }

  async function handleRestore() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      const inputPath = result[0];
      appState.startLoading("Restoring library...");
      try {
        const json = await invoke<string>("read_text_file", { path: inputPath });
        const success = await historyState.importLibrary(json);
        if (success) appState.showStatus("Library restored successfully.", false);
        else appState.showStatus("Invalid backup file.", true);
      } catch (e) { appState.showStatus(`Restore failed: ${e}`, true); }
    }
  }

  async function updateOllamaUrl() {
    // In a real app, you'd store this in a global config or localStorage
    appState.showStatus("Ollama URL updated.", false);
  }
</script>

<ToolPane title="Settings" subtitle="System Configuration">
  <div class="space-y-8">
    <div class="space-y-4">
      <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">AI Provider</h3>
      <div class="grid grid-cols-1 gap-2">
        <button 
          onclick={() => chatState.aiProvider = 'ollama'}
          class="flex items-center justify-between p-3 rounded-xl border transition-all {chatState.aiProvider === 'ollama' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-slate-800'}"
        >
          <div class="flex flex-col text-left">
            <span class="text-xs font-bold text-slate-700 dark:text-slate-200">Ollama (Local)</span>
            <span class="text-[10px] text-slate-500">Stable, requires installation.</span>
          </div>
          {#if chatState.aiProvider === 'ollama'}
            <span class="text-blue-500 text-xs">✓</span>
          {/if}
        </button>
        <button 
          onclick={() => chatState.aiProvider = 'webllm'}
          class="flex items-center justify-between p-3 rounded-xl border transition-all {chatState.aiProvider === 'webllm' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-slate-800'}"
        >
          <div class="flex flex-col text-left">
            <span class="text-xs font-bold text-slate-700 dark:text-slate-200">In-App (Experimental)</span>
            <span class="text-[10px] text-slate-500">Zero-install, browser-based.</span>
          </div>
          {#if chatState.aiProvider === 'webllm'}
            <span class="text-blue-500 text-xs">✓</span>
          {/if}
        </button>
      </div>
    </div>

    {#if chatState.aiProvider === 'ollama'}
        <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
          <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Ollama Config</h3>
          <div class="space-y-2">
            <label for="ollama-url" class="text-[9px] font-bold text-slate-500 uppercase">Server URL</label>
            <input id="ollama-url" type="text" bind:value={ollamaUrl} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
          </div>
          <div class="space-y-2">
            <label for="ollama-model" class="text-[9px] font-bold text-slate-500 uppercase">Model Name</label>
            <input id="ollama-model" type="text" bind:value={chatState.ollamaModel} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
          </div>
          <button onclick={() => chatState.checkOllama()} class="w-full py-2 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded font-bold text-[10px] uppercase tracking-widest shadow-md">Test Connection</button>
        </div>
      {/if}

      <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
        <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">General</h3>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-slate-600 dark:text-slate-300">Dark Mode</span>
            <button 
              class="w-10 h-5 bg-slate-200 dark:bg-blue-600 rounded-full relative transition-colors"
              aria-label="Toggle Dark Mode"
              onclick={() => {
                document.documentElement.classList.toggle('dark');
              }}
            >
              <div class="absolute top-1 left-1 dark:left-6 w-3 h-3 bg-white rounded-full transition-all"></div>
            </button>
          </div>
          
          <div class="space-y-2">
            <span class="text-[9px] font-bold text-slate-500 uppercase block mb-1">Accent Theme</span>
            <div class="flex gap-2">
              {#each accents as acc}
                <button 
                  onclick={() => appState.accentColor = acc.color}
                  class="w-6 h-6 rounded-full border-2 transition-all {appState.accentColor === acc.color ? 'border-slate-900 dark:border-white scale-110 shadow-md' : 'border-transparent hover:scale-105'}"
                  style="background-color: {acc.color}"
                  title={acc.name}
                  aria-label="Select {acc.name} accent color"
                ></button>
              {/each}
            </div>
          </div>

          <div class="space-y-2 pt-2 border-t border-slate-50 dark:border-slate-800">
            <span class="text-[9px] font-bold text-slate-500 uppercase block mb-1">Folder Sync (Auto-Import)</span>
            <div class="flex gap-2">
               <button onclick={handleSelectSyncPath} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-[10px] font-black uppercase tracking-tighter rounded-lg hover:border-blue-500 transition-colors shadow-sm truncate px-2">
                 {syncPath ? syncPath.split(/[/\\]/).pop() : 'Select Folder'}
               </button>
            </div>
          </div>

          <div class="space-y-2 pt-2 border-t border-slate-50 dark:border-slate-800">
            <span class="text-[9px] font-bold text-slate-500 uppercase block mb-1">Data Sovereignty</span>
            <div class="flex gap-2">
              <button onclick={handleBackup} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-[10px] font-black uppercase tracking-tighter rounded-lg hover:border-blue-500 transition-colors shadow-sm">Backup Library</button>
              <button onclick={handleRestore} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-[10px] font-black uppercase tracking-tighter rounded-lg hover:border-blue-500 transition-colors shadow-sm">Restore</button>
            </div>
          </div>
        </div>
      </div>

    <div class="pt-8 text-center">
      <p class="text-[10px] text-slate-400 font-medium">PDF Manager v1.0.0</p>
      <p class="text-[9px] text-slate-500 mt-1">Made with ❤️ for Privacy</p>
    </div>
  </div>
</ToolPane>
