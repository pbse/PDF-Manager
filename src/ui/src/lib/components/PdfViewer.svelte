<script lang="ts">
  import { onMount, createEventDispatcher, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { browser } from "$app/environment";

  export let filePath: string;
  export let pageNumber: number = 1;
  export let mode: "rect" | "points" = "rect";

  const dispatch = createEventDispatcher();

  let canvas: HTMLCanvasElement;
  let container: HTMLDivElement;
  let pdfjs: any = null;
  let pdfDoc: any = null;
  let viewport: any = null;
  let scale = 1.5;
  let loading = false;
  let error = "";

  // Selection state
  let isDrawing = false;
  let startX = 0;
  let startY = 0;
  let currentRect = { x1: 0, y1: 0, x2: 0, y2: 0 };
  let points: [number, number][] = [];

  // Guard variables to prevent infinite loops
  let lastLoadedPath = "";
  let lastRenderedPage = -1;
  let currentRenderTask: any = null;
  let isRendering = false;

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
      error = "Failed to initialize PDF.js: " + err.toString();
    }
  }

  async function loadDocument() {
    if (!pdfjs || !filePath || filePath === lastLoadedPath) return;
    loading = true;
    error = "";
    try {
      const bytes = await invoke<number[]>("read_file_bytes", { path: filePath });
      const uint8Array = new Uint8Array(bytes);
      const loadingTask = pdfjs.getDocument({ data: uint8Array });
      pdfDoc = await loadingTask.promise;
      lastLoadedPath = filePath;
      lastRenderedPage = -1; // Force re-render of current page
      await renderPage();
    } catch (err: any) {
      error = "Failed to load document: " + err.toString();
    } finally {
      loading = false;
    }
  }

  async function renderPage() {
    if (!pdfDoc || !canvas || (pageNumber === lastRenderedPage && !loading)) return;

    // If already rendering, cancel and wait for it to finish completely
    if (currentRenderTask) {
      try {
        currentRenderTask.cancel();
        // Crucial: Await the rejection of the previous task to ensure canvas is released
        await currentRenderTask.promise;
      } catch (e) {
        // Ignore expected cancellation error
      }
      currentRenderTask = null;
    }

    // Secondary lock to prevent race conditions during the await above
    if (isRendering) return;
    isRendering = true;

    try {
      await tick(); // Ensure DOM is settled
      const page = await pdfDoc.getPage(pageNumber);
      const context = canvas.getContext("2d");
      if (!context) {
        isRendering = false;
        return;
      }

      viewport = page.getViewport({ scale });
      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const renderContext = {
        canvasContext: context,
        viewport: viewport,
      };

      currentRenderTask = page.render(renderContext);
      await currentRenderTask.promise;

      lastRenderedPage = pageNumber;
    } catch (err: any) {
      if (err.name === "RenderingCancelledException") {
        // Normal cancellation, do nothing
      } else {
        error = "Rendering failed: " + err.toString();
      }
    } finally {
      currentRenderTask = null;
      isRendering = false;
    }
  }

  // --- Reactivity ---
  
  // 1. Initialize PDF.js once
  onMount(async () => {
    await initPdfJs();
  });

  // 2. Load document when filePath or pdfjs changes
  $: if (pdfjs && filePath !== lastLoadedPath) {
    loadDocument();
  }

  // 3. Render page when pageNumber or pdfDoc changes
  $: if (pdfDoc && (pageNumber !== lastRenderedPage)) {
    renderPage();
  }

  function handleMouseDown(e: MouseEvent) {
    if (loading || error || !viewport) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (mode === "rect") {
      isDrawing = true;
      startX = x;
      startY = y;
      currentRect = { x1: x, y1: y, x2: x, y2: y };
    } else {
      const [pdfX, pdfY] = viewport.convertToPdfPoint(x, y);
      points = [...points, [pdfX, pdfY]];
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDrawing) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    currentRect = { ...currentRect, x2: x, y2: y };
  }

  function handleMouseUp() {
    if (mode === "rect" && isDrawing) {
      isDrawing = false;
      // Convert to PDF coordinates
      const [pdfX1, pdfY1] = viewport.convertToPdfPoint(currentRect.x1, currentRect.y1);
      const [pdfX2, pdfY2] = viewport.convertToPdfPoint(currentRect.x2, currentRect.y2);
      
      // lopdf expects [x1, y1, x2, y2]
      // pdf.js convertToPdfPoint returns [x, y] in PDF space (0,0 is bottom-left usually)
      dispatch("select", { rect: [pdfX1, pdfY1, pdfX2, pdfY2] });
    }
  }

  function confirmPoints() {
    if (mode === "points" && points.length >= 2) {
      // For ink, we also need a bounding box. 
      // We can calculate it from points or just ask user to draw it.
      // Usually, the signature tool in this app asks for BOTH rect and points.
      // Let's simplify: the first 2 points define the rect roughly? No.
      // Let's just emit the points and let the user draw the rect later or vice-versa.
      dispatch("select", { points });
    }
  }

  function clear() {
    points = [];
    currentRect = { x1: 0, y1: 0, x2: 0, y2: 0 };
  }
