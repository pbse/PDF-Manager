<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';

  let { children } = $props();

  let isDark = $state(false);

  function toggleDarkMode() {
    isDark = !isDark;
    if (isDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }

  onMount(() => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'dark' || (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      isDark = true;
      document.documentElement.classList.add('dark');
    } else {
      isDark = false;
      document.documentElement.classList.remove('dark');
    }
  });
</script>

<div class="min-h-screen bg-white dark:bg-slate-950 text-slate-900 dark:text-slate-100 flex flex-col font-sans transition-colors duration-300">
  {@render children()}
  
  <button 
    onclick={toggleDarkMode}
    class="fixed bottom-6 right-6 p-3 rounded-full bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-all z-50 group outline-none"
    title="Toggle Dark Mode"
  >
    {#if isDark}
      <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 text-yellow-400 group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 18v1m9-9h1M3 9h1m15.364-6.364l-.707.707M6.343 17.657l-.707.707m12.728 0l-.707-.707M6.343 6.343l-.707-.707M12 5a7 7 0 100 14 7 7 0 000-14z" />
      </svg>
    {:else}
      <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 text-slate-700 group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
      </svg>
    {/if}
  </button>
</div>

<style>
  :global(html) {
    scroll-behavior: smooth;
  }
</style>
