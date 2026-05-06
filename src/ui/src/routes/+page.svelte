<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount, tick } from "svelte";

  // Local imports
  import StatusDisplay from "$lib/components/StatusDisplay.svelte";
  import PdfViewer from "$lib/components/PdfViewer.svelte";
  import { parsePageString } from "$lib/utils";
  import { isLoading, showStatus, startLoading } from "$lib/stores";

  // --- Reactive State ---
  type SelectionTarget =
    | "parse"
    | "split"
    | "rotate"
    | "delete"
    | "annotate"
    | "signature"
    | "crypto";

  let selectedParseFile = $state<string | null>(null);
  let selectedSplitFile = $state<string | null>(null);
  let selectedRotateFile = $state<string | null>(null);
  let selectedDeleteFile = $state<string | null>(null);
  let selectedAnnotateFile = $state<string | null>(null);
  let selectedSignatureFile = $state<string | null>(null);
  let selectedCryptoFile = $state<string | null>(null);
  let parseResult = $state<Record<string, string> | null>(null);
  let selectedMergeFiles = $state<string[]>([]);
  let splitPagesInput = $state("");
  let rotatePagesInput = $state("");
  let deletePagesInput = $state("");
  let annotationRectInput = $state("");
  let annotationType = $state<"highlight" | "underline" | "strikeout" | "note">("highlight");
  let annotationText = $state("");
  let annotationColor = $state("#facc15");
  let signatureRectInput = $state("");
  let signatureColor = $state("#0f172a");
  let signatureWidth = $state<number | null>(2);
  let signatureStrokes = $state<[number, number][][]>([]);
  let signCertPath = $state("");
  let signCertPassword = $state("");
  let rememberPassword = $state(false);
  let signRectInput = $state("");
  let signReason = $state("");
  let signLocation = $state("");
  let signContact = $state("");

  // --- Active Tool State ---
  type ToolId = "merge" | "annotate" | "signature" | "crypto" | "split" | "organize";
  let activeTool = $state<ToolId>("merge");

  function switchTool(id: ToolId) {
    if (id === activeTool) return;
    
    activeTool = id;
    viewerFilePath = ""; // Clear right pane
    viewerPageNumber = 1;
    
    // Reset all tool-specific selections to force "reselection"
    selectedParseFile = null;
    selectedSplitFile = null;
    selectedRotateFile = null;
    selectedDeleteFile = null;
    selectedAnnotateFile = null;
    selectedSignatureFile = null;
    selectedCryptoFile = null;
    selectedMergeFiles = [];
    
    // Also clear secondary inputs
    signatureStrokes = [];
    annotationRectInput = "";
    signatureRectInput = "";
    signRectInput = "";
    splitPagesInput = "";
    rotatePagesInput = "";
    deletePagesInput = "";
    
    showStatus(`Switched to ${id.toUpperCase()}`, false);
  }

  // --- Viewer State ---
  let viewerFilePath = $state("");
  let viewerPageNumber = $state(1);
  let viewerMode = $state<"rect" | "points" | "view">("view");
  let viewerTarget = $state<SelectionTarget | null>(null);

  let currentPreviewRect = $derived(
    activeTool === "annotate"
      ? parseRect(annotationRectInput)
      : activeTool === "crypto"
        ? parseRect(signRectInput)
        : activeTool === "signature"
          ? parseRect(signatureRectInput)
          : null
  );

  let currentPreviewStrokes = $derived(activeTool === "signature" ? signatureStrokes : []);

  let currentPreviewColor = $derived(
    activeTool === "annotate"
      ? annotationColor
      : activeTool === "signature"
        ? signatureColor
        : activeTool === "crypto"
          ? "#3b82f6"
          : "#6366f1"
  );

  function openViewer(target: SelectionTarget, mode: "rect" | "points" | "view" = "rect") {
    let path = "";
    switch (target) {
      case "annotate": path = selectedAnnotateFile || ""; break;
      case "signature": path = selectedSignatureFile || ""; break;
      case "crypto": path = selectedCryptoFile || ""; break;
      case "split": path = selectedSplitFile || ""; break;
      case "rotate": path = selectedRotateFile || ""; break;
      case "delete": path = selectedDeleteFile || ""; break;
    }

    if (!path) {
      showStatus("Please select a PDF file first.", true);
      return;
    }

    viewerFilePath = path;
    viewerMode = mode;
    viewerTarget = target;

    tick().then(() => {
      const v = document.querySelector(".viewer-container") as HTMLElement;
      v?.focus();
    });
  }

  function handleViewerSelect(event: any) {
    const { rect, strokes: selectedStrokes } = event;

    if (rect) {
      const rectStr = rect.map((n: number) => Math.round(n)).join(", ");
      switch (viewerTarget) {
        case "annotate": annotationRectInput = rectStr; break;
        case "signature": signatureRectInput = rectStr; break;
        case "crypto": signRectInput = rectStr; break;
      }
    }

    if (selectedStrokes) {
      if (viewerTarget === "signature") {
        signatureStrokes = selectedStrokes;
        if (signatureStrokes.length > 0) {
          const allPoints = signatureStrokes.flat();
          const xs = allPoints.map((p: [number, number]) => p[0]);
          const ys = allPoints.map((p: [number, number]) => p[1]);
          const minX = Math.min(...xs) - 5;
          const minY = Math.min(...ys) - 5;
          const maxX = Math.max(...xs) + 5;
          const maxY = Math.max(...ys) + 5;
          signatureRectInput = `${Math.round(minX)}, ${Math.round(minY)}, ${Math.round(maxX)}, ${Math.round(maxY)}`;
        }
      }
    }
  }

  // --- Helper functions ---

  async function openFileDialog(multiple = false): Promise<string | string[] | null> {
    const result = await invoke("open_file_dialog", { multiple });
    if (Array.isArray(result)) return multiple ? result : result[0] || null;
    return null;
  }

  async function saveFileDialog(defaultPath: string): Promise<string | null> {
    return await invoke("save_file_dialog", { defaultPath });
  }

  async function shellOpen(filePath: string): Promise<void> {
    await invoke("shell_open", { filePath });
  }

  async function openPathInExplorer(filePath: string): Promise<void> {
    try { await shellOpen(filePath); } catch (err) { showStatus(`Error opening file: ${err}`, true); }
  }

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

  // --- Handlers ---

  async function handleMerge() {
    if (selectedMergeFiles.length < 2) { showStatus("Please select at least two PDF files to merge.", true); return; }
    const outputPath = await getSavePath("merged.pdf");
    if (!outputPath) return;
    startLoading("Merging PDFs...");
    try {
      await invoke("merge_pdfs", { paths: selectedMergeFiles, outputPath });
      showStatus(`PDFs merged successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error merging PDFs: ${err}`, true); }
  }

  async function handleSplit() {
    if (!selectedSplitFile) { showStatus("Please select a PDF file to split.", true); return; }
    const pagesArray = parsePageString(splitPagesInput);
    if (!pagesArray || pagesArray.length === 0) { showStatus("Invalid or empty page range.", true); return; }
    const outputPath = await getSavePath("split.pdf");
    if (!outputPath) return;
    startLoading("Splitting PDF...");
    try {
      await invoke("split_pdf", { path: selectedSplitFile, pages: pagesArray, outputPath });
      showStatus(`PDF split successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error splitting PDF: ${err}`, true); }
  }

  async function handleExtract() {
    if (!selectedSplitFile) { showStatus("Please select a PDF file first.", true); return; }
    if (!viewerPageNumber || viewerPageNumber <= 0) { showStatus("Invalid page number.", true); return; }
    const outputPath = await getSavePath(`page_${viewerPageNumber}.pdf`);
    if (!outputPath) return;
    startLoading("Extracting page...");
    try {
      await invoke("extract_pdf_page", { path: selectedSplitFile, pageNumber: viewerPageNumber, outputPath });
      showStatus(`Page ${viewerPageNumber} extracted successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error extracting page: ${err}`, true); }
  }

  async function handleRotate(rotation: number) {
    if (!selectedRotateFile) { showStatus("Please select a PDF to rotate.", true); return; }
    const pagesArray = parsePageString(rotatePagesInput);
    if (!pagesArray) { showStatus("Invalid page format.", true); return; }
    const outputPath = await getSavePath("rotated.pdf");
    if (!outputPath) return;
    startLoading("Rotating PDF...");
    try {
      await invoke("rotate_pdf", { path: selectedRotateFile, pages: pagesArray, rotation, outputPath });
      showStatus(`PDF rotated successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error rotating PDF: ${err}`, true); }
  }

  async function handleDelete() {
    if (!selectedDeleteFile) { showStatus("Please select a PDF to delete pages from.", true); return; }
    const pagesArray = parsePageString(deletePagesInput);
    if (!pagesArray || pagesArray.length === 0) { showStatus("Invalid or empty page format.", true); return; }
    const outputPath = await getSavePath("deleted.pdf");
    if (!outputPath) return;
    startLoading("Deleting pages...");
    try {
      await invoke("delete_pages", { path: selectedDeleteFile, pagesToDelete: pagesArray, outputPath });
      showStatus(`Pages deleted successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error deleting pages: ${err}`, true); }
  }

  async function handleAnnotate() {
    if (!selectedAnnotateFile) { showStatus("Please select a PDF to annotate.", true); return; }
    if (!viewerPageNumber || viewerPageNumber <= 0) { showStatus("Enter a valid page number.", true); return; }
    const rectArray = parseRect(annotationRectInput);
    if (!rectArray) { showStatus("Invalid rect selection.", true); return; }
    const colorArray = parseColorHex(annotationColor);
    if (!colorArray) { showStatus("Invalid color.", true); return; }
    const outputPath = await getSavePath("annotated.pdf");
    if (!outputPath) return;
    startLoading("Adding annotation...");
    try {
      await invoke("add_annotation", { path: selectedAnnotateFile, page: viewerPageNumber, rect: rectArray, kind: annotationType, contents: annotationText || null, color: colorArray, outputPath });
      showStatus(`Annotation added successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error adding annotation: ${err}`, true); }
  }

  async function handleSignatureVisual() {
    if (!selectedSignatureFile) { showStatus("Please select a PDF to sign.", true); return; }
    const rectArray = parseRect(signatureRectInput);
    if (!rectArray || signatureStrokes.length === 0) { showStatus("Please draw your signature first.", true); return; }
    const colorArray = parseColorHex(signatureColor);
    const outputPath = await getSavePath("signed.pdf");
    if (!outputPath) return;
    startLoading("Applying signature...");
    try {
      await invoke("add_signature_visual", { path: selectedSignatureFile, page: viewerPageNumber, rect: rectArray, strokes: signatureStrokes, color: colorArray, width: signatureWidth ?? 2, outputPath });
      showStatus(`Signature applied successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error signing: ${err}`, true); }
  }

  async function handleCryptoSign() {
    if (!selectedCryptoFile || !signCertPath || !signCertPassword) { showStatus("Missing certificate or password.", true); return; }
    const rectArray = parseRect(signRectInput);
    if (!rectArray) { showStatus("Invalid signature area.", true); return; }
    const outputPath = await getSavePath("signed-crypto.pdf");
    if (!outputPath) return;
    startLoading("Signing (crypto)...");
    try {
      await invoke("sign_pdf_pfx", { path: selectedCryptoFile, page: viewerPageNumber, rect: rectArray, pfxPath: signCertPath, pfxPassword: signCertPassword, reason: signReason || null, location: signLocation || null, contact: signContact || null, outputPath });
      showStatus("Crypto signing completed.", false, outputPath);
      if (!rememberPassword) signCertPassword = "";
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Signing failed: ${err}`, true); }
  }

  async function getSavePath(defaultFilename: string): Promise<string | null> {
    return await invoke("save_file_dialog", { defaultPath: defaultFilename });
  }

  async function selectFile(target: SelectionTarget) {
    const result = await openFileDialog(false);
    if (result && typeof result === "string") {
      switch (target) {
        case "split": selectedSplitFile = result; break;
        case "rotate": selectedRotateFile = result; break;
        case "delete": selectedDeleteFile = result; break;
        case "annotate": selectedAnnotateFile = result; break;
        case "signature": selectedSignatureFile = result; break;
        case "crypto": selectedCryptoFile = result; break;
      }
      viewerFilePath = result;
    }
  }

  async function selectMergeFiles() {
    const result = await openFileDialog(true);
    if (result) selectedMergeFiles = Array.isArray(result) ? result : [result];
  }

  async function selectCertFile() {
    const result = await openFileDialog(false);
    if (result && typeof result === "string") signCertPath = result;
  }

  function handleDroppedFiles(paths: string[]) {
    const pdfs = paths.filter((p) => p.toLowerCase().endsWith(".pdf"));
    if (pdfs.length === 0) { showStatus("Only PDF files are supported.", true); return; }
    if (pdfs.length > 1) {
      selectedMergeFiles = [...new Set([...selectedMergeFiles, ...pdfs])];
      showStatus(`Added ${pdfs.length} PDFs to Merge.`, false);
    } else {
      const path = pdfs[0];
      selectedSplitFile = selectedRotateFile = selectedDeleteFile = selectedAnnotateFile = selectedSignatureFile = selectedCryptoFile = path;
      viewerFilePath = path;
      showStatus(`Selected: ${path.split(/[/\\]/).pop()}`, false);
    }
  }

  function handleViewerClear() {
    switch (viewerTarget) {
      case "annotate": annotationRectInput = ""; break;
      case "signature": signatureStrokes = []; signatureRectInput = ""; break;
      case "crypto": signRectInput = ""; break;
    }
  }

  onMount(async () => {
    const handleGlobalKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && viewerFilePath) viewerFilePath = "";
    };
    window.addEventListener("keydown", handleGlobalKey);
    const unlisten = await getCurrentWebviewWindow().onFileDropEvent((event) => {
      if (event.payload.type === "drop") handleDroppedFiles(event.payload.paths);
    });
    return () => {
      window.removeEventListener("keydown", handleGlobalKey);
      unlisten.then((fn) => fn());
    };
  });

  const tools: { id: ToolId; label: string; icon: string }[] = [
    { id: "merge", label: "Merge", icon: "M" },
    { id: "split", label: "Split", icon: "S" },
    { id: "annotate", label: "Annotate", icon: "A" },
    { id: "signature", label: "Sign", icon: "I" },
    { id: "crypto", label: "Crypto", icon: "C" },
    { id: "organize", label: "Organize", icon: "O" },
  ];
</script>

<div class="flex h-screen overflow-hidden font-sans transition-colors duration-300 bg-white dark:bg-slate-950 text-slate-900 dark:text-slate-100">
  <!-- Sidebar Navigation -->
  <nav class="w-16 flex flex-col items-center py-4 border-r border-slate-200 dark:border-slate-800 transition-colors duration-300 bg-white dark:bg-slate-950">
    <div class="mb-8 font-bold text-blue-600 dark:text-blue-400 text-xl tracking-tighter">PDF</div>
    <div class="flex flex-col gap-2 w-full px-2">
      {#each tools as tool}
        <button
          onclick={() => switchTool(tool.id)}
          class="w-full aspect-square flex flex-col items-center justify-center rounded-lg transition-all {activeTool === tool.id ? 'bg-blue-600 text-white shadow-md' : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100'}"
          title={tool.label}
        >
          <span class="text-lg font-bold">{tool.icon}</span>
          <span class="text-[10px] font-medium mt-0.5 tracking-tight">{tool.label}</span>
        </button>
      {/each}
    </div>
    <div class="mt-auto flex flex-col gap-4">
      <button onclick={() => invoke('shell_open', { filePath: 'https://github.com/sponsors/pbse' })} class="p-2 text-slate-400 hover:text-pink-500 transition-colors" title="Sponsor">
        ❤️
      </button>
    </div>
  </nav>

  <!-- Tool Settings Pane -->
  <aside class="w-80 flex flex-col border-r border-slate-200 dark:border-slate-800 overflow-y-auto transition-colors duration-300 bg-white dark:bg-slate-950">
    <div class="p-6 border-b border-slate-100 dark:border-slate-900 transition-colors duration-300">
      <h2 class="text-xl font-semibold capitalize tracking-tight text-slate-900 dark:text-white transition-colors duration-300">{activeTool}</h2>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 font-medium tracking-tight uppercase tracking-widest opacity-60 transition-colors duration-300">Operations Center</p>
    </div>

    <div class="flex-1 p-6 space-y-8 overflow-y-auto">
      {#if activeTool === 'merge'}
        <div class="space-y-4">
          <button onclick={selectMergeFiles} class="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium transition-colors text-sm shadow-sm">Select PDF Files</button>
          {#if selectedMergeFiles.length > 0}
            <div class="space-y-2 max-h-60 overflow-y-auto rounded-lg border border-slate-200 dark:border-slate-800 p-2 bg-slate-50 dark:bg-slate-900/30 transition-colors shadow-inner">
              {#each selectedMergeFiles as file, i}
                <div class="flex items-center gap-3 p-2 bg-white dark:bg-slate-800 rounded border border-slate-100 dark:border-slate-700 text-[11px] transition-colors shadow-sm">
                  <span class="text-slate-400 w-4 font-mono">{i + 1}</span>
                  <span class="truncate flex-1 font-medium text-slate-700 dark:text-slate-200 transition-colors">{file.split(/[/\\]/).pop()}</span>
                  <button onclick={() => selectedMergeFiles = selectedMergeFiles.filter((_, idx) => idx !== i)} class="text-slate-400 hover:text-red-500 transition-colors">✕</button>
                </div>
              {/each}
            </div>
            <button onclick={() => selectedMergeFiles = []} class="text-[10px] text-slate-400 hover:text-red-500 font-bold uppercase tracking-wider">Clear All</button>
          {/if}
          <button onclick={handleMerge} disabled={selectedMergeFiles.length < 2} class="w-full mt-4 py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold disabled:opacity-30 disabled:cursor-not-allowed text-xs uppercase tracking-widest transition-all shadow-md">Generate Merged PDF</button>
        </div>

      {:else if activeTool === 'split'}
        <div class="space-y-6">
          <button onclick={() => selectFile('split')} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:border-blue-500 dark:hover:border-blue-400 rounded-md transition-all text-sm font-medium truncate shadow-sm">
            {selectedSplitFile ? selectedSplitFile.split(/[/\\]/).pop() : 'Select PDF to Split'}
          </button>
          
          <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
            <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Extract Range</h3>
            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 uppercase tracking-tighter transition-colors">Page Numbers (e.g. 1, 3-5)</label>
              <input type="text" bind:value={splitPagesInput} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none transition-all shadow-sm" />
            </div>
            <button onclick={handleSplit} disabled={!selectedSplitFile || !splitPagesInput} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-xs uppercase tracking-widest transition-colors shadow-md shadow-blue-500/10">Save Range</button>
          </div>

          <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
            <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Extract Current</h3>
            <button onclick={handleExtract} disabled={!selectedSplitFile} class="w-full py-2 border border-blue-600 text-blue-600 dark:text-blue-400 rounded font-bold text-xs uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all">Save Page {viewerPageNumber}</button>
          </div>
        </div>

      {:else if activeTool === 'annotate'}
        <div class="space-y-6">
          <button onclick={() => selectFile('annotate')} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
            {selectedAnnotateFile ? selectedAnnotateFile.split(/[/\\]/).pop() : 'Select PDF'}
          </button>
          
          <div class="space-y-4">
            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Selection Area</label>
              <div class="flex gap-2">
                <input type="text" bind:value={annotationRectInput} class="flex-1 p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none font-mono transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" placeholder="x1, y1, x2, y2" />
                <button onclick={() => openViewer('annotate', 'rect')} class="p-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded border border-blue-100 dark:border-blue-900 hover:bg-blue-100 transition-colors">🎯</button>
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Type</label>
              <select bind:value={annotationType} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm">
                <option value="highlight">Highlight</option>
                <option value="underline">Underline</option>
                <option value="strikeout">Strikeout</option>
                <option value="note">Note</option>
              </select>
            </div>

            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest transition-colors">Content</label>
              <input type="text" bind:value={annotationText} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" />
            </div>

            <button onclick={handleAnnotate} disabled={!selectedAnnotateFile || !annotationRectInput} class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest shadow-lg transition-all hover:scale-[1.02]">Apply Annotation</button>
          </div>
        </div>

      {:else if activeTool === 'signature'}
        <div class="space-y-6">
          <button onclick={() => selectFile('signature')} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
            {selectedSignatureFile ? selectedSignatureFile.split(/[/\\]/).pop() : 'Select PDF'}
          </button>
          
          <div class="space-y-4">
            <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Drawing</h3>
            <div class="flex gap-2">
              <button onclick={() => openViewer('signature', 'points')} class="flex-1 py-2 bg-blue-600 text-white rounded font-bold text-xs uppercase tracking-widest transition-opacity hover:opacity-90 shadow-md shadow-blue-500/20">Open Pad</button>
              <button onclick={() => signatureStrokes = signatureStrokes.slice(0, -1)} disabled={signatureStrokes.length === 0} class="p-2 border border-slate-200 dark:border-slate-800 rounded hover:bg-slate-100 dark:hover:bg-slate-800 disabled:opacity-20 text-[10px] font-bold uppercase transition-colors text-slate-600 dark:text-slate-300">Undo</button>
              <button onclick={() => { signatureStrokes = []; signatureRectInput = ""; }} disabled={signatureStrokes.length === 0} class="p-2 border border-red-200 dark:border-red-900 text-red-500 rounded text-[10px] font-bold uppercase transition-colors hover:bg-red-50 dark:hover:bg-red-950/20">Clear</button>
            </div>
            {#if signatureStrokes.length > 0}
              <div class="flex items-center gap-2 text-green-600 dark:text-green-500 text-[10px] font-bold uppercase tracking-wider transition-colors">
                <span class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
                Signature Ready
              </div>
            {/if}
          </div>

          <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900 transition-colors">
            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Ink Color</label>
              <div class="flex items-center gap-3">
                <input type="color" bind:value={signatureColor} class="w-10 h-10 p-0 border-0 bg-transparent cursor-pointer rounded-full overflow-hidden transition-colors shadow-sm" />
                <span class="text-[10px] font-mono uppercase text-slate-400 dark:text-slate-500 font-bold transition-colors tracking-widest">{signatureColor}</span>
              </div>
            </div>
            <button onclick={handleSignatureVisual} disabled={!selectedSignatureFile || signatureStrokes.length === 0} class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest transition-all shadow-xl hover:scale-[1.02]">Apply Signature</button>
          </div>
        </div>

      {:else if activeTool === 'crypto'}
        <div class="space-y-6">
          <button onclick={() => selectFile('crypto')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
            {selectedCryptoFile ? selectedCryptoFile.split(/[/\\]/).pop() : 'Select PDF'}
          </button>
          <button onclick={selectCertFile} class="w-full py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded text-[10px] font-bold uppercase tracking-wider truncate px-3 transition-colors shadow-sm">
            {signCertPath ? signCertPath.split(/[/\\]/).pop() : 'Select PFX Certificate'}
          </button>

          <div class="space-y-4">
            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Cert Password</label>
              <input type="password" bind:value={signCertPassword} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" />
              <label class="flex items-center gap-2 cursor-pointer mt-1">
                <input type="checkbox" bind:checked={rememberPassword} class="rounded border-slate-300 text-blue-600 focus:ring-blue-500 w-3 h-3 transition-colors" />
                <span class="text-[9px] text-slate-500 font-bold uppercase tracking-tighter transition-colors">Remember for this session</span>
              </label>
            </div>

            <div class="space-y-1.5">
              <label class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Signature Area</label>
              <div class="flex gap-2">
                <input type="text" bind:value={signRectInput} class="flex-1 p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors font-mono shadow-sm" placeholder="x1, y1, x2, y2" />
                <button onclick={() => openViewer('crypto', 'rect')} class="p-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded border border-blue-100 transition-colors hover:bg-blue-100">🎯</button>
              </div>
            </div>

            <button onclick={handleCryptoSign} disabled={!selectedCryptoFile || !signCertPath || !signCertPassword} class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest shadow-xl transition-all hover:scale-[1.02]">Sign Permanently</button>
          </div>
        </div>

      {:else if activeTool === 'organize'}
        <div class="space-y-10">
          <button onclick={() => selectFile('rotate')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
            {selectedRotateFile ? selectedRotateFile.split(/[/\\]/).pop() : 'Select PDF'}
          </button>
          
          <div class="space-y-4">
            <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Rotate Pages</h3>
            <input type="text" bind:value={rotatePagesInput} placeholder="Pages (e.g. 1, 3-5)" class="w-full p-2 border border-slate-200 dark:border-slate-800 rounded text-xs bg-white dark:bg-slate-900 text-slate-900 dark:text-white transition-colors outline-none focus:ring-2 focus:ring-blue-500 shadow-sm" />
            <div class="flex gap-1.5">
              <button onclick={() => handleRotate(90)} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">90°</button>
              <button onclick={() => handleRotate(180)} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">180°</button>
              <button onclick={() => handleRotate(270)} class="flex-1 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all uppercase shadow-sm">270°</button>
            </div>
          </div>

          <div class="space-y-4 pt-6 border-t border-slate-100 dark:border-slate-900 transition-colors">
            <h3 class="text-[10px] font-bold text-red-500 dark:text-red-400 uppercase tracking-widest transition-colors">Delete Pages</h3>
            <input type="text" bind:value={deletePagesInput} placeholder="Pages to remove" class="w-full p-2 border border-slate-200 dark:border-slate-800 rounded text-xs bg-white dark:bg-slate-900 text-slate-900 dark:text-white transition-colors outline-none focus:ring-2 focus:ring-red-500 shadow-sm" />
            <button onclick={handleDelete} class="w-full py-2 border border-red-200 dark:border-red-900 text-red-500 hover:bg-red-50 dark:hover:bg-red-950/20 rounded font-bold text-[10px] uppercase tracking-widest transition-all">Remove Pages</button>
          </div>
        </div>
      {/if}
    </div>
  </aside>

  <!-- Main Preview Area -->
  <main class="flex-1 flex flex-col min-w-0 transition-colors duration-300 bg-slate-50 dark:bg-slate-950">
    {#if viewerFilePath}
      <div class="flex items-center justify-between px-8 py-3 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 shadow-sm z-10 transition-colors duration-300">
        <div class="flex items-center gap-3 overflow-hidden">
          <span class="p-1 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded-md text-[9px] font-black uppercase tracking-tighter transition-colors">PDF</span>
          <h1 class="text-xs font-bold truncate text-slate-700 dark:text-slate-300 max-w-md tracking-tight transition-colors">{viewerFilePath.split(/[/\\]/).pop()}</h1>
        </div>
        <div class="flex items-center gap-4 text-xs font-medium text-slate-500 transition-colors">
           <span class="bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 px-2 py-0.5 rounded-full uppercase tracking-widest text-[9px] font-black transition-colors">Live Preview</span>
        </div>
      </div>
      
      <div class="flex-1 overflow-auto p-8 flex justify-center items-start transition-colors duration-300 bg-slate-100 dark:bg-slate-900/40">
        <PdfViewer
          filePath={viewerFilePath}
          pageNumber={viewerPageNumber}
          mode={viewerMode}
          previewRect={currentPreviewRect}
          previewStrokes={currentPreviewStrokes}
          previewColor={currentPreviewColor}
          onselect={handleViewerSelect}
          onclear={handleViewerClear}
          ondone={() => (viewerMode = "view")}
          onprev={() => (viewerPageNumber = Math.max(1, viewerPageNumber - 1))}
          onnext={() => viewerPageNumber++}
          onclose={() => (viewerFilePath = "")}
        />
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center p-12 transition-colors duration-300 bg-slate-50 dark:bg-slate-950">
        <div class="max-w-sm text-center">
          <div class="w-16 h-16 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl flex items-center justify-center mx-auto mb-6 text-2xl shadow-sm text-slate-400 transition-colors tracking-tighter">📄</div>
          <h2 class="text-xl font-bold text-slate-900 dark:text-white mb-2 tracking-tight transition-colors">Select a tool to begin</h2>
          <p class="text-xs text-slate-500 dark:text-slate-400 mb-8 leading-relaxed font-medium transition-colors tracking-tight">Your files never leave your device. All processing is done locally and securely.</p>
          <div class="inline-flex gap-2 p-1 bg-white dark:bg-slate-900 rounded-full border border-slate-200 dark:border-slate-800 transition-colors shadow-sm">
            <span class="px-3 py-0.5 bg-slate-50 dark:bg-slate-800 text-slate-900 dark:text-white text-[9px] font-black rounded-full shadow-inner uppercase tracking-widest transition-colors">Local</span>
            <span class="px-3 py-0.5 text-slate-500 dark:text-slate-400 text-[9px] font-black uppercase tracking-widest transition-colors tracking-tighter">Private</span>
          </div>
        </div>
      </div>
    {/if}
  </main>
</div>

<StatusDisplay />

<style>
  @reference "tailwindcss";

  :global(.viewer-container) {
    @apply shadow-2xl rounded-2xl overflow-hidden border border-slate-300 dark:border-slate-800 bg-white dark:bg-slate-900 transition-colors duration-300;
  }
  
  :global(.canvas-wrapper) {
    @apply rounded-xl;
  }

  :global(.viewer-controls) {
    @apply bg-slate-100 dark:bg-slate-900 border-t border-slate-300 dark:border-slate-800 p-6 transition-colors duration-300;
  }
</style>
