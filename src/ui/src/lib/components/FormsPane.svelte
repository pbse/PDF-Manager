<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let fields = $state<{ name: string, field_type: string, value: string }[]>([]);
  let fieldUpdates = $state<Record<string, string>>({});
  let isScanning = $state(false);

  async function scanFields() {
    if (!pdfState.viewerFilePath) return;
    isScanning = true;
    try {
      fields = await invoke("get_form_fields", { path: pdfState.viewerFilePath });
      fieldUpdates = {};
      fields.forEach(f => fieldUpdates[f.name] = f.value);
      if (fields.length === 0) appState.showStatus("No interactive fields found.", false);
    } catch (e) {
      appState.showStatus(`Scan failed: ${e}`, true);
    } finally {
      isScanning = false;
    }
  }

  async function handleSaveForm() {
    if (!pdfState.viewerFilePath) return;
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "filled_form.pdf" });
    if (!outputPath) return;

    appState.startLoading("Saving form data...");
    try {
      await invoke("set_form_fields", { 
        path: pdfState.viewerFilePath, 
        updates: fieldUpdates, 
        outputPath 
      });
      appState.showStatus("Form saved successfully.", false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      appState.showStatus(`Failed to save: ${e}`, true);
    }
  }

  // AI Fill Suggestion
  async function handleAiSuggestFill() {
    // This would ideally look at user profile/history
    appState.showStatus("AI is analyzing your profile for suggestions...", false);
    // Placeholder for "Genius" feature
  }
</script>

<ToolPane title="Forms" subtitle="Interactive Filler">
  <div class="space-y-6">
    <button 
      onclick={() => pdfState.selectFile('extract')} 
      class="w-full py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors"
    >
      {pdfState.viewerFilePath ? pdfState.viewerFilePath.split(/[/\\]/).pop() : 'Select PDF Form'}
    </button>

    <button 
      onclick={() => !pdfState.viewerFilePath ? pdfState.selectFile('extract') : scanFields()} 
      disabled={pdfState.viewerFilePath && isScanning}
      class="w-full py-2 bg-blue-600 text-white rounded font-bold text-[10px] uppercase tracking-widest shadow-md"
    >
      {!pdfState.viewerFilePath ? 'Select PDF' : isScanning ? 'Scanning...' : 'Scan for Fields'}
    </button>

    {#if fields.length > 0}
      <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
        <div class="flex justify-between items-center">
          <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Fields Found ({fields.length})</h3>
          <button onclick={handleAiSuggestFill} class="text-[8px] font-black text-blue-500 uppercase tracking-tighter">✨ AI Fill</button>
        </div>

        <div class="max-h-96 overflow-y-auto space-y-3 pr-1">
          {#each fields as field}
            <div class="space-y-1">
              <label for="field-{field.name}" class="text-[9px] font-bold text-slate-500 uppercase truncate block">{field.name}</label>
              {#if field.field_type === 'Btn'}
                 <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" checked={fieldUpdates[field.name] === 'Yes'} onchange={(e) => fieldUpdates[field.name] = (e.target as HTMLInputElement).checked ? 'Yes' : 'Off'} class="w-3 h-3 rounded text-blue-600" />
                    <span class="text-[10px] text-slate-600 dark:text-slate-400">Checked</span>
                 </label>
              {:else}
                <input 
                  id="field-{field.name}"
                  type="text" 
                  bind:value={fieldUpdates[field.name]} 
                  class="w-full p-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" 
                />
              {/if}
            </div>
          {/each}
        </div>

        <button 
          onclick={() => !pdfState.viewerFilePath ? pdfState.selectFile('extract') : handleSaveForm()} 
          class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-xl font-bold text-[10px] uppercase tracking-widest shadow-xl"
        >
          {!pdfState.viewerFilePath ? 'Select PDF' : 'Export Filled PDF'}
        </button>
      </div>
    {/if}
  </div>
</ToolPane>
