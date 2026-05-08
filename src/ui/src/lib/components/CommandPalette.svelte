<script lang="ts">
  import { onMount } from "svelte";
  import { pdfState, type ToolId } from "$lib/state/pdfState.svelte";
  import { fade, fly } from "svelte/transition";

  let isOpen = $state(false);
  let query = $state("");
  let selectedIndex = $state(0);

  const actions: { id: ToolId; label: string; icon: string; description: string }[] = [
    { id: "merge", label: "Merge PDFs", icon: "M", description: "Combine multiple PDF files into one." },
    { id: "split", label: "Split PDF", icon: "S", description: "Extract pages or ranges into a new file." },
    { id: "extract", label: "Extract & AI Chat", icon: "✨", description: "Analyze text or chat with your document." },
    { id: "annotate", label: "Annotate", icon: "A", description: "Add highlights, notes, or underlines." },
    { id: "signature", label: "Sign (Visual)", icon: "I", description: "Draw and apply a visual signature." },
    { id: "security", label: "Security & Redact", icon: "🔒", description: "Remove PII or digitally sign with PFX." },
    { id: "organize", label: "Organize", icon: "O", description: "Rotate, delete pages, or auto-rename." },
  ];

  let filteredActions = $derived(
    query === "" 
      ? actions 
      : actions.filter(a => 
          a.label.toLowerCase().includes(query.toLowerCase()) || 
          a.description.toLowerCase().includes(query.toLowerCase())
        )
  );

  let inputElement: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (isOpen && inputElement) {
      inputElement.focus();
    }
  });

  function close() {
    isOpen = false;
    query = "";
    selectedIndex = 0;
  }

  function executeAction(id: ToolId) {
    pdfState.switchTool(id);
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      isOpen = !isOpen;
    }
    if (!isOpen) return;

    if (e.key === "Escape") close();
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredActions.length;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredActions.length) % filteredActions.length;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (filteredActions[selectedIndex]) {
        executeAction(filteredActions[selectedIndex].id);
      }
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-[9000] flex items-start justify-center pt-[15vh] px-4 bg-slate-950/40 backdrop-blur-sm"
    onclick={close}
    transition:fade={{ duration: 150 }}
  >
    <div 
      class="w-full max-w-xl bg-white dark:bg-slate-900 rounded-2xl shadow-2xl border border-slate-200 dark:border-slate-800 overflow-hidden"
      onclick={e => e.stopPropagation()}
      transition:fly={{ y: -20, duration: 200 }}
    >
      <div class="flex items-center px-4 border-b border-slate-100 dark:border-slate-800">
        <span class="text-slate-400">🔍</span>
        <input 
          bind:this={inputElement}
          bind:value={query}
          placeholder="Type a command or search..."
          class="w-full p-4 bg-transparent outline-none text-slate-900 dark:text-white text-sm"
        />
        <kbd class="px-2 py-0.5 bg-slate-100 dark:bg-slate-800 rounded text-[10px] text-slate-500 font-mono">ESC</kbd>
      </div>

      <div class="max-h-[40vh] overflow-y-auto p-2">
        {#each filteredActions as action, i}
          <button
            class="w-full flex items-center gap-4 p-3 rounded-xl transition-colors {i === selectedIndex ? 'bg-blue-50 dark:bg-blue-900/40' : 'hover:bg-slate-50 dark:hover:bg-slate-800/50'}"
            onclick={() => executeAction(action.id)}
            onmouseenter={() => selectedIndex = i}
          >
            <div class="w-10 h-10 flex items-center justify-center rounded-lg bg-white dark:bg-slate-800 border border-slate-100 dark:border-slate-700 shadow-sm text-lg">
              {action.icon}
            </div>
            <div class="flex-1 text-left">
              <div class="text-sm font-bold text-slate-900 dark:text-white">{action.label}</div>
              <div class="text-[11px] text-slate-500 dark:text-slate-400">{action.description}</div>
            </div>
            {#if i === selectedIndex}
              <span class="text-xs text-blue-500 font-bold">↵</span>
            {/if}
          </button>
        {/each}

        {#if filteredActions.length === 0}
          <div class="p-8 text-center text-slate-500 text-sm italic">
            No commands found for "{query}"
          </div>
        {/if}
      </div>

      <div class="p-3 bg-slate-50 dark:bg-slate-950/50 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-[10px] text-slate-400 uppercase tracking-widest font-bold">
        <div class="flex gap-4">
          <span><span class="text-slate-500 dark:text-slate-300 mr-1">↑↓</span> Navigate</span>
          <span><span class="text-slate-500 dark:text-slate-300 mr-1">↵</span> Select</span>
        </div>
        <span>Command Palette</span>
      </div>
    </div>
  </div>
{/if}
