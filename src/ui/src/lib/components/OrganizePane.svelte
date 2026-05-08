<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { chatState } from "$lib/state/chatState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import { parsePageString } from "$lib/utils";
  import ToolPane from "./ToolPane.svelte";

  let autoRenameFiles = $state<{ path: string, originalName: string, newName: string, status: 'pending' | 'processing' | 'done' | 'error' }[]>([]);
  let isRenaming = $state(false);

  async function selectAutoRenameFiles() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: true });
    if (result && result.length > 0) {
      autoRenameFiles = result.map(p => ({
        path: p,
        originalName: p.split(/[/\\]/).pop() || p,
        newName: "",
        status: 'pending' as const
      }));
      pdfState.openTab(result[0]);
    }
  }

  async function handleAutoRename() {
    if (autoRenameFiles.length === 0) return;
    isRenaming = true;
    for (let i = 0; i < autoRenameFiles.length; i++) {
      const file = autoRenameFiles[i];
      if (file.status === 'done') continue;
      
      autoRenameFiles[i].status = 'processing';
      try {
        let pdfText = await invoke<string>("pdf_to_text_string", { path: file.path });
        if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000); 

        const system = "You are a file naming assistant. Based on the document text, suggest a concise, descriptive filename ending in .pdf. Use YYYY-MM-DD_Subject.pdf format if possible. Return ONLY the filename string.";
        const prompt = `TEXT FROM PDF:\n${pdfText}`;
        
        let newName = await chatState.runAiTask(system, prompt);
        newName = newName.trim().replace(/["']/g, "").replace(/[\/:*?"<>|]/g, "_");
        if (!newName.toLowerCase().endsWith(".pdf")) newName += ".pdf";
        
        const lastSlash = Math.max(file.path.lastIndexOf("/"), file.path.lastIndexOf("\\"));
        const oldDir = lastSlash !== -1 ? file.path.substring(0, lastSlash + 1) : "";
        const newPath = oldDir + newName;
        
        await invoke("rename_file", { oldPath: file.path, newPath });
        autoRenameFiles[i].newName = newName;
        autoRenameFiles[i].path = newPath;
        autoRenameFiles[i].status = 'done';
        // Open the newly renamed file if it's the current one
        if (pdfState.viewerFilePath === file.path) pdfState.openTab(newPath);
      } catch (e) {
        console.error(e);
        autoRenameFiles[i].status = 'error';
      }
    }
    isRenaming = false;
    appState.showStatus("Auto-rename process finished.", false);
  }

  async function handleAutoOutline() {
    if (!pdfState.selectedRotateFile) return;
    appState.startLoading("AI generating document structure...");
    try {
      const items = await chatState.generateOutline(pdfState.selectedRotateFile);
      if (items.length === 0) { appState.showStatus("AI could not determine structure.", true); return; }
      
      const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "structured.pdf" });
      if (!outputPath) return;
      
      await invoke("set_pdf_outline", { path: pdfState.selectedRotateFile, items, outputPath });
      appState.showStatus("Native Table of Contents generated.", false, outputPath);
      pdfState.openTab(outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) { appState.showStatus(`Auto-Outline failed: ${e}`, true); }
  }

  async function handleReorder(newOrder: number[]) {
    if (!pdfState.selectedRotateFile) return;
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "reordered.pdf" });
    if (!outputPath) return;
    appState.startLoading("Reordering pages...");
    try {
      await invoke("reorder_pages", { path: pdfState.selectedRotateFile, newOrder, outputPath });
      appState.showStatus("Pages reordered successfully.", false, outputPath);
      pdfState.openTab(outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      appState.showStatus(`Reorder failed: ${e}`, true);
    }
  }

  async function handleRotate(rotation: number) {
    if (!pdfState.selectedRotateFile) { appState.showStatus("Please select a PDF to rotate.", true); return; }
    const pagesArray = parsePageString(pdfState.rotatePagesInput);
    if (!pagesArray) { appState.showStatus("Invalid page format.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "rotated.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Rotating PDF...");
    try {
      await invoke("rotate_pdf", { path: pdfState.selectedRotateFile, pages: pagesArray, rotation, outputPath });
      appState.showStatus(`PDF rotated successfully.`, false, outputPath);
      pdfState.openTab(outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Error rotating PDF: ${err}`, true); }
  }

  async function handleDelete() {
    if (!pdfState.selectedDeleteFile) { appState.showStatus("Please select a PDF to delete pages from.", true); return; }
    const pagesArray = parsePageString(pdfState.deletePagesInput);
    if (!pagesArray || pagesArray.length === 0) { appState.showStatus("Invalid or empty page format.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "deleted.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Deleting pages...");
    try {
      await invoke("delete_pages", { path: pdfState.selectedDeleteFile, pagesToDelete: pagesArray, outputPath });
      appState.showStatus(`Pages deleted successfully.`, false, outputPath);
      pdfState.openTab(outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Error deleting pages: ${err}`, true); }
  }
  function organizeCurrent() {
    if (pdfState.viewerFilePath) {
      pdfState.setFileForTarget('organize', pdfState.viewerFilePath);
    }
  }
</script>

<ToolPane title="Organize">
  <div class="space-y-10">
    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-blue-600 dark:text-blue-400 uppercase tracking-widest transition-colors">AI Auto-Rename (Batch)</h3>
      <button onclick={selectAutoRenameFiles} class="w-full py-2 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-xs font-medium shadow-sm">Select Files to Rename</button>
      
      {#if autoRenameFiles.length > 0}
        <div class="max-h-40 overflow-y-auto space-y-2 border-t border-slate-200 dark:border-slate-800 pt-3">
          {#each autoRenameFiles as file}
            <div class="flex flex-col gap-0.5 p-2 rounded bg-white dark:bg-slate-800 border border-slate-100 dark:border-slate-700 shadow-sm">
              <span class="text-[9px] text-slate-400 truncate">{file.originalName}</span>
              {#if file.status === 'done'}
                <span class="text-[10px] text-green-500 font-bold truncate">→ {file.newName}</span>
              {:else if file.status === 'processing'}
                <span class="text-[9px] text-blue-500 animate-pulse">Renaming...</span>
              {:else if file.status === 'error'}
                <span class="text-[9px] text-red-500">Error renaming</span>
              {/if}
            </div>
          {/each}
        </div>
        <button onclick={handleAutoRename} disabled={isRenaming} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-xs uppercase tracking-widest transition-colors shadow-md shadow-blue-500/20">
          {autoRenameFiles.length === 0 ? 'Select Files' : isRenaming ? 'Renaming...' : 'Run AI Auto-Rename'}
        </button>
        <button onclick={() => autoRenameFiles = []} class="w-full text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter">Clear List</button>
      {/if}
    </div>

    <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
      <div class="flex flex-col gap-2">
        {#if pdfState.viewerFilePath && pdfState.selectedRotateFile !== pdfState.viewerFilePath}
          <button onclick={organizeCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
            Organize Current
          </button>
        {/if}
        <button onclick={() => pdfState.selectFile('rotate')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
          {pdfState.selectedRotateFile ? pdfState.selectedRotateFile.split(/[/\\]/).pop() : 'Select PDF to Rotate/Delete'}
        </button>
      </div>
      
      <div class="space-y-4">
        <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Visual & Structure</h3>
        <div class="flex gap-2">
          <button 
            onclick={() => { 
              if (!pdfState.selectedRotateFile) return; 
              pdfState.viewerFilePath = pdfState.selectedRotateFile; 
              pdfState.viewerMode = 'view';
              pdfState.viewerTarget = 'rotate';
            }}
            disabled={!pdfState.selectedRotateFile}
            class="flex-1 py-2 bg-blue-600 text-white rounded font-bold text-[10px] uppercase tracking-widest hover:bg-blue-700 transition-all shadow-md shadow-blue-500/20"
          >
            {!pdfState.selectedRotateFile ? 'Select PDF' : 'Reorder'}
          </button>
          <button 
            onclick={handleAutoOutline}
            disabled={!pdfState.selectedRotateFile}
            class="flex-1 py-2 border border-blue-600 text-blue-600 dark:text-blue-400 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all"
          >
            {!pdfState.selectedRotateFile ? 'Select PDF' : '✨ Outline'}
          </button>
        </div>
      </div>
      <div class="space-y-4 pt-6 border-t border-slate-100 dark:border-slate-900 transition-colors">
        <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Rotate Pages</h3>

        <input type="text" bind:value={pdfState.rotatePagesInput} placeholder="Pages (e.g. 1, 3-5)" class="w-full p-2 border border-slate-200 dark:border-slate-800 rounded text-xs bg-white dark:bg-slate-900 text-slate-900 dark:text-white transition-colors outline-none focus:ring-2 focus:ring-blue-500 shadow-sm" />
        <div class="flex gap-1.5">
          <button onclick={() => handleRotate(90)} disabled={!pdfState.selectedRotateFile} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">
            {!pdfState.selectedRotateFile ? 'Select PDF' : '90°'}
          </button>
          <button onclick={() => handleRotate(180)} disabled={!pdfState.selectedRotateFile} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">
            {!pdfState.selectedRotateFile ? 'Select PDF' : '180°'}
          </button>
          <button onclick={() => handleRotate(270)} disabled={!pdfState.selectedRotateFile} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">
            {!pdfState.selectedRotateFile ? 'Select PDF' : '270°'}
          </button>
        </div>
      </div>

      <div class="space-y-4 pt-6 border-t border-slate-100 dark:border-slate-900 transition-colors">
        <h3 class="text-[10px] font-bold text-red-500 dark:text-red-400 uppercase tracking-widest transition-colors">Delete Pages</h3>
        <input type="text" bind:value={pdfState.deletePagesInput} placeholder="Pages to remove" class="w-full p-2 border border-slate-200 dark:border-slate-800 rounded text-xs bg-white dark:bg-slate-900 text-slate-900 dark:text-white transition-colors outline-none focus:ring-2 focus:ring-red-500 shadow-sm" />
        <button onclick={handleDelete} disabled={!pdfState.selectedDeleteFile} class="w-full py-2 border border-red-200 dark:border-red-900 text-red-500 hover:bg-red-50 dark:hover:bg-red-950/20 rounded font-bold text-[10px] uppercase tracking-widest transition-all">
          {!pdfState.selectedDeleteFile ? 'Select PDF' : 'Remove Pages'}
        </button>
      </div>
    </div>
  </div>
</ToolPane>
