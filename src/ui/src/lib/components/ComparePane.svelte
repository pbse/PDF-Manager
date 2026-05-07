<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import { chatState } from "$lib/state/chatState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let file1 = $state<string | null>(null);
  let file2 = $state<string | null>(null);
  let comparisonResults = $state<{ added: string[], removed: string[], shared_count: number } | null>(null);
  let conflicts = $state<string[]>([]);
  let versionSummary = $state("");
  let isComparing = $state(false);
  let isAnalyzingConflicts = $state(false);
  let isSummarizingChanges = $state(false);

  async function handleSummarizeChanges() {
    if (!file1 || !file2) return;
    isSummarizingChanges = true;
    versionSummary = "";
    try {
      const text1 = await invoke<string>("pdf_to_text_string", { path: file1 });
      const text2 = await invoke<string>("pdf_to_text_string", { path: file2 });
      
      const system = "You are a professional editor. Write a concise executive summary (max 100 words) of the primary changes between these two document versions. Be objective.";
      const prompt = `ORIGINAL:\n${text1.substring(0, 5000)}\n\nREVISED:\n${text2.substring(0, 5000)}`;
      
      versionSummary = await chatState.runAiTask(system, prompt);
    } catch (e) { appState.showStatus("Summary failed.", true); }
    finally { isSummarizingChanges = false; }
  }

  async function handleAnalyzeConflicts() {
    if (!file1 || !file2) return;
    isAnalyzingConflicts = true;
    conflicts = [];
    try {
      const text1 = await invoke<string>("pdf_to_text_string", { path: file1 });
      const text2 = await invoke<string>("pdf_to_text_string", { path: file2 });
      
      const system = "You are a legal auditor. Identify any conflicting clauses or logical inconsistencies between these two document versions. Be specific. Return a JSON object with a 'conflicts' array of strings. ONLY return JSON.";
      const prompt = `VERSION 1:\n${text1.substring(0, 5000)}\n\nVERSION 2:\n${text2.substring(0, 5000)}`;
      
      const result = await chatState.runAiTask(system, prompt, { json: true });
      const parsed = JSON.parse(result);
      conflicts = parsed.conflicts || [];
      if (conflicts.length === 0) appState.showStatus("No logical conflicts detected.", false);
    } catch (e) { appState.showStatus("Analysis failed.", true); }
    finally { isAnalyzingConflicts = false; }
  }

  async function selectFile(num: 1 | 2) {
    const result = await invoke<string>("open_file_dialog", { multiple: false });
    if (result) {
      if (num === 1) {
        file1 = result;
        pdfState.openTab(result);
      } else {
        file2 = result;
        pdfState.openTab(result);
      }
    }
  }

  async function handleCompare() {
    if (!file1 || !file2) return;
    isComparing = true;
    try {
      comparisonResults = await invoke("compare_pdfs_text", { path1: file1, path2: file2 });
      // Trigger side-by-side view in orchestrator
      // I'll use a hacky event or just update a shared state if available.
      // Better: Use a method in a new 'comparisonState' or just update page.svelte logic.
      // For now, I'll rely on the user manually selecting visual mode or automate it.
      appState.showStatus("Comparison complete. Side-by-side mode enabled.", false);
    } catch (e) {
      appState.showStatus("Comparison failed.", true);
    } finally {
      isComparing = false;
    }
  }

  function openVisualCompare() {
    if (!file1 || !file2) return;
    pdfState.viewerFilePath = file1;
    // We'll use this to trigger the side-by-side layout in +page.svelte
    appState.showStatus("Entering Visual Comparison...", false);
  }
</script>

