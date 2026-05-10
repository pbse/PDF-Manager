<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
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

  let makePermanent = $state(false);

  async function handleAnnotate() {
    if (!pdfState.selectedAnnotateFile) { appState.showStatus("Please select a PDF to annotate.", true); return; }
    if (!pdfState.viewerPageNumber || pdfState.viewerPageNumber <= 0) { appState.showStatus("Enter a valid page number.", true); return; }
    
    const colorArray = parseColorHex(pdfState.annotationColor);
    if (!colorArray) { appState.showStatus("Invalid color.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "annotated.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Adding annotation...");
    try {
      if (pdfState.annotationType === "ink") {
        if (pdfState.annotationStrokes.length === 0) {
          appState.showStatus("No drawings to apply.", true);
          return;
        }
        await invoke("add_ink_annotation", { 
          path: pdfState.selectedAnnotateFile, 
          page: pdfState.viewerPageNumber, 
          gestures: pdfState.annotationStrokes, 
          color: colorArray, 
          width: 2.0,
          outputPath 
        });
      } else {
        const rectArray = parseRect(pdfState.annotationRectInput);
        if (!rectArray) { appState.showStatus("Invalid rect selection.", true); return; }
        
        await invoke("add_annotation", { 
          path: pdfState.selectedAnnotateFile, 
          page: pdfState.viewerPageNumber, 
          rect: rectArray, 
          kind: pdfState.annotationType, 
          contents: pdfState.annotationText || null, 
          color: colorArray, 
          outputPath 
        });
      }
      
      if (makePermanent) {
        appState.startLoading("Burning annotation into content...");
        await invoke("flatten_annotations", { path: outputPath, outputPath: outputPath });
      }

      appState.showStatus(`Annotation added successfully ${makePermanent ? '(Permanent)' : ''}.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Error adding annotation: ${err}`, true); }
  }

  function openViewer(mode: "rect" | "points" | "view" = "rect") {
    if (!pdfState.selectedAnnotateFile) {
      appState.showStatus("Please select a PDF file first.", true);
      return;
    }
    pdfState.viewerFilePath = pdfState.selectedAnnotateFile;
    pdfState.viewerMode = mode;
    pdfState.viewerTarget = "annotate";
  }

  function toggleDrawingMode() {
    if (pdfState.annotationType === "ink") {
      pdfState.annotationType = "highlight";
      pdfState.viewerMode = "rect";
    } else {
      pdfState.annotationType = "ink";
      pdfState.viewerMode = "points";
    }
    if (pdfState.viewerTarget === 'annotate') {
       openViewer(pdfState.viewerMode);
    }
  }
  async function selectFile() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      const path = result[0];
      pdfState.setFileForTarget('annotate', path);
      pdfState.openTab(path);
      openViewer('rect');
    }
  }

  function annotateCurrent() {
    if (pdfState.viewerFilePath) {
      pdfState.setFileForTarget('annotate', pdfState.viewerFilePath);
      openViewer('rect');
    }
  }
</script>

<ToolPane title="Annotate">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      {#if pdfState.viewerFilePath && pdfState.selectedAnnotateFile !== pdfState.viewerFilePath}
        <button onclick={annotateCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
          Annotate Current
        </button>
      {:else if !pdfState.viewerFilePath && pdfState.selectedAnnotateFile}
        <button onclick={() => openViewer('rect')} class="w-full py-2 px-4 bg-green-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-green-700 uppercase tracking-tight">
          Open Viewer
        </button>
      {/if}
      <button onclick={selectFile} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
        {pdfState.selectedAnnotateFile ? pdfState.selectedAnnotateFile.split(/[/\\]/).pop() : 'Select PDF'}
      </button>
    </div>
    
    <div class="space-y-4">
      <div class="space-y-1.5">
        <label for="annotate-rect" class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Selection Area</label>
        <div class="flex gap-2">
          <input id="annotate-rect" type="text" bind:value={pdfState.annotationRectInput} class="flex-1 p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none font-mono transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" placeholder="x1, y1, x2, y2" />
          <button onclick={() => openViewer('rect')} class="p-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded border border-blue-100 dark:border-blue-900 hover:bg-blue-100 transition-colors">🎯</button>
        </div>
      </div>

      <div class="space-y-1.5">
        <label for="annotate-type" class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Type</label>
        <div class="flex gap-2">
          <select id="annotate-type" bind:value={pdfState.annotationType} onchange={() => {
            if (pdfState.annotationType === 'ink') {
              pdfState.viewerMode = 'points';
            } else {
              pdfState.viewerMode = 'rect';
            }
            if (pdfState.viewerTarget === 'annotate') openViewer(pdfState.viewerMode);
          }} class="flex-1 p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm">
            <option value="highlight">Highlight</option>
            <option value="underline">Underline</option>
            <option value="strikeout">Strikeout</option>
            <option value="note">Note</option>
            <option value="square">Square</option>
            <option value="circle">Circle</option>
            <option value="ink">Freehand (Ink)</option>
          </select>
          <button 
            onclick={toggleDrawingMode}
            class="p-2 border rounded transition-all {pdfState.annotationType === 'ink' ? 'bg-blue-600 text-white border-blue-600' : 'bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700 text-slate-400'}"
            title="Toggle Freehand Mode"
          >
            ✏️
          </button>
        </div>
      </div>

      <div class="space-y-1.5">
        <label for="annotate-color" class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Color</label>
        <div class="flex gap-2 items-center">
          <input id="annotate-color-hex" type="text" bind:value={pdfState.annotationColor} class="flex-1 p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm font-mono" />
          <input id="annotate-color" type="color" bind:value={pdfState.annotationColor} class="w-10 h-10 p-1 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded cursor-pointer" />
        </div>
      </div>

      <div class="space-y-1.5">
        <label for="annotate-content" class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Content</label>
        <input id="annotate-content" type="text" bind:value={pdfState.annotationText} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" />
      </div>

      <label class="flex items-center gap-2 cursor-pointer pt-2 group">
        <input type="checkbox" bind:checked={makePermanent} class="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 transition-all" />
        <span class="text-[10px] font-bold text-slate-500 group-hover:text-slate-700 dark:group-hover:text-slate-300 transition-colors uppercase tracking-tight">Make Permanent (Flatten)</span>
      </label>

      <button 
        onclick={() => {
          if (!pdfState.selectedAnnotateFile) {
            selectFile();
          } else if (pdfState.annotationType !== 'ink' && !pdfState.annotationRectInput) {
            openViewer('rect');
          } else {
            handleAnnotate();
          }
        }} 
        class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest shadow-lg transition-all hover:scale-[1.02]"
      >
        {!pdfState.selectedAnnotateFile 
          ? 'Select PDF' 
          : (pdfState.annotationType !== 'ink' && !pdfState.annotationRectInput) 
            ? 'Enter Selection Mode' 
            : 'Apply Annotation'}
      </button>
    </div>
  </div>
</ToolPane>
