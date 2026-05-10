<script lang="ts">
  import { db, type DocumentRecord, type EntityRecord } from "$lib/state/db";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { historyState } from "$lib/state/historyState.svelte";
  import ToolPane from "./ToolPane.svelte";
  import { onMount } from "svelte";

  let doc = $state<DocumentRecord | null>(null);
  let entities = $state<EntityRecord[]>([]);
  let relatedDocs = $state<DocumentRecord[]>([]);

  async function loadInsights() {
    if (!pdfState.viewerFilePath) return;
    
    doc = await db.documents.where('path').equals(pdfState.viewerFilePath).first() || null;
    entities = await db.entities.where('docPaths').equals(pdfState.viewerFilePath).toArray();
    relatedDocs = await historyState.getRelatedDocuments(pdfState.viewerFilePath);
  }

  $effect(() => {
    if (pdfState.viewerFilePath) loadInsights();
  });

  onMount(() => {
    loadInsights();
  });
</script>

<ToolPane title="Insights" subtitle="Proactive Intelligence">
  {#if !pdfState.viewerFilePath}
    <div class="flex flex-col items-center justify-center h-full text-center p-8 space-y-6">
      <div class="text-4xl animate-bounce">🔎</div>
      <div class="space-y-2">
        <p class="text-xs text-slate-400 font-bold uppercase tracking-widest">No Document Active</p>
        <p class="text-[10px] text-slate-400 font-medium leading-relaxed px-4">Open a document to see AI-powered summaries, extracted entities, and related knowledge instantly.</p>
      </div>
      <button 
        onclick={() => pdfState.openNewDocument()}
        class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-xl text-[10px] font-black uppercase tracking-widest transition-all shadow-lg shadow-blue-500/20"
      >
        Open Document
      </button>
    </div>
  {:else}
    <div class="space-y-8 flex flex-col h-full overflow-y-auto pr-1 no-scrollbar">
      <!-- Summary Section -->
      <div class="space-y-3">
        <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">Executive Summary</h3>
        <div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-2xl border border-blue-100 dark:border-blue-900/40 shadow-sm">
           <p class="text-[11px] font-medium text-slate-700 dark:text-blue-100 leading-relaxed italic">
             {doc?.summary || "Analyzing document structure..."}
           </p>
        </div>
      </div>

      <!-- Entities Section -->
      <div class="space-y-4">
        <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">Key Entities</h3>
        <div class="flex flex-wrap gap-2 px-1">
          {#each entities as ent}
            <div class="flex items-center gap-1.5 px-2 py-1 bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 shadow-sm">
               <div class="w-1.5 h-1.5 rounded-full {ent.type === 'org' ? 'bg-blue-500' : ent.type === 'date' ? 'bg-green-500' : 'bg-amber-500'}"></div>
               <span class="text-[10px] font-bold text-slate-600 dark:text-slate-300">{ent.name}</span>
            </div>
          {/each}
          {#if entities.length === 0}
            <div class="text-[10px] text-slate-400 italic">No entities mapped yet.</div>
          {/if}
        </div>
      </div>

      <!-- Related Docs Section -->
      <div class="space-y-4 flex-1">
        <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">Related Knowledge</h3>
        <div class="space-y-2">
          {#each relatedDocs as related}
             <!-- svelte-ignore a11y_click_events_have_key_events -->
             <!-- svelte-ignore a11y_no_static_element_interactions -->
             <div 
              onclick={() => pdfState.openTab(related.path)}
              class="group p-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-xl hover:border-blue-500 transition-all shadow-sm cursor-pointer"
             >
                <div class="text-[10px] font-bold text-slate-700 dark:text-slate-200 truncate">{related.name}</div>
                <div class="text-[8px] font-black uppercase text-slate-400 mt-1">Shared Entities: {entities.filter(e => e.docPaths.includes(related.path)).length}</div>
             </div>
          {/each}
          {#if relatedDocs.length === 0}
            <div class="text-[10px] text-slate-400 italic px-1">No direct relationships found in your library yet.</div>
          {/if}
        </div>
      </div>

      <div class="p-4 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-2xl shadow-xl shrink-0 mt-auto">
         <div class="text-[8px] font-black uppercase opacity-60 mb-2">Workspace context</div>
         <p class="text-[10px] font-medium leading-relaxed">The AI assistant and library chat are now automatically focused on this document's specific context.</p>
      </div>
    </div>
  {/if}
</ToolPane>