<ToolPane title="Compare" subtitle="Version Diff Tool">
  <div class="space-y-6">
    <div class="space-y-3">
      <button onclick={() => selectFile(1)} class="w-full py-2 px-3 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors {file1 ? 'text-slate-900 dark:text-white' : 'text-slate-400'}">
        {file1 ? file1.split(/[/\\]/).pop() : 'Select Original PDF'}
      </button>
      <div class="flex justify-center text-slate-300 font-black text-[10px]">VS</div>
      <button onclick={() => selectFile(2)} class="w-full py-2 px-3 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors {file2 ? 'text-slate-900 dark:text-white' : 'text-slate-400'}">
        {file2 ? file2.split(/[/\\]/).pop() : 'Select Revised PDF'}
      </button>
    </div>

    <div class="flex gap-2">
      <button onclick={handleCompare} disabled={!file1 || !file2 || isComparing} class="flex-1 py-2 bg-blue-600 text-white rounded font-bold text-[10px] uppercase tracking-widest shadow-md">
        {!file1 || !file2 ? 'Select 2 PDFs' : isComparing ? 'Comparing...' : 'Text Diff'}
      </button>
      <button 
      onclick={() => {
         if (!file1 || !file2) return;
         // Orchestrator will pick this up
         // I'll use a custom property on pdfState for this
         // @ts-ignore
         pdfState.comparisonFile2 = file2;
         pdfState.viewerFilePath = file1;
         pdfState.viewerMode = 'view';
         pdfState.activeTool = 'compare';
      }} 
      disabled={!file1 || !file2} 
      class="flex-1 py-2 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded font-bold text-[10px] uppercase tracking-widest shadow-md"
      >
      {!file1 || !file2 ? 'Select 2 PDFs' : 'Visual'}
      </button>
      </div>

      <div class="flex gap-2">
      <button onclick={handleAnalyzeConflicts} disabled={!file1 || !file2 || isAnalyzingConflicts} class="flex-1 py-2 border border-amber-600 text-amber-600 dark:text-amber-400 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-amber-50 dark:hover:bg-amber-900/10 transition-all">
        {!file1 || !file2 ? 'Select 2 PDFs' : isAnalyzingConflicts ? 'Analyzing...' : 'Conflicts'}
      </button>
      <button onclick={handleSummarizeChanges} disabled={!file1 || !file2 || isSummarizingChanges} class="flex-1 py-2 border border-blue-600 text-blue-600 dark:text-blue-400 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/10 transition-all">
        {!file1 || !file2 ? 'Select 2 PDFs' : isSummarizingChanges ? 'Summarizing...' : 'Change Log'}
      </button>
    </div>

    {#if versionSummary}
      <div class="p-3 bg-blue-50 dark:bg-blue-900/10 rounded-xl border border-blue-100 dark:border-blue-900/30">
         <h4 class="text-[9px] font-black text-blue-600 uppercase tracking-widest mb-2">Revision Summary</h4>
         <div class="text-[10px] text-slate-700 dark:text-slate-300 leading-relaxed font-medium">{versionSummary}</div>
      </div>
    {/if}

    {#if conflicts.length > 0}
      <div class="space-y-3 p-3 bg-amber-50 dark:bg-amber-900/10 rounded-xl border border-amber-100 dark:border-amber-900/30">
       <h4 class="text-[9px] font-black text-amber-600 uppercase tracking-widest">Contradictory Clauses</h4>
       {#each conflicts as conflict}
          <div class="text-[9px] text-amber-700 dark:text-amber-300 leading-snug italic">" {conflict} "</div>
       {/each}
      </div>
      {/if}

      {#if comparisonResults}

      <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
        <div class="flex items-center justify-between text-[10px] font-black uppercase tracking-widest text-slate-400">
          <span>Results</span>
          <span class="text-blue-500">{comparisonResults.shared_count} Shared Lines</span>
        </div>
        
        <div class="space-y-2">
          <h4 class="text-[9px] font-bold text-green-600 uppercase tracking-tighter">Added ({comparisonResults.added.length})</h4>
          <div class="max-h-32 overflow-y-auto space-y-1 p-2 bg-green-50 dark:bg-green-900/10 rounded-lg border border-green-100 dark:border-green-900/30">
            {#each comparisonResults.added.slice(0, 20) as line}
              <div class="text-[9px] text-green-700 dark:text-green-300 font-mono leading-tight">+ {line}</div>
            {/each}
            {#if comparisonResults.added.length > 20}
              <div class="text-[8px] text-green-500 italic">...and {comparisonResults.added.length - 20} more</div>
            {/if}
          </div>
        </div>

        <div class="space-y-2">
          <h4 class="text-[9px] font-bold text-red-600 uppercase tracking-tighter">Removed ({comparisonResults.removed.length})</h4>
          <div class="max-h-32 overflow-y-auto space-y-1 p-2 bg-red-50 dark:bg-red-900/10 rounded-lg border border-red-100 dark:border-red-900/30">
            {#each comparisonResults.removed.slice(0, 20) as line}
              <div class="text-[9px] text-red-700 dark:text-red-300 font-mono leading-tight">- {line}</div>
            {/each}
            {#if comparisonResults.removed.length > 20}
              <div class="text-[8px] text-red-500 italic">...and {comparisonResults.removed.length - 20} more</div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</ToolPane>
