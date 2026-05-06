<script lang="ts">
  import { onMount, tick, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { browser } from "$app/environment";
  import Tesseract from "tesseract.js";

  let {
    filePath = "",
    pageNumber = 1,
    mode = "view",
    previewRect = null,
    previewStrokes = [],
    previewColor = "red",
    ocrTrigger = 0,
    onselect,
    onclear,
    onclose,
    ondone,
    onprev,
    onnext
  } = $props<{
    filePath?: string;
    pageNumber?: number;
    mode?: "rect" | "points" | "view";
    previewRect?: number[] | null;
    previewStrokes?: [number, number][][];
    previewColor?: string;
    ocrTrigger?: number;
    onselect?: (detail: any) => void;
    onclear?: () => void;
    onclose?: () => void;
    ondone?: () => void;
    onprev?: () => void;
    onnext?: () => void;
  }>();

  let canvas: HTMLCanvasElement | undefined = $state();
  let container: HTMLDivElement | undefined = $state();
  let pdfjs: any = $state(null);
  let pdfDoc: any = $state(null);
  let viewport: any = $state(null);
  let scale = 1.5;
  let loading = $state(false);
  let error = $state("");
  let ocrProcessing = $state(false);

  let isDrawing = $state(false);
  let currentRect = $state({ x1: 0, y1: 0, x2: 0, y2: 0 });
  let strokes = $state<[number, number][][]>([]);
  let currentStroke = $state<[number, number][]>([]);

  let lastLoadedPath = "";
  let lastRenderedPage = -1;
  let currentRenderTask: any = null;
  let isRendering = false;

  $effect(() => {
    if (ocrTrigger > 0 && canvas && browser) {
      performOcr();
    }
  });

  async function performOcr() {
    if (!canvas) return;
    ocrProcessing = true;
    try {
      const dataUrl = canvas.toDataURL("image/png");
      const { data: { text } } = await Tesseract.recognize(dataUrl, "eng", {
        logger: m => console.log(m)
      });
      
      const defaultPath = `page_${pageNumber}_ocr.txt`;
      const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath });
      
      if (outputPath && text) {
        await invoke("write_text_file", { path: outputPath, contents: text });
        await invoke("shell_open", { filePath: outputPath });
      }
    } catch (e: any) {
      error = "OCR Failed: " + e.toString();
    } finally {
      ocrProcessing = false;
    }
  }

  async function initPdfJs() {
    if (!browser || pdfjs) return;
    try {
      // @ts-ignore
      pdfjs = await import("pdfjs-dist");
      pdfjs.GlobalWorkerOptions.workerSrc = new URL(
        "pdfjs-dist/build/pdf.worker.mjs",
        import.meta.url
      ).toString();
    } catch (err: any) {
      error = "Failed to initialize: " + err.toString();
    }
  }

  async function loadDocument() {
    if (!pdfjs || !filePath || filePath === lastLoadedPath) return;
    if (pdfDoc) { try { await pdfDoc.destroy(); } catch (e) {} pdfDoc = null; }
    loading = true; error = "";
    try {
      const bytes = await invoke<number[]>("read_file_bytes", { path: filePath });
      const uint8Array = new Uint8Array(bytes);
      const loadingTask = pdfjs.getDocument({ data: uint8Array });
      pdfDoc = await loadingTask.promise;
      lastLoadedPath = filePath; lastRenderedPage = -1;
      await renderPage();
    } catch (err: any) { error = "Load failed: " + err.toString(); } finally { loading = false; }
  }

  async function renderPage() {
    if (!pdfDoc || !canvas || (pageNumber === lastRenderedPage && !loading)) return;
    if (currentRenderTask) { try { currentRenderTask.cancel(); await currentRenderTask.promise; } catch (e) {} currentRenderTask = null; }
    if (isRendering) return;
    isRendering = true;
    try {
      await tick();
      const page = await pdfDoc.getPage(pageNumber);
      const context = canvas.getContext("2d");
      if (!context) { isRendering = false; return; }
      viewport = page.getViewport({ scale });
      canvas.height = viewport.height; canvas.width = viewport.width;
      currentRenderTask = page.render({ canvasContext: context, viewport });
      await currentRenderTask.promise;
      lastRenderedPage = pageNumber;
    } catch (err: any) { if (err.name !== "RenderingCancelledException") error = "Render failed: " + err.toString(); } finally { currentRenderTask = null; isRendering = false; }
  }

  onMount(async () => { await initPdfJs(); });
  $effect(() => { if (pdfjs && filePath !== lastLoadedPath) loadDocument(); });
  $effect(() => { if (pdfDoc && (pageNumber !== lastRenderedPage)) renderPage(); });
  $effect(() => { if (!isDrawing) strokes = [...previewStrokes]; });

  function handleMouseDown(e: MouseEvent) {
    if (loading || error || !viewport || !canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left; const y = e.clientY - rect.top;
    if (mode === "rect") { isDrawing = true; currentRect = { x1: x, y1: y, x2: x, y2: y }; }
    else if (mode === "points") { 
      isDrawing = true; 
      const [pdfX, pdfY] = viewport.convertToPdfPoint(x, y); 
      currentStroke = [[pdfX, pdfY]]; 
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDrawing || !canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left; const y = e.clientY - rect.top;
    if (mode === "rect") currentRect = { ...currentRect, x2: x, y2: y };
    else if (mode === "points") {
      const [pdfX, pdfY] = viewport.convertToPdfPoint(x, y);
      const last = currentStroke[currentStroke.length - 1];
      if (last) {
        const [lvx, lvy] = viewport.convertToViewportPoint(last[0], last[1]);
        if (Math.sqrt(Math.pow(x - lvx, 2) + Math.pow(y - lvy, 2)) > 2) currentStroke = [...currentStroke, [pdfX, pdfY]];
      }
    }
  }

  function handleMouseUp() {
    if (!isDrawing) return;
    isDrawing = false;
    if (mode === "rect" && viewport) {
      const [px1, py1] = viewport.convertToPdfPoint(currentRect.x1, currentRect.y1);
      const [px2, py2] = viewport.convertToPdfPoint(currentRect.x2, currentRect.y2);
      onselect?.({
        rect: [parseFloat(px1.toFixed(2)), parseFloat(py1.toFixed(2)), parseFloat(px2.toFixed(2)), parseFloat(py2.toFixed(2))]
      });
    } else if (mode === "points") {
      if (currentStroke.length > 0) {
        strokes = [...strokes, currentStroke];
        currentStroke = [];
        onselect?.({
          strokes: strokes.map(s => s.map(p => [parseFloat(p[0].toFixed(2)), parseFloat(p[1].toFixed(2))]))
        });
      }
    }
  }

  function clear() { strokes = []; currentStroke = []; currentRect = { x1: 0, y1: 0, x2: 0, y2: 0 }; onclear?.(); }
  function undo() { if (mode === "points" && strokes.length > 0) { strokes = strokes.slice(0, -1); onselect?.({ strokes: strokes.map(s => s.map(p => [parseFloat(p[0].toFixed(2)), parseFloat(p[1].toFixed(2))])) }); } }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose?.();
    else if (e.key === "z" && (e.ctrlKey || e.metaKey)) undo();
  }

  onDestroy(async () => { if (pdfDoc) try { await pdfDoc.destroy(); } catch (e) {} });
</script>

<div class="relative bg-white dark:bg-slate-900 rounded-xl shadow-2xl border border-slate-300 dark:border-slate-800 flex flex-col outline-none overflow-hidden max-h-[85vh] transition-colors duration-300" bind:this={container} onkeydown={handleKeyDown} tabindex="0">
  {#if loading || ocrProcessing}
    <div class="absolute inset-0 z-50 flex items-center justify-center bg-white/90 dark:bg-slate-900/80 backdrop-blur-sm transition-colors duration-300">
      <div class="flex flex-col items-center gap-3">
        <div class="w-10 h-10 border-4 border-blue-100 dark:border-blue-900 border-t-blue-600 rounded-full animate-spin"></div>
        <span class="text-sm font-semibold text-slate-700 dark:text-slate-400 tracking-tight">
          {ocrProcessing ? "Performing Local OCR..." : "Optimizing view..."}
        </span>
      </div>
    </div>
  {:else if error}
    <div class="absolute inset-0 z-50 flex items-center justify-center bg-red-50 dark:bg-red-950/20 text-red-700 dark:text-red-400 p-8 text-center font-bold">{error}</div>
  {/if}

  <div class="relative mx-auto p-8 overflow-auto transition-colors duration-300">
    <canvas bind:this={canvas} onmousedown={handleMouseDown} onmousemove={handleMouseMove} onmouseup={handleMouseUp} onmouseleave={handleMouseUp} class="bg-white shadow-md ring-1 ring-slate-300 dark:ring-slate-700 rounded-sm cursor-crosshair max-w-full transition-all duration-300"></canvas>

    {#if isDrawing && mode === "rect"}
      <div class="absolute border-2 border-blue-600 bg-blue-600/20 pointer-events-none transition-all z-20" style="left: {Math.min(currentRect.x1, currentRect.x2)+32}px; top: {Math.min(currentRect.y1, currentRect.y2)+32}px; width: {Math.abs(currentRect.x2 - currentRect.x1)}px; height: {Math.abs(currentRect.y2 - currentRect.y1)}px;"></div>
    {/if}

    {#if previewRect && viewport && !isDrawing}
      {@const [vx1, vy1] = viewport.convertToViewportPoint(previewRect[0], previewRect[1])}
      {@const [vx2, vy2] = viewport.convertToViewportPoint(previewRect[2], previewRect[3])}
      <div class="absolute border-2 pointer-events-none z-10 rounded-sm transition-colors duration-300" style="left: {Math.min(vx1, vx2)+32}px; top: {Math.min(vy1, vy2)+32}px; width: {Math.abs(vx2 - vx1)}px; height: {Math.abs(vy2 - vy1)}px; border-color: {previewColor}; background: {previewColor}33;"></div>
    {/if}

    {#if viewport}
      <svg class="absolute top-0 left-0 pointer-events-none p-8 z-10" width={canvas?.width ? canvas.width + 64 : 64} height={canvas?.height ? canvas.height + 64 : 64}>
        {#if mode === "points"}
          {#each strokes as stroke}
            <polyline points={stroke.map(pt => { const [cx, cy] = viewport.convertToViewportPoint(pt[0], pt[1]); return `${cx},${cy}`; }).join(' ')} fill="none" stroke={previewColor} stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
          {/each}
          {#if currentStroke.length > 0}
            <polyline points={currentStroke.map(pt => { const [cx, cy] = viewport.convertToViewportPoint(pt[0], pt[1]); return `${cx},${cy}`; }).join(' ')} fill="none" stroke={previewColor} stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
          {/if}
        {:else if previewStrokes.length > 0 && !isDrawing}
          {#each previewStrokes as stroke}
            <polyline points={stroke.map((pt: [number, number]) => { const [cx, cy] = viewport.convertToViewportPoint(pt[0], pt[1]); return `${cx},${cy}`; }).join(' ')} fill="none" stroke={previewColor} stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
          {/each}
        {/if}
      </svg>
    {/if}
  </div>

  <div class="shrink-0 px-8 py-4 bg-slate-100 dark:bg-slate-900/50 border-t border-slate-300 dark:border-slate-800 flex items-center justify-between transition-colors duration-300">
    <div class="flex items-center gap-1 bg-white dark:bg-slate-800 rounded-lg border border-slate-300 dark:border-slate-700 p-1 shadow-sm transition-colors duration-300">
       <button onclick={onprev} disabled={pageNumber <= 1} class="p-1.5 hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-300 rounded-md disabled:opacity-20 transition-colors">◀</button>
       <span class="px-3 text-xs font-bold text-slate-800 dark:text-slate-300 min-w-[4rem] text-center uppercase tracking-widest transition-colors">Page {pageNumber}</span>
       <button onclick={onnext} class="p-1.5 hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-300 rounded-md transition-colors">▶</button>
    </div>
    
    <div class="flex items-center gap-2">
       {#if mode === 'points'}
         <button onclick={undo} disabled={strokes.length === 0} class="px-4 py-1.5 text-xs font-bold text-slate-700 dark:text-slate-300 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 rounded-lg transition-all disabled:opacity-20 shadow-sm transition-colors duration-300 uppercase tracking-tighter">Undo Stroke</button>
       {/if}
       <button onclick={clear} class="px-4 py-1.5 text-xs font-bold text-slate-700 dark:text-slate-300 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 hover:text-red-600 rounded-lg transition-all shadow-sm transition-colors duration-300 uppercase tracking-tighter">Reset</button>
       <div class="w-[1px] h-6 bg-slate-300 dark:border-slate-700 mx-1 transition-colors duration-300"></div>
       {#if mode !== 'view'}
         <button onclick={ondone} class="px-5 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-xs font-bold rounded-lg transition-all shadow-md shadow-blue-500/20 uppercase tracking-tight">Lock & Finish</button>
       {:else}
         <button onclick={onclose} class="px-5 py-1.5 bg-slate-900 dark:bg-white text-white dark:text-slate-900 text-xs font-bold rounded-lg transition-all shadow-lg uppercase tracking-tight transition-colors duration-300">Close Preview</button>
       {/if}
    </div>
  </div>
</div>

<style>
  /* Local component styles - minimalist approach */
</style>