</script>

<div class="viewer-container" bind:this={container}>
  {#if loading}
    <div class="overlay">Loading PDF...</div>
  {:else if error}
    <div class="overlay error">{error}</div>
  {/if}

  <div class="canvas-wrapper">
    <canvas
      bind:this={canvas}
      on:mousedown={handleMouseDown}
      on:mousemove={handleMouseMove}
      on:mouseup={handleMouseUp}
    ></canvas>

    {#if isDrawing && mode === "rect"}
      <div
        class="selection-rect"
        style="
          left: {Math.min(currentRect.x1, currentRect.x2)}px;
          top: {Math.min(currentRect.y1, currentRect.y2)}px;
          width: {Math.abs(currentRect.x2 - currentRect.x1)}px;
          height: {Math.abs(currentRect.y2 - currentRect.y1)}px;
        "
      ></div>
    {/if}

    {#if mode === "points"}
      {#each points as pt}
        <!-- We need to convert back to canvas pixels to show them -->
        {@const [cx, cy] = viewport ? viewport.convertToViewportPoint(pt[0], pt[1]) : [0,0]}
        <div class="point" style="left: {cx}px; top: {cy}px;"></div>
      {/each}
      
      {#if points.length > 0}
        <svg class="points-svg" width={canvas?.width} height={canvas?.height}>
           <polyline
            points={points.map(pt => {
              const [cx, cy] = viewport.convertToViewportPoint(pt[0], pt[1]);
              return `${cx},${cy}`;
            }).join(' ')}
            fill="none"
            stroke="red"
            stroke-width="2"
          />
        </svg>
      {/if}
    {/if}
  </div>

  <div class="viewer-controls">
    <span>Page {pageNumber}</span>
    <button on:click={clear}>Clear</button>
    {#if mode === "points"}
      <button on:click={confirmPoints} disabled={points.length < 2}>Confirm Points</button>
    {/if}
    <button on:click={() => dispatch('close')}>Close</button>
  </div>
</div>

<style>
  .viewer-container {
    position: relative;
    background: #1e1e1e;
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    max-height: 80vh;
    overflow: auto;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .canvas-wrapper {
    position: relative;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5);
    cursor: crosshair;
  }

  canvas {
    display: block;
    max-width: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0,0,0,0.5);
    z-index: 10;
    color: white;
  }

  .error {
    color: #ff4444;
  }

  .selection-rect {
    position: absolute;
    border: 2px solid #22d3ee;
    background: rgba(34, 211, 238, 0.2);
    pointer-events: none;
  }

  .point {
    position: absolute;
    width: 6px;
    height: 6px;
    background: red;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .points-svg {
    position: absolute;
    top: 0;
    left: 0;
    pointer-events: none;
  }

  .viewer-controls {
    margin-top: 1rem;
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  button {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(255, 255, 255, 0.1);
    color: white;
    cursor: pointer;
  }

  button:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
