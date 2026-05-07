<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let snapshotLabel = $state("Initial Version");

  async function handleSaveSnapshot() {
    if (!pdfState.viewerFilePath) return;
    appState.startLoading("Saving document snapshot...");
    try {
      await pdfState.saveSnapshot(pdfState.viewerFilePath, snapshotLabel);
      appState.showStatus("Snapshot saved to local database.", false);
      snapshotLabel = `Version ${pdfState.versions.length + 1}`;
    } catch (e) {
      appState.showStatus("Failed to save snapshot.", true);
    }
  }

  async function handleRestore(version: any) {
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: `restored_${version.label}.pdf` });
    if (outputPath) {
      appState.startLoading("Restoring version...");
      try {
        await pdfState.restoreSnapshot(version, outputPath);
        // Automatically switch to the restored file
        pdfState.openTab(outputPath);
      } catch (e) {
        appState.showStatus("Restore failed.", true);
      }
    }
  }

  $effect(() => {
    if (pdfState.viewerFilePath) pdfState.loadVersions(pdfState.viewerFilePath);
  });
</script>

<ToolPane title="Versions" subtitle="Document Snapshots">
  <div class="space-y-8">
    <div class="space-y-4">
      <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Create Snapshot</h3>
      <div class="space-y-2">
        <label for="snap-label" class="text-[9px] font-bold text-slate-500 uppercase">Version Label</label>
        <input id="snap-label" type="text" bind:value={snapshotLabel} class="w-full p-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
        <button 
          onclick={handleSaveSnapshot} 
          disabled={!pdfState.viewerFilePath}
          class="w-full py-2 bg-blue-600 text-white rounded font-bold text-[10px] uppercase tracking-widest shadow-md"
        >
          Save Current State
        </button>
      </div>
    </div>

    <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
      <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Version History</h3>
      <div class="space-y-3">
        {#each pdfState.versions as version}
          <div class="p-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-xl shadow-sm group">
            <div class="flex justify-between items-center mb-1">
              <span class="text-xs font-bold text-slate-700 dark:text-slate-200">{version.label}</span>
              <span class="text-[8px] text-slate-400">{new Date(version.timestamp).toLocaleDateString()}</span>
            </div>
            <div class="flex gap-2 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
               <button onclick={() => handleRestore(version)} class="text-[9px] font-black text-blue-600 uppercase tracking-tighter">Restore to File</button>
            </div>
          </div>
        {/each}
        {#if pdfState.versions.length === 0}
          <div class="text-[10px] text-slate-400 italic text-center py-8">No snapshots saved for this document.</div>
        {/if}
      </div>
    </div>

    <div class="p-4 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-2xl shadow-xl">
       <div class="text-[8px] font-black uppercase opacity-60 mb-2">Version Control</div>
       <p class="text-[10px] font-medium leading-relaxed">Snapshots are stored in your private local database. Use them to save document states before redaction or metadata changes.</p>
    </div>
  </div>
</ToolPane>
