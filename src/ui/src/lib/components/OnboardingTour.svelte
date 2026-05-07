<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly, scale } from "svelte/transition";
  import { browser } from "$app/environment";

  let step = $state(0);
  let isVisible = $state(false);

  const steps = [
    { 
      title: "Welcome to PDF Manager", 
      desc: "Your private, world-class intelligent document assistant. 100% on-device processing.",
      icon: "👋"
    },
    { 
      title: "The Power Palette", 
      desc: "Press Cmd+K anytime to open the Command Palette. Switch tools instantly without clicking.",
      icon: "⌨️"
    },
    { 
      title: "Knowledge Hub", 
      desc: "The Library stores your documents, metadata, and persistent AI chats safely in IndexedDB.",
      icon: "📚"
    },
    { 
      title: "Streaming Intelligence", 
      desc: "Chat with your PDF or entire library. Verified citations help you find facts instantly.",
      icon: "🧠"
    },
    { 
      title: "You're all set", 
      desc: "Explore professional security, manipulation, and analysis tools in the sidebar.",
      icon: "✨"
    }
  ];

  onMount(() => {
    if (browser) {
      const hasSeen = localStorage.getItem("onboarding_complete");
      if (!hasSeen) isVisible = true;
    }
  });

  function complete() {
    isVisible = false;
    localStorage.setItem("onboarding_complete", "true");
  }
</script>

{#if isVisible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-[400] flex items-center justify-center bg-slate-950/80 backdrop-blur-xl px-4"
    transition:fade={{ duration: 300 }}
  >
    <div 
      class="w-full max-w-sm bg-white dark:bg-slate-900 rounded-[2.5rem] shadow-2xl border border-slate-200 dark:border-slate-800 p-10 overflow-hidden text-center relative"
      transition:scale={{ start: 0.9, duration: 400 }}
    >
      <div class="mb-8">
        <div class="w-20 h-20 bg-blue-50 dark:bg-blue-900/30 rounded-3xl flex items-center justify-center mx-auto mb-6 text-4xl shadow-inner border border-blue-100 dark:border-blue-900/50 animate-bounce">
          {steps[step].icon}
        </div>
        <h2 class="text-2xl font-black text-slate-900 dark:text-white tracking-tight leading-tight">{steps[step].title}</h2>
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-4 leading-relaxed font-medium">{steps[step].desc}</p>
      </div>

      <div class="flex flex-col gap-3">
        {#if step < steps.length - 1}
          <button 
            onclick={() => step++}
            class="w-full py-4 bg-blue-600 text-white rounded-2xl font-black text-xs uppercase tracking-[0.2em] shadow-xl shadow-blue-500/20 hover:scale-[1.02] active:scale-95 transition-all"
          >
            Next Step
          </button>
        {:else}
          <button 
            onclick={complete}
            class="w-full py-4 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-2xl font-black text-xs uppercase tracking-[0.2em] shadow-xl hover:scale-[1.02] active:scale-95 transition-all"
          >
            Get Started
          </button>
        {/if}
        
        <button onclick={complete} class="text-[10px] font-black text-slate-400 uppercase tracking-widest hover:text-slate-600 dark:hover:text-slate-200 transition-colors">Skip Tour</button>
      </div>

      <div class="mt-8 flex justify-center gap-1.5">
        {#each steps as _, i}
          <div class="h-1 rounded-full transition-all {i === step ? 'w-6 bg-blue-600' : 'w-2 bg-slate-200 dark:bg-slate-800'}"></div>
        {/each}
      </div>
    </div>
  </div>
{/if}
