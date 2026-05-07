<script lang="ts">
  import { pdfState } from "$lib/state/pdfState.svelte";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    subtitle?: string;
    children: Snippet;
  }

  let { title, subtitle = "Operations Center", children }: Props = $props();
</script>

<aside class="flex flex-col h-full overflow-y-auto transition-colors duration-300 bg-white dark:bg-slate-950">
  <div class="p-6 border-b border-slate-100 dark:border-slate-900 transition-colors duration-300 flex justify-between items-start">
    <div>
      <h2 class="text-xl font-semibold capitalize tracking-tight text-slate-900 dark:text-white transition-colors duration-300">{title}</h2>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 font-medium tracking-tight uppercase tracking-widest opacity-60 transition-colors duration-300">{subtitle}</p>
    </div>
    <button 
      onclick={() => pdfState.activeTool = 'peek'}
      class="p-2 -mr-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors"
      title="Close Sidebar"
    >
      ✕
    </button>
  </div>

  <div class="flex-1 p-6 space-y-8 overflow-y-auto">
    {@render children()}
  </div>
</aside>
