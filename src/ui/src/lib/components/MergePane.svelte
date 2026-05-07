<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  async function selectMergeFiles() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: true });
    if (result && result.length > 0) {
      pdfState.selectedMergeFiles = result;
      // Preview the first file
      pdfState.openTab(result[0]);
    }
  }

  let draggedIndex = $state<number | null>(null);

  function handleDragStart(index: number) { draggedIndex = index; }
  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;
    const newFiles = [...pdfState.selectedMergeFiles];
    const item = newFiles.splice(draggedIndex, 1)[0];
    newFiles.splice(index, 0, item);
    pdfState.selectedMergeFiles = newFiles;
    draggedIndex = index;
  }

  async function handleMerge() {
    if (pdfState.selectedMergeFiles.length < 2) { 
      appState.showStatus("Please select at least two PDF files to merge.", true); 
      return; 
    }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "merged.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Merging PDFs...");
    try {
      await invoke("merge_pdfs", { paths: pdfState.selectedMergeFiles, outputPath });
      appState.showStatus(`PDFs merged successfully.`, false, outputPath);
      pdfState.openTab(outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { 
      appState.showStatus(`Error merging PDFs: ${err}`, true); 
    }
  }
  function addCurrent() {
    if (pdfState.viewerFilePath && !pdfState.selectedMergeFiles.includes(pdfState.viewerFilePath)) {
      pdfState.selectedMergeFiles = [...pdfState.selectedMergeFiles, pdfState.viewerFilePath];
    }
  }
</script>

<ToolPane title="Merge">
  <div class="space-y-4">
    <div class="flex flex-col gap-2">
      {#if pdfState.viewerFilePath && !pdfState.selectedMergeFiles.includes(pdfState.viewerFilePath)}
        <button onclick={addCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
          Add Current to Merge
        </button>
      {/if}
      <button onclick={selectMergeFiles} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md font-medium transition-colors text-sm shadow-sm">Select PDF Files</button>
    </div>
    {#if pdfState.selectedMergeFiles.length > 0}
      <div class="space-y-2 max-h-60 overflow-y-auto rounded-lg border border-slate-200 dark:border-slate-800 p-2 bg-slate-50 dark:bg-slate-900/30 transition-colors shadow-inner">
        {#each pdfState.selectedMergeFiles as file, i}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div 
            draggable="true"
            ondragstart={() => handleDragStart(i)}
            ondragover={(e) => handleDragOver(e, i)}
            ondrop={() => draggedIndex = null}
            onclick={() => pdfState.openTab(file)}
            class="flex items-center gap-3 p-2 bg-white dark:bg-slate-800 rounded border border-slate-100 dark:border-slate-700 text-[11px] transition-all shadow-sm cursor-grab active:cursor-grabbing {draggedIndex === i ? 'opacity-50 scale-95 border-blue-500' : ''} {pdfState.viewerFilePath === file ? 'ring-1 ring-blue-500 bg-blue-50 dark:bg-blue-900/10' : ''}"
          >
            <span class="text-slate-400 w-4 font-mono">{i + 1}</span>
            <span class="truncate flex-1 font-medium text-slate-700 dark:text-slate-200 transition-colors">{file.split(/[/\\]/).pop()}</span>
            <button onclick={() => pdfState.selectedMergeFiles = pdfState.selectedMergeFiles.filter((_, idx) => idx !== i)} class="text-slate-400 hover:text-red-500 transition-colors">✕</button>
          </div>
        {/each}
      </div>
      <button onclick={() => pdfState.selectedMergeFiles = []} class="text-[10px] text-slate-400 hover:text-red-500 font-bold uppercase tracking-wider">Clear All</button>
    {/if}
    <button onclick={handleMerge} disabled={pdfState.selectedMergeFiles.length < 2} class="w-full mt-4 py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold disabled:opacity-30 disabled:cursor-not-allowed text-xs uppercase tracking-widest transition-all shadow-md">
      {pdfState.selectedMergeFiles.length < 2 ? 'Select 2+ PDFs' : 'Generate Merged PDF'}
    </button>
  </div>
</ToolPane>
