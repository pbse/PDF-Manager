<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let watermarkText = $state("CONFIDENTIAL");
  let watermarkOpacity = $state(0.3);
  let watermarkColor = $state("#ff0000");

  function parseColorHex(hex: string): [number, number, number] | null {
    const match = /^#?([a-fA-F0-9]{6})$/.exec(hex.trim());
    if (!match) return null;
    const intVal = parseInt(match[1], 16);
    return [((intVal >> 16) & 255) / 255, ((intVal >> 8) & 255) / 255, (intVal & 255) / 255];
  }

  async function handleAddWatermark() {
    if (!pdfState.viewerFilePath || !watermarkText) return;
    const colorArray = parseColorHex(watermarkColor);
    if (!colorArray) { appState.showStatus("Invalid color.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "watermarked.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Stamping watermark...");
    try {
      await invoke("add_watermark", {
        path: pdfState.viewerFilePath,
        text: watermarkText,
        opacity: watermarkOpacity,
        color: colorArray,
        outputPath
      });
      appState.showStatus("Watermark applied successfully.", false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      appState.showStatus(`Failed: ${e}`, true);
    }
  }
</script>

<ToolPane title="Watermark" subtitle="Pro Protection">
  <div class="space-y-6">
    <button 
      onclick={() => pdfState.selectFile('split')} 
      class="w-full py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors"
    >
      {pdfState.viewerFilePath ? pdfState.viewerFilePath.split(/[/\\]/).pop() : 'Select PDF'}
    </button>

    <div class="space-y-4">
      <div class="space-y-1">
        <label for="wm-text" class="text-[9px] font-bold text-slate-500 uppercase">Text</label>
        <input id="wm-text" type="text" bind:value={watermarkText} class="w-full p-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
      </div>

      <div class="space-y-1">
        <label for="wm-opacity" class="text-[9px] font-bold text-slate-500 uppercase">Opacity ({Math.round(watermarkOpacity * 100)}%)</label>
        <input id="wm-opacity" type="range" min="0.05" max="1" step="0.05" bind:value={watermarkOpacity} class="w-full h-1.5 bg-slate-200 rounded-lg appearance-none cursor-pointer" />
      </div>

      <div class="space-y-1">
        <label for="wm-color" class="text-[9px] font-bold text-slate-500 uppercase">Color</label>
        <div class="flex items-center gap-3">
          <input id="wm-color" type="color" bind:value={watermarkColor} class="w-10 h-10 p-0 border-0 bg-transparent cursor-pointer rounded-full overflow-hidden transition-colors shadow-sm" />
          <span class="text-[10px] font-mono uppercase text-slate-400 font-bold">{watermarkColor}</span>
        </div>
      </div>

      <button onclick={handleAddWatermark} disabled={!pdfState.viewerFilePath || !watermarkText} class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-xl font-bold text-[10px] uppercase tracking-widest shadow-xl hover:scale-[1.02] transition-all">
        {!pdfState.viewerFilePath ? 'Select PDF' : !watermarkText ? 'Enter Text' : 'Stamp Document'}
      </button>
    </div>

    <div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-2xl border border-blue-100 dark:border-blue-900/40">
       <div class="text-[8px] font-black uppercase text-blue-600 dark:text-blue-400 mb-1">Elite Tip</div>
       <p class="text-[10px] font-medium leading-relaxed">Watermarks are stamped diagonally across every page. Use low opacity (0.1 - 0.3) for the best results in professional distribution.</p>
    </div>
  </div>
</ToolPane>
