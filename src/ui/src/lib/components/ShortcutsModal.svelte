<script lang="ts">
  import { fade, scale } from "svelte/transition";
  
  let { isOpen, onclose } = $props<{ isOpen: boolean, onclose: () => void }>();

  const shortcuts = [
    { key: "Cmd + K", description: "Open Command Palette" },
    { key: "Cmd + Z", description: "Undo (Signatures / Annotations)" },
    { key: "Cmd + Shift + Z", description: "Redo" },
    { key: "ESC", description: "Close Viewer / Palette" },
    { key: "← / →", description: "Previous / Next Page" },
    { key: "Enter", description: "Confirm Search / AI Chat" },
  ];
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-[300] flex items-center justify-center bg-slate-950/60 backdrop-blur-md px-4"
    onclick={onclose}
    transition:fade={{ duration: 200 }}
  >
    <div 
      class="w-full max-w-md bg-white dark:bg-slate-900 rounded-3xl shadow-2xl border border-slate-200 dark:border-slate-800 p-8 overflow-hidden"
      onclick={e => e.stopPropagation()}
      transition:scale={{ start: 0.9, duration: 300 }}
    >
      <div class="flex justify-between items-start mb-8">
        <div>
          <h2 class="text-2xl font-black text-slate-900 dark:text-white tracking-tight">Shortcuts</h2>
          <p class="text-xs text-slate-500 font-medium uppercase tracking-widest mt-1">Boost your productivity</p>
        </div>
        <button onclick={onclose} class="text-slate-400 hover:text-slate-900 dark:hover:text-white transition-colors">✕</button>
      </div>

      <div class="space-y-4">
        {#each shortcuts as s}
          <div class="flex items-center justify-between group">
            <span class="text-xs font-bold text-slate-500 dark:text-slate-400 group-hover:text-slate-900 dark:group-hover:text-white transition-colors uppercase tracking-tight">{s.description}</span>
            <kbd class="px-2 py-1 bg-slate-100 dark:bg-slate-800 rounded-lg text-[10px] font-black font-mono text-blue-600 dark:text-blue-400 shadow-sm border border-slate-200 dark:border-slate-700">{s.key}</kbd>
          </div>
        {/each}
      </div>

      <div class="mt-12 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-2xl border border-blue-100 dark:border-blue-900/40">
        <p class="text-[10px] text-blue-600 dark:text-blue-400 font-bold text-center leading-relaxed">Tip: Use Command Palette (Cmd+K) to find any feature instantly without touching your mouse.</p>
      </div>
    </div>
  </div>
{/if}
