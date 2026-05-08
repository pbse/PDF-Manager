<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState, type ToolId } from "$lib/state/pdfState.svelte";

  const intelligenceTools: { id: ToolId; label: string; icon: string }[] = [
    { id: "extract", label: "Assistant", icon: "✨" },
    { id: "compare", label: "Compare", icon: "⚖️" },
    { id: "notepad", label: "Notepad", icon: "📝" },
  ];

  const operationTools: { id: ToolId; label: string; icon: string }[] = [
    { id: "merge", label: "Merge", icon: "➕" },
    { id: "split", label: "Split", icon: "✂️" },
    { id: "annotate", label: "Annotate", icon: "✍️" },
    { id: "signature", label: "Sign", icon: "🖋️" },
    { id: "security", label: "Protect", icon: "🔒" },
    { id: "organize", label: "Organize", icon: "📂" },
    { id: "forms", label: "Forms", icon: "📋" },
    { id: "watermark", label: "Watermark", icon: "💧" },
  ];

  const systemTools: { id: ToolId; label: string; icon: string }[] = [
    { id: "library", label: "Library", icon: "📚" },
    { id: "versions", label: "Versions", icon: "🕒" },
  ];

  let isCollapsed = $state(false);
</script>

<nav class="flex flex-col items-center py-6 border-r border-slate-200 dark:border-slate-800 transition-all duration-500 bg-white dark:bg-slate-950 {isCollapsed ? 'w-16' : 'w-20'} shadow-xl z-50">
  <button 
    onclick={() => isCollapsed = !isCollapsed}
    class="mb-8 font-black text-blue-600 dark:text-blue-400 text-2xl tracking-tighter hover:scale-110 transition-transform flex items-center justify-center w-12 h-12 bg-blue-50 dark:bg-blue-900/20 rounded-2xl"
  >
    {isCollapsed ? 'P' : 'Pi'}
  </button>

  <div class="flex flex-col gap-6 w-full px-3 overflow-y-auto no-scrollbar pb-8">
    <!-- Intelligence Section -->
    <div class="flex flex-col gap-1.5">
      {#if !isCollapsed}
        <span class="text-[8px] font-black text-slate-400 uppercase tracking-widest px-2 mb-1">Intelligence</span>
      {/if}
      {#each intelligenceTools as tool}
        <button
          onclick={() => pdfState.activeTool === tool.id ? pdfState.activeTool = 'peek' : pdfState.switchTool(tool.id)}
          class="w-full aspect-square flex flex-col items-center justify-center rounded-xl transition-all duration-300 {pdfState.activeTool === tool.id ? 'bg-blue-600 text-white shadow-lg scale-105' : 'text-slate-500 hover:bg-blue-50 dark:hover:bg-slate-800 hover:text-blue-600 dark:hover:text-blue-400'}"
          title={tool.label}
          data-testid="tool-button-{tool.id}"
        >
          <span class="text-xl">{tool.icon}</span>
          {#if !isCollapsed}
            <span class="text-[8px] font-bold mt-1 uppercase tracking-tighter line-clamp-1">{tool.label}</span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Operations Section -->
    <div class="flex flex-col gap-1.5">
      {#if !isCollapsed}
        <span class="text-[8px] font-black text-slate-400 uppercase tracking-widest px-2 mb-1">Operations</span>
      {/if}
      {#each operationTools as tool}
        <button
          onclick={() => pdfState.activeTool === tool.id ? pdfState.activeTool = 'peek' : pdfState.switchTool(tool.id)}
          class="w-full aspect-square flex flex-col items-center justify-center rounded-xl transition-all duration-300 {pdfState.activeTool === tool.id ? 'bg-indigo-600 text-white shadow-lg scale-105' : 'text-slate-500 hover:bg-indigo-50 dark:hover:bg-slate-800 hover:text-indigo-600 dark:hover:text-indigo-400'}"
          title={tool.label}
          data-testid="tool-button-{tool.id}"
        >
          <span class="text-xl">{tool.icon}</span>
          {#if !isCollapsed}
            <span class="text-[8px] font-bold mt-1 uppercase tracking-tighter line-clamp-1">{tool.label}</span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Library Section -->
    <div class="flex flex-col gap-1.5">
      {#if !isCollapsed}
        <span class="text-[8px] font-black text-slate-400 uppercase tracking-widest px-2 mb-1">Vault</span>
      {/if}
      {#each systemTools as tool}
        <button
          onclick={() => pdfState.activeTool === tool.id ? pdfState.activeTool = 'peek' : pdfState.switchTool(tool.id)}
          class="w-full aspect-square flex flex-col items-center justify-center rounded-xl transition-all duration-300 {pdfState.activeTool === tool.id ? 'bg-emerald-600 text-white shadow-lg scale-105' : 'text-slate-500 hover:bg-emerald-50 dark:hover:bg-slate-800 hover:text-emerald-600 dark:hover:text-emerald-400'}"
          title={tool.label}
        >
          <span class="text-xl">{tool.icon}</span>
          {#if !isCollapsed}
            <span class="text-[8px] font-bold mt-1 uppercase tracking-tighter line-clamp-1">{tool.label}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <div class="mt-auto flex flex-col gap-4 border-t border-slate-100 dark:border-slate-900 pt-6 w-full items-center">
    <button 
      onclick={() => { 
        pdfState.activeTool = pdfState.activeTool === 'settings' ? 'peek' : 'settings'; 
      }} 
      class="p-2.5 rounded-xl text-slate-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-slate-800 transition-all {pdfState.activeTool === 'settings' ? 'text-blue-600 bg-blue-50' : ''}" 
      title="Settings"
    >
      ⚙️
    </button>
    <button onclick={() => invoke('shell_open', { filePath: 'https://github.com/sponsors/pbse' })} class="p-2.5 rounded-xl text-slate-400 hover:text-pink-500 hover:bg-pink-50 dark:hover:bg-slate-800 transition-all mb-4" title="Sponsor">
      ❤️
    </button>
  </div>
</nav>

<style>
  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
