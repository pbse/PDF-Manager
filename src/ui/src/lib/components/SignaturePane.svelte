<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { chatState } from "$lib/state/chatState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  function parseRect(rectStr: string): number[] | null {
    const parts = rectStr.split(",").map((p) => p.trim()).filter((p) => p.length > 0);
    if (parts.length !== 4) return null;
    const nums = parts.map((p) => Number(p));
    if (nums.some((n) => Number.isNaN(n))) return null;
    return nums;
  }

  function parseColorHex(hex: string): [number, number, number] | null {
    const match = /^#?([a-fA-F0-9]{6})$/.exec(hex.trim());
    if (!match) return null;
    const intVal = parseInt(match[1], 16);
    return [((intVal >> 16) & 255) / 255, ((intVal >> 8) & 255) / 255, (intVal & 255) / 255];
  }

  let makePermanent = $state(true); // Default to true for signatures for better security

  async function handleSignatureVisual() {
    if (!pdfState.selectedSignatureFile) { appState.showStatus("Please select a PDF to sign.", true); return; }
    const rectArray = parseRect(pdfState.signatureRectInput);
    if (!rectArray || pdfState.signatureStrokes.length === 0) { appState.showStatus("Please draw your signature first.", true); return; }
    const colorArray = parseColorHex(pdfState.signatureColor);
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "signed.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Applying signature...");
    try {
      await invoke("add_signature_visual", { path: pdfState.selectedSignatureFile, page: pdfState.viewerPageNumber, rect: rectArray, strokes: pdfState.signatureStrokes, color: colorArray, width: pdfState.signatureWidth ?? 2, outputPath });
      
      if (makePermanent) {
        appState.startLoading("Burning signature into content...");
        await invoke("flatten_annotations", { path: outputPath, outputPath: outputPath });
      }

      appState.showStatus(`Signature applied successfully ${makePermanent ? '(Permanent)' : ''}.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Error signing: ${err}`, true); }
  }

  async function handleSecureShare() {
    const file = pdfState.selectedSignatureFile || pdfState.selectedCryptoFile;
    if (!file) return;
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: file });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);

      const system = "Draft a professional email sharing a signed document. Mention privacy. Return ONLY the Subject and Body.";
      const draft = await chatState.runAiTask(system, `TEXT:\n${pdfText}`);
      
      const subject = `Signed Document: ${file.split(/[/\\]/).pop()}`;
      await invoke("shell_open", { filePath: `mailto:?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(draft)}` });
      appState.showStatus("Opening mail client...", false);
    } catch (e) {
      console.error(e);
      appState.showStatus("Error sharing.", true);
    }
  }

  function openViewer(mode: "points" | "view" = "points") {
    if (!pdfState.selectedSignatureFile) {
      appState.showStatus("Please select a PDF file first.", true);
      return;
    }
    pdfState.viewerFilePath = pdfState.selectedSignatureFile;
    pdfState.viewerMode = mode;
    pdfState.viewerTarget = "signature";
  }
  async function selectFile() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      const path = result[0];
      pdfState.setFileForTarget('signature', path);
      pdfState.openTab(path);
      openViewer('points');
    }
  }

  function signCurrent() {
    if (pdfState.viewerFilePath) {
      pdfState.setFileForTarget('signature', pdfState.viewerFilePath);
      openViewer('points');
    }
  }
</script>

<ToolPane title="Sign">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      {#if pdfState.viewerFilePath && pdfState.selectedSignatureFile !== pdfState.viewerFilePath}
        <button onclick={signCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
          Sign Current
        </button>
      {/if}
      <button onclick={selectFile} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
        {pdfState.selectedSignatureFile ? pdfState.selectedSignatureFile.split(/[/\\]/).pop() : 'Select PDF'}
      </button>
    </div>
    
    <div class="space-y-4">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Drawing</h3>
      <div class="flex gap-2">
        <button onclick={() => openViewer('points')} class="flex-1 py-2 bg-blue-600 text-white rounded font-bold text-xs uppercase tracking-widest transition-opacity hover:opacity-90 shadow-md shadow-blue-500/20">Open Pad</button>
        <button onclick={() => pdfState.signatureStrokes = pdfState.signatureStrokes.slice(0, -1)} disabled={pdfState.signatureStrokes.length === 0} class="p-2 border border-slate-200 dark:border-slate-800 rounded hover:bg-slate-100 dark:hover:bg-slate-800 disabled:opacity-20 text-[10px] font-bold uppercase transition-colors text-slate-600 dark:text-slate-300">Undo</button>
        <button onclick={() => { pdfState.signatureStrokes = []; pdfState.signatureRectInput = ""; }} disabled={pdfState.signatureStrokes.length === 0} class="p-2 border border-red-200 dark:border-red-900 text-red-500 rounded text-[10px] font-bold uppercase transition-colors hover:bg-red-50 dark:hover:bg-red-950/20">Clear</button>
      </div>
      {#if pdfState.signatureStrokes.length > 0}
        <div class="flex items-center gap-2 text-green-600 dark:text-green-500 text-[10px] font-bold uppercase tracking-wider transition-colors">
          <span class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
          Signature Ready
        </div>
      {/if}
    </div>

    <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900 transition-colors">
      <div class="space-y-1.5">
        <label for="sig-color" class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Ink Color</label>
        <div class="flex items-center gap-3">
          <input id="sig-color" type="color" bind:value={pdfState.signatureColor} class="w-10 h-10 p-0 border-0 bg-transparent cursor-pointer rounded-full overflow-hidden transition-colors shadow-sm" />
          <span class="text-[10px] font-mono uppercase text-slate-400 dark:text-slate-500 font-bold transition-colors tracking-widest">{pdfState.signatureColor}</span>
        </div>
      </div>
      
      <label class="flex items-center gap-2 cursor-pointer pt-2 group">
        <input type="checkbox" bind:checked={makePermanent} class="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 transition-all" />
        <span class="text-[10px] font-bold text-slate-500 group-hover:text-slate-700 dark:group-hover:text-slate-300 transition-colors uppercase tracking-tight">Non-Deletable Signature (Flatten)</span>
      </label>

      <div class="flex gap-2">
        <button 
          onclick={() => !pdfState.selectedSignatureFile ? selectFile() : handleSignatureVisual()} 
          disabled={pdfState.selectedSignatureFile && pdfState.signatureStrokes.length === 0} 
          class="flex-[2] py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest transition-all shadow-xl hover:scale-[1.02]"
        >
          {!pdfState.selectedSignatureFile ? 'Select PDF' : pdfState.signatureStrokes.length === 0 ? 'Draw Signature' : 'Apply Signature'}
        </button>
        <button 
          onclick={() => !pdfState.selectedSignatureFile ? selectFile() : handleSecureShare()} 
          class="flex-1 py-3 border border-blue-600 text-blue-600 dark:text-blue-400 rounded-lg font-bold text-[10px] uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all shadow-md"
        >
          {!pdfState.selectedSignatureFile ? 'Select PDF' : 'Share'}
        </button>
      </div>
    </div>
  </div>
</ToolPane>
