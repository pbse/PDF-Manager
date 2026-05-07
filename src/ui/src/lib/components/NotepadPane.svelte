<script lang="ts">
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import { db } from "$lib/state/db";
  import ToolPane from "./ToolPane.svelte";
  import { onMount } from "svelte";

  let newNoteText = $state("");

  async function handleAddManualNote() {
    if (!newNoteText.trim()) return;
    await pdfState.addNote(newNoteText);
    newNoteText = "";
  }

  function jumpToCitation(cit: any) {
    pdfState.openTab(cit.docPath);
    pdfState.viewerPageNumber = cit.pageNumber;
    pdfState.highlightedSnippet = cit.text;
  }

  onMount(() => {
    pdfState.loadNotes();
  });
</script>

<ToolPane title="Research" subtitle="Intelligent Notepad">
  <div class="space-y-8 flex flex-col h-full">
    <div class="space-y-3 shrink-0">
      <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">New Note</h3>
      <textarea 
        bind:value={newNoteText} 
        placeholder="Capture an insight..." 
        class="w-full p-3 bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl text-xs outline-none focus:ring-2 focus:ring-blue-500 min-h-[80px] resize-none shadow-inner"
      ></textarea>
      <button 
        onclick={handleAddManualNote} 
        disabled={!newNoteText.trim()}
        class="w-full py-2 bg-blue-600 text-white rounded-xl font-black text-[9px] uppercase tracking-[0.2em] shadow-lg shadow-blue-500/20 disabled:opacity-50"
      >
        Save Note
      </button>
    </div>

    <div class="flex-1 overflow-y-auto pr-1 space-y-4">
      <div class="flex items-center justify-between px-1">
        <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">History</h3>
        <span class="text-[9px] font-bold text-slate-400">{pdfState.notes.length} Total</span>
      </div>

      {#each pdfState.notes as note}
        <div class="p-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[1.5rem] shadow-sm group hover:border-blue-500 transition-all">
          <div class="text-[11px] text-slate-700 dark:text-slate-200 leading-relaxed font-medium">
            {note.content}
          </div>
          
          {#if note.citations && note.citations.length > 0}
            <div class="mt-3 pt-3 border-t border-slate-50 dark:border-slate-800 space-y-2">
               {#each note.citations as cit}
                  <button 
                    onclick={() => jumpToCitation(cit)}
                    class="w-full flex items-center gap-2 p-1.5 rounded-lg bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 hover:bg-blue-100 transition-all text-left"
                  >
                     <span class="text-[8px] font-black uppercase shrink-0 px-1 border border-blue-200 rounded">p.{cit.pageNumber}</span>
                     <span class="text-[9px] font-bold truncate italic">"{cit.text.substring(0, 40)}..."</span>
                  </button>
               {/each}
            </div>
          {/if}

          <div class="flex justify-between items-center mt-3">
             <span class="text-[8px] font-black text-slate-400 uppercase">{new Date(note.timestamp).toLocaleDateString()}</span>
             <button onclick={() => note.id && pdfState.deleteNote(note.id)} class="text-[8px] font-black text-slate-400 hover:text-red-500 uppercase tracking-tighter opacity-0 group-hover:opacity-100 transition-all">Discard</button>
          </div>
        </div>
      {/each}

      {#if pdfState.notes.length === 0}
         <div class="p-12 text-center text-slate-400 text-xs italic border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl">Collecting document insights will help you build a private knowledge repository.</div>
      {/if}
    </div>

    <div class="p-4 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-2xl shadow-xl shrink-0">
       <div class="text-[8px] font-black uppercase opacity-60 mb-2">Researcher Tip</div>
       <p class="text-[10px] font-medium leading-relaxed">Select text on any document and click "✨ Ask AI" or drag it here to create a cited note instantly.</p>
    </div>
  </div>
</ToolPane>
