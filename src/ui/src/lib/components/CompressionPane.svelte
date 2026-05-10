<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let selectedPreset = $state("web");
  let isCompressing = $state(false);

  const presets = [
    { id: "web", name: "Web Optimized", desc: "Best for sharing. High compression, standard quality.", icon: "🌐" },
    { id: "print", name: "Print Ready", desc: "Lossless. Removes unused data but keeps high-res assets.", icon: "🖨️" },
    { id: "minimal", name: "Minimal Size", desc: "Extreme compression. Removes all metadata and info.", icon: "📉" }
  ];

  async function handleCompress() {
    if (!pdfState.viewerFilePath) {
      appState.showStatus("Please select a PDF first.", true);
      return;
    }

    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "compressed_document.pdf" });
    if (!outputPath) return;

    isCompressing = true;
    appState.startLoading(`Compressing using ${selectedPreset} preset...`);
    try {
      await invoke("compress_pdf", { 
        path: pdfState.viewerFilePath, 
        outputPath, 
        preset: selectedPreset 
      });
      appState.showStatus("Compression completed successfully.", false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      appState.showStatus(`Compression failed: ${e}`, true);
    } finally {
      isCompressing = false;
    }
  }
</script>

<ToolPane title="Compress" subtitle="Optimize File Size">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      <button 
        onclick={() => pdfState.selectFile('security')} 
        class="w-full py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors"
      >
        {pdfState.viewerFilePath ? pdfState.viewerFilePath.split(/[/\\]/).pop() : 'Select PDF to Compress'}
      </button>
    </div>

    <div class="space-y-3">
      <label class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Select Optimization Preset</label>
      
      {#each presets as preset}
        <button 
          onclick={() => selectedPreset = preset.id}
          class="w-full p-4 text-left rounded-2xl border transition-all flex items-start gap-4 {selectedPreset === preset.id ? 'bg-blue-600 border-blue-600 text-white shadow-lg shadow-blue-500/20' : 'bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:border-blue-500'}"
        >
          <span class="text-2xl">{preset.icon}</span>
          <div class="flex flex-col">
            <span class="text-xs font-black uppercase tracking-widest">{preset.name}</span>
            <span class="text-[9px] mt-1 opacity-70 leading-relaxed font-medium">{preset.desc}</span>
          </div>
        </button>
      {/each}
    </div>

    <div class="pt-6 border-t border-slate-100 dark:border-slate-900">
      <button 
        onclick={handleCompress}
        disabled={isCompressing || !pdfState.viewerFilePath}
        class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-xl font-bold text-xs uppercase tracking-widest shadow-xl transition-all hover:scale-[1.02] active:scale-95 disabled:opacity-50"
      >
        {isCompressing ? 'Processing...' : 'Start Compression'}
      </button>
    </div>
  </div>
</ToolPane>
