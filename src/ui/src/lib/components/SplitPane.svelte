<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import { parsePageString } from "$lib/utils";
  import ToolPane from "./ToolPane.svelte";

  async function handleSplit() {
    if (!pdfState.selectedSplitFile) { appState.showStatus("Please select a PDF file to split.", true); return; }
    const pagesArray = parsePageString(pdfState.splitPagesInput);
    if (!pagesArray || pagesArray.length === 0) { appState.showStatus("Invalid or empty page range.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "split.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Splitting PDF...");
    try {
      await invoke("split_pdf", { path: pdfState.selectedSplitFile, pages: pagesArray, outputPath });
      appState.showStatus(`PDF split successfully.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { 
      appState.showStatus(`Error splitting PDF: ${err}`, true); 
    }
  }

  async function handleExtract() {
    if (!pdfState.selectedSplitFile) { appState.showStatus("Please select a PDF file first.", true); return; }
    if (!pdfState.viewerPageNumber || pdfState.viewerPageNumber <= 0) { appState.showStatus("Invalid page number.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: `page_${pdfState.viewerPageNumber}.pdf` });
    if (!outputPath) return;
    
    appState.startLoading("Extracting page...");
    try {
      await invoke("extract_pdf_page", { path: pdfState.selectedSplitFile, pageNumber: pdfState.viewerPageNumber, outputPath });
      appState.showStatus(`Page ${pdfState.viewerPageNumber} extracted successfully.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { 
      appState.showStatus(`Error extracting page: ${err}`, true); 
    }
  }
  function splitCurrent() {
    if (pdfState.viewerFilePath) {
      pdfState.setFileForTarget('split', pdfState.viewerFilePath);
    }
  }
</script>

<ToolPane title="Split">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      {#if pdfState.viewerFilePath && pdfState.selectedSplitFile !== pdfState.viewerFilePath}
        <button onclick={splitCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
          Split Current Document
        </button>
      {/if}
      <button onclick={() => pdfState.selectFile('split')} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:border-blue-500 dark:hover:border-blue-400 rounded-md transition-all text-sm font-medium truncate shadow-sm">
        {pdfState.selectedSplitFile ? pdfState.selectedSplitFile.split(/[/\\]/).pop() : 'Select PDF to Split'}
      </button>
    </div>
    
    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Extract Range</h3>
      <div class="space-y-1.5">
        <label for="split-pages" class="text-[10px] font-bold text-slate-500 uppercase tracking-tighter transition-colors">Page Numbers (e.g. 1, 3-5)</label>
        <input id="split-pages" type="text" bind:value={pdfState.splitPagesInput} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none transition-all shadow-sm" />
      </div>
      <button onclick={handleSplit} disabled={!pdfState.selectedSplitFile || !pdfState.splitPagesInput} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-xs uppercase tracking-widest transition-colors shadow-md shadow-blue-500/10">
        {!pdfState.selectedSplitFile ? 'Select PDF' : !pdfState.splitPagesInput ? 'Enter Range' : 'Save Range'}
      </button>
    </div>

    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Extract Current</h3>
      <button onclick={handleExtract} disabled={!pdfState.selectedSplitFile} class="w-full py-2 border border-blue-600 text-blue-600 dark:text-blue-400 rounded font-bold text-xs uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all">
        {!pdfState.selectedSplitFile ? 'Select PDF' : `Save Page ${pdfState.viewerPageNumber}`}
      </button>
    </div>
  </div>
</ToolPane>
