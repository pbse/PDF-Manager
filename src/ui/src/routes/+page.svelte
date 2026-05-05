<script lang="ts">
  // Import invoke from the core namespace (which remains available)
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";

  // Local imports
  import StatusDisplay from "$lib/components/StatusDisplay.svelte";
  import PdfViewer from "$lib/components/PdfViewer.svelte";
  import { parsePageString } from "$lib/utils";
  import { isLoading, showStatus, startLoading } from "$lib/stores";

  // --- Reactive State ---
  type SelectionTarget =
    | "parse"
    | "split"
    | "extract"
    | "rotate"
    | "delete"
    | "annotate"
    | "signature"
    | "crypto";

  let selectedParseFile: string | null = null;
  let selectedSplitFile: string | null = null;
  let selectedExtractFile: string | null = null;
  let selectedRotateFile: string | null = null;
  let selectedDeleteFile: string | null = null;
  let selectedAnnotateFile: string | null = null;
  let selectedSignatureFile: string | null = null;
  let selectedCryptoFile: string | null = null;
  let parseResult: Record<string, string> | null = null;
  let selectedMergeFiles: string[] = [];
  let splitPagesInput: string = ""; // e.g., "1, 3-5, 8"
  let extractPageInput: number | null = null;
  let rotatePagesInput: string = "";
  let deletePagesInput: string = "";
  let annotationPageInput: number | null = null;
  let annotationRectInput: string = "";
  let annotationType: "highlight" | "underline" | "strikeout" | "note" = "highlight";
  let annotationText: string = "";
  let annotationColor: string = "#facc15";
  let signaturePageInput: number | null = null;
  let signatureRectInput: string = "";
  let signatureColor: string = "#1ebc85";
  let signatureWidth: number | null = 2;
  let signaturePointsInput: string = "";
  let signCertPath: string = "";
  let signCertPassword: string = "";
  let signPageInput: number | null = null;
  let signRectInput: string = "";
  let signReason: string = "";
  let signLocation: string = "";
  let signContact: string = "";

  // --- Active Tool State ---
  type ToolId = "merge" | "annotate" | "signature" | "crypto" | "split" | "parse" | "organize";
  let activeTool: ToolId = "annotate";

  // --- Viewer State ---
  let viewerFilePath = "";
  let viewerPageNumber = 1;
  let viewerMode: "rect" | "points" = "rect";
  let viewerTarget: SelectionTarget | null = null;

  function openViewer(target: SelectionTarget, mode: "rect" | "points" = "rect") {
    let path = "";
    let page = 1;
    switch (target) {
      case "annotate":
        path = selectedAnnotateFile || "";
        page = annotationPageInput || 1;
        break;
      case "signature":
        path = selectedSignatureFile || "";
        page = signaturePageInput || 1;
        break;
      case "crypto":
        path = selectedCryptoFile || "";
        page = signPageInput || 1;
        break;
    }

    if (!path) {
      showStatus("Please select a PDF file first.", true);
      return;
    }

    viewerFilePath = path;
    viewerPageNumber = page;
    viewerMode = mode;
    viewerTarget = target;
  }

  function handleViewerSelect(event: CustomEvent) {
    const { rect, points } = event.detail;

    if (rect) {
      const rectStr = rect.map((n: number) => Math.round(n)).join(", ");
      switch (viewerTarget) {
        case "annotate":
          annotationRectInput = rectStr;
          break;
        case "signature":
          signatureRectInput = rectStr;
          break;
        case "crypto":
          signRectInput = rectStr;
          break;
      }
    }

    if (points) {
      const pointsStr = points.map((p: [number, number]) => `${Math.round(p[0])},${Math.round(p[1])}`).join("; ");
      if (viewerTarget === "signature") {
        signaturePointsInput = pointsStr;
      }
    }
  }

  // --- Helper functions using invoke (wrap native functionality) ---

  async function openFileDialog(
    multiple: boolean = false,
  ): Promise<string | string[] | null> {
    const result = await invoke("open_file_dialog", { multiple });
    if (Array.isArray(result)) {
      return multiple ? result : result[0] || null;
    }
    return null;
  }

  async function saveFileDialog(defaultPath: string): Promise<string | null> {
    // Similarly, implement a `save_file_dialog` command in Rust that returns the chosen save path
    return await invoke("save_file_dialog", { defaultPath });
  }

  async function shellOpen(filePath: string): Promise<void> {
    // Implement a Rust command (e.g. `shell_open`) that uses Tauri’s shell open functionality
    await invoke("shell_open", { filePath });
  }

  async function openPathInExplorer(filePath: string): Promise<void> {
    try {
      await shellOpen(filePath);
    } catch (err) {
      showStatus(`Error opening file: ${err}`, true);
    }
  }

  const sponsorUrl = "https://github.com/sponsors/pbse";
  async function openSponsor() {
    try {
      await shellOpen(sponsorUrl);
    } catch (err) {
      showStatus(`Unable to open sponsor link: ${err}`, true);
    }
  }

  function parseRect(rectStr: string): number[] | null {
    const parts = rectStr
      .split(",")
      .map((p) => p.trim())
      .filter((p) => p.length > 0);
    if (parts.length !== 4) return null;
    const nums = parts.map((p) => Number(p));
    if (nums.some((n) => Number.isNaN(n))) return null;
    return nums;
  }

  function parseColorHex(hex: string): [number, number, number] | null {
    const match = /^#?([a-fA-F0-9]{6})$/.exec(hex.trim());
    if (!match) return null;
    const intVal = parseInt(match[1], 16);
    const r = ((intVal >> 16) & 255) / 255;
    const g = ((intVal >> 8) & 255) / 255;
    const b = (intVal & 255) / 255;
    return [r, g, b];
  }

  async function handleCryptoSign() {
    if (!selectedCryptoFile) {
      showStatus("Select a PDF to sign.", true);
      return;
    }
    if (!signCertPath) {
      showStatus("Select a PFX/P12 certificate.", true);
      return;
    }
    if (!signCertPassword) {
      showStatus("Enter the certificate password.", true);
      return;
    }
    if (!signPageInput || signPageInput <= 0) {
      showStatus("Enter a valid page number.", true);
      return;
    }
    const rectArray = parseRect(signRectInput);
    if (!rectArray) {
      showStatus("Invalid rect. Use format: x1, y1, x2, y2", true);
      return;
    }
    const outputPath = await getSavePath("signed-crypto.pdf");
    if (!outputPath) return;

    startLoading("Signing (crypto)...");
    try {
      await invoke("sign_pdf_pfx", {
        path: selectedCryptoFile,
        page: signPageInput,
        rect: rectArray,
        pfxPath: signCertPath,
        pfxPassword: signCertPassword,
        reason: signReason || null,
        location: signLocation || null,
        contact: signContact || null,
        outputPath,
      });
      showStatus("Signing completed.", false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Signing failed: ${err}`, true);
    }
  }

  async function handleVerifySignatures() {
    if (!selectedCryptoFile) {
      showStatus("Select a PDF to verify.", true);
      return;
    }
    startLoading("Verifying signatures...");
    try {
      const result = await invoke("verify_signatures", { path: selectedCryptoFile });
      showStatus(`Verification result: ${JSON.stringify(result)}`, false);
    } catch (err) {
      showStatus(`Verification failed: ${err}`, true);
    }
  }

  // --- Event Handlers ---

  async function selectFile(target: SelectionTarget) {
    try {
      const result = await openFileDialog(false);
      if (result && typeof result === "string") {
        switch (target) {
          case "parse":
            selectedParseFile = result;
            break;
          case "split":
            selectedSplitFile = result;
            break;
          case "extract":
            selectedExtractFile = result;
            break;
          case "rotate":
            selectedRotateFile = result;
            break;
          case "delete":
            selectedDeleteFile = result;
            break;
          case "annotate":
            selectedAnnotateFile = result;
            break;
          case "signature":
            selectedSignatureFile = result;
            break;
          case "crypto":
            selectedCryptoFile = result;
            break;
        }
        viewerFilePath = result; // Auto-show in preview
      }
    } catch (err) {
      showStatus(`Error selecting file: ${err}`, true);
    }
  }

  async function selectMergeFiles() {
    try {
      const result = await openFileDialog(true);
      if (result) {
        if (Array.isArray(result)) {
          selectedMergeFiles = result;
        } else {
          selectedMergeFiles = [result];
        }
      }
    } catch (err) {
      showStatus(`Error selecting files: ${err}`, true);
    }
  }

  async function getSavePath(defaultFilename: string): Promise<string | null> {
    try {
      const result = await saveFileDialog(defaultFilename);
      return result;
    } catch (err) {
      showStatus(`Error getting save path: ${err}`, true);
      return null;
    }
  }

  async function selectCertFile() {
    try {
      const result = await openFileDialog(false);
      if (result && typeof result === "string") {
        signCertPath = result;
      }
    } catch (err) {
      showStatus(`Error selecting certificate: ${err}`, true);
    }
  }

  // --- Backend Command Invocations ---

  async function handleParse() {
    if (!selectedParseFile) {
      showStatus("Please select a PDF file to parse.", true);
      return;
    }
    startLoading("Parsing PDF...");
    parseResult = null;
    try {
      const result: Record<string, string> = await invoke("parse_pdf", {
        path: selectedParseFile,
      });
      parseResult = result;
      showStatus("PDF parsed successfully.", false);
    } catch (err) {
      showStatus(`Error parsing PDF: ${err}`, true);
    }
  }

  async function handleMerge() {
    if (selectedMergeFiles.length < 2) {
      showStatus("Please select at least two PDF files to merge.", true);
      return;
    }
    const outputPath = await getSavePath("merged.pdf");
    if (!outputPath) return;
    startLoading("Merging PDFs...");
    try {
      await invoke("merge_pdfs", { paths: selectedMergeFiles, outputPath });
      showStatus(`PDFs merged successfully to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error merging PDFs: ${err}`, true);
    }
  }

  async function handleSplit() {
    if (!selectedSplitFile) {
      showStatus("Please select a PDF file to split.", true);
      return;
    }
    const pagesArray = parsePageString(splitPagesInput);
    if (pagesArray === null) {
      showStatus(
        "Invalid page format. Use numbers, commas, and hyphens (e.g., 1, 3-5, 8).",
        true,
      );
      return;
    }
    if (pagesArray.length === 0) {
      showStatus("Please enter page numbers to extract.", true);
      return;
    }
    const outputPath = await getSavePath("split.pdf");
    if (!outputPath) return;
    startLoading("Splitting PDF...");
    try {
      await invoke("split_pdf", {
        path: selectedSplitFile,
        pages: pagesArray,
        outputPath,
      });
      showStatus(`PDF split successfully to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error splitting PDF: ${err}`, true);
    }
  }

  async function handleExtract() {
    if (!selectedExtractFile) {
      showStatus("Please select a PDF file to extract from.", true);
      return;
    }
    if (extractPageInput === null || extractPageInput <= 0) {
      showStatus("Please enter a valid positive page number to extract.", true);
      return;
    }
    const outputPath = await getSavePath(`page_${extractPageInput}.pdf`);
    if (!outputPath) return;
    startLoading("Extracting page...");
    try {
      await invoke("extract_pdf_page", {
        path: selectedExtractFile,
        pageNumber: extractPageInput,
        outputPath,
      });
      showStatus(
        `Page ${extractPageInput} extracted successfully to ${outputPath}.`,
        false,
      );
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error extracting page: ${err}`, true);
    }
  }

  async function handleRotate(rotation: number) {
    if (!selectedRotateFile) {
      showStatus("Please select a PDF file to rotate.", true);
      return;
    }

    const pagesArray = parsePageString(rotatePagesInput);
    if (pagesArray === null) {
      showStatus(
        "Invalid page format. Use numbers, commas, and hyphens (e.g., 1, 3-5, 8).",
        true,
      );
      return;
    }

    const outputPath = await getSavePath("rotated.pdf");
    if (!outputPath) return;

    startLoading("Rotating PDF...");
    try {
      await invoke("rotate_pdf", {
        path: selectedRotateFile,
        pages: pagesArray,
        rotation: rotation,
        outputPath,
      });
      showStatus(`PDF rotated successfully to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error rotating PDF: ${err}`, true);
    }
  }

  async function handleDelete() {
    if (!selectedDeleteFile) {
      showStatus("Please select a PDF file to delete pages from.", true);
      return;
    }

    const pagesArray = parsePageString(deletePagesInput);
    if (pagesArray === null) {
      showStatus(
        "Invalid page format. Use numbers, commas, and hyphens (e.g., 1, 3-5, 8).",
        true,
      );
      return;
    }

    if (pagesArray.length === 0) {
      showStatus("Please enter page numbers to delete.", true);
      return;
    }

    const outputPath = await getSavePath("deleted.pdf");
    if (!outputPath) return;

    startLoading("Deleting pages...");
    try {
      await invoke("delete_pages", {
        path: selectedDeleteFile,
        pagesToDelete: pagesArray,
        outputPath,
      });
      showStatus(`Pages deleted successfully to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error deleting pages: ${err}`, true);
    }
  }

  function clearParseSelection() {
    selectedParseFile = null;
    parseResult = null;
    if (viewerFilePath === selectedParseFile) viewerFilePath = "";
  }

  function clearMergeSelection() {
    selectedMergeFiles = [];
  }

  function clearSplitSelection() {
    selectedSplitFile = null;
    splitPagesInput = "";
    if (viewerFilePath === selectedSplitFile) viewerFilePath = "";
  }

  function clearExtractSelection() {
    selectedExtractFile = null;
    extractPageInput = null;
    if (viewerFilePath === selectedExtractFile) viewerFilePath = "";
  }

  function clearRotateSelection() {
    selectedRotateFile = null;
    rotatePagesInput = "";
    if (viewerFilePath === selectedRotateFile) viewerFilePath = "";
  }

  function clearDeleteSelection() {
    selectedDeleteFile = null;
    deletePagesInput = "";
    if (viewerFilePath === selectedDeleteFile) viewerFilePath = "";
  }

  function clearAnnotateSelection() {
    selectedAnnotateFile = null;
    annotationPageInput = null;
    annotationRectInput = "";
    annotationText = "";
    annotationType = "highlight";
    annotationColor = "#facc15";
    if (viewerFilePath === selectedAnnotateFile) viewerFilePath = "";
  }

  function clearSignatureSelection() {
    selectedSignatureFile = null;
    signaturePageInput = null;
    signatureRectInput = "";
    signatureColor = "#1ebc85";
    signatureWidth = 2;
    signaturePointsInput = "";
    if (viewerFilePath === selectedSignatureFile) viewerFilePath = "";
  }

  function clearCryptoSelection() {
    selectedCryptoFile = null;
    signCertPath = "";
    signCertPassword = "";
    signPageInput = null;
    signRectInput = "";
    signReason = "";
    signLocation = "";
    signContact = "";
    if (viewerFilePath === selectedCryptoFile) viewerFilePath = "";
  }

  $: {
    if (viewerTarget === "annotate" && annotationPageInput) {
      viewerPageNumber = annotationPageInput;
    } else if (viewerTarget === "signature" && signaturePageInput) {
      viewerPageNumber = signaturePageInput;
    } else if (viewerTarget === "crypto" && signPageInput) {
      viewerPageNumber = signPageInput;
    }
  }

  async function handleAnnotate() {
    if (!selectedAnnotateFile) {
      showStatus("Please select a PDF to annotate.", true);
      return;
    }
    if (!annotationPageInput || annotationPageInput <= 0) {
      showStatus("Enter a valid page number.", true);
      return;
    }
    const rectArray = parseRect(annotationRectInput);
    if (!rectArray) {
      showStatus("Invalid rect. Use format: x1, y1, x2, y2", true);
      return;
    }
    const colorArray = parseColorHex(annotationColor);
    if (!colorArray) {
      showStatus("Invalid color. Use 6-digit hex.", true);
      return;
    }

    const outputPath = await getSavePath("annotated.pdf");
    if (!outputPath) return;

    startLoading("Adding annotation...");
    try {
      await invoke("add_annotation", {
        path: selectedAnnotateFile,
        page: annotationPageInput,
        rect: rectArray,
        kind: annotationType,
        contents: annotationText || null,
        color: colorArray,
        outputPath,
      });
      showStatus(`Annotation added and saved to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error adding annotation: ${err}`, true);
    }
  }

  function parsePoints(pointsStr: string): [number, number][] | null {
    const cleaned = pointsStr.trim();
    if (!cleaned) return null;
    const pairs = cleaned
      .split(/[\n;]+/) // allow semicolons or newlines between pairs
      .map((p) => p.trim())
      .filter((p) => p.length > 0);
    if (pairs.length < 2) return null;
    const result: [number, number][] = [];
    for (const pair of pairs) {
      const coords = pair.split(/[, ]+/).map((c) => c.trim()).filter((c) => c.length > 0);
      if (coords.length !== 2) return null;
      const x = Number(coords[0]);
      const y = Number(coords[1]);
      if (Number.isNaN(x) || Number.isNaN(y)) return null;
      result.push([x, y]);
    }
    return result;
  }

  async function handleSignatureVisual() {
    if (!selectedSignatureFile) {
      showStatus("Please select a PDF to sign visually.", true);
      return;
    }
    if (!signaturePageInput || signaturePageInput <= 0) {
      showStatus("Enter a valid page number.", true);
      return;
    }
    const rectArray = parseRect(signatureRectInput);
    if (!rectArray) {
      showStatus("Invalid rect. Use format: x1, y1, x2, y2", true);
      return;
    }
    const colorArray = parseColorHex(signatureColor);
    if (!colorArray) {
      showStatus("Invalid color. Use 6-digit hex.", true);
      return;
    }
    const points = parsePoints(signaturePointsInput);
    if (!points) {
      showStatus("Provide at least two points as x,y; x,y;", true);
      return;
    }
    const outputPath = await getSavePath("signed-visual.pdf");
    if (!outputPath) return;

    startLoading("Applying visual signature...");
    try {
      await invoke("add_signature_visual", {
        path: selectedSignatureFile,
        page: signaturePageInput,
        rect: rectArray,
        points,
        color: colorArray,
        width: signatureWidth ?? 2,
        outputPath,
      });
      showStatus(`Visual signature added and saved to ${outputPath}.`, false);
      await openPathInExplorer(outputPath);
    } catch (err) {
      showStatus(`Error adding visual signature: ${err}`, true);
    }
  }

  // --- Drag and Drop ---

  function handleDroppedFiles(paths: string[]) {
    if (paths.length === 0) return;
    const pdfs = paths.filter((p) => p.toLowerCase().endsWith(".pdf"));
    if (pdfs.length === 0) {
      showStatus("Only PDF files are supported for drop.", true);
      return;
    }

    if (pdfs.length > 1) {
      selectedMergeFiles = [...new Set([...selectedMergeFiles, ...pdfs])];
      showStatus(`Added ${pdfs.length} PDFs to Merge section.`, false);
    } else {
      const path = pdfs[0];
      // Assign to all single-file tools for convenience
      selectedParseFile = path;
      selectedSplitFile = path;
      selectedExtractFile = path;
      selectedRotateFile = path;
      selectedDeleteFile = path;
      selectedAnnotateFile = path;
      selectedSignatureFile = path;
      selectedCryptoFile = path;
      viewerFilePath = path; // Auto-show in preview
      showStatus(`Selected PDF: ${path.split(/[/\\]/).pop()}`, false);
    }
  }

  onMount(async () => {
    const unlisten = await getCurrentWebviewWindow().onFileDropEvent((event) => {
      if (event.payload.type === "drop") {
        handleDroppedFiles(event.payload.paths);
      }
    });
    return unlisten;
  });
</script>

<div class="app-container">
  <header class="hero">
    <div class="hero-content">
      <p class="eyebrow">Workspace / PDF Ops</p>
      <h1>PDF Manager</h1>
      <p class="subtitle">Reliable desktop-grade PDF tools. Files never leave your device.</p>
    </div>
    <div class="hero-actions">
      <span class="chip success">Ready</span>
      <span class="chip neutral">Local Processing</span>
      <button class="sponsor-button-small" on:click={openSponsor}>Sponsor</button>
    </div>
  </header>

  <main class="workspace">
    <!-- Leftmost: Tool Switcher -->
    <nav class="tool-nav">
      <button class:active={activeTool === 'merge'} on:click={() => activeTool = 'merge'}>
        <span class="nav-icon">🔗</span>
        <span class="nav-label">Merge</span>
      </button>
      <button class:active={activeTool === 'annotate'} on:click={() => activeTool = 'annotate'}>
        <span class="nav-icon">✏️</span>
        <span class="nav-label">Annotate</span>
      </button>
      <button class:active={activeTool === 'signature'} on:click={() => activeTool = 'signature'}>
        <span class="nav-icon">✍️</span>
        <span class="nav-label">Ink</span>
      </button>
      <button class:active={activeTool === 'crypto'} on:click={() => activeTool = 'crypto'}>
        <span class="nav-icon">🔐</span>
        <span class="nav-label">Crypto</span>
      </button>
      <button class:active={activeTool === 'split'} on:click={() => activeTool = 'split'}>
        <span class="nav-icon">✂️</span>
        <span class="nav-label">Split</span>
      </button>
      <button class:active={activeTool === 'organize'} on:click={() => activeTool = 'organize'}>
        <span class="nav-icon">📑</span>
        <span class="nav-label">Pages</span>
      </button>
    </nav>

    <!-- Middle Pane: Active Tool Settings -->
    <aside class="tool-settings">
      {#if activeTool === 'merge'}
        <div class="settings-header">
          <h2>Merge PDFs</h2>
          <button class="clear-button-small" on:click={clearMergeSelection} disabled={selectedMergeFiles.length === 0}>Clear</button>
        </div>
        <div class="settings-body">
          <p class="settings-hint">Combine multiple files into one.</p>
          <button class="primary-button" on:click={selectMergeFiles}>Select Files</button>
          {#if selectedMergeFiles.length > 0}
            <div class="files-list-vertical">
              {#each selectedMergeFiles as file, i}
                <div class="file-item-mini">
                  <span class="num">{i+1}</span>
                  <button class="name" on:click={() => viewerFilePath = file}>{file.split(/[/\\]/).pop()}</button>
                </div>
              {/each}
            </div>
          {/if}
          <button class="action-button-main" on:click={handleMerge} disabled={selectedMergeFiles.length < 2}>
            Generate Merged PDF
          </button>
        </div>

      {:else if activeTool === 'annotate'}
        <div class="settings-header">
          <h2>Annotate</h2>
        </div>
        <div class="settings-body">
          <button class="primary-button" on:click={() => selectFile("annotate")}>Select PDF</button>
          
          <div class="input-group">
            <label for="a-page">Page</label>
            <input id="a-page" type="number" bind:value={annotationPageInput} />
          </div>

          <div class="input-group">
            <label for="a-rect">Selection Area</label>
            <div class="input-with-action">
              <input id="a-rect" type="text" bind:value={annotationRectInput} placeholder="x1, y1, x2, y2" />
              <button class="icon-button" on:click={() => openViewer("annotate", "rect")}>🎯</button>
            </div>
          </div>

          <div class="input-group">
            <label for="a-type">Type</label>
            <select id="a-type" bind:value={annotationType}>
              <option value="highlight">Highlight</option>
              <option value="underline">Underline</option>
              <option value="strikeout">Strikeout</option>
              <option value="note">Note</option>
            </select>
          </div>

          <div class="input-group">
            <label for="a-text">Text Content</label>
            <input id="a-text" type="text" bind:value={annotationText} />
          </div>

          <button class="action-button-main" on:click={handleAnnotate} disabled={!selectedAnnotateFile || !annotationRectInput}>
            Apply Annotation
          </button>
        </div>

      {:else if activeTool === 'signature'}
        <div class="settings-header">
          <h2>Ink Signature</h2>
        </div>
        <div class="settings-body">
          <button class="primary-button" on:click={() => selectFile("signature")}>Select PDF</button>
          
          <div class="input-group">
            <label for="s-page">Page</label>
            <input id="s-page" type="number" bind:value={signaturePageInput} />
          </div>

          <div class="input-group">
            <label for="s-rect">Visual Box</label>
            <div class="input-with-action">
              <input id="s-rect" type="text" bind:value={signatureRectInput} placeholder="Bounding box" />
              <button class="icon-button" on:click={() => openViewer("signature", "rect")}>🎯</button>
            </div>
          </div>

          <div class="input-group">
            <label for="s-points">Signature Path</label>
            <div class="input-with-action">
              <input id="s-points" type="text" bind:value={signaturePointsInput} placeholder="Drawn points" />
              <button class="icon-button" on:click={() => openViewer("signature", "points")}>✍️</button>
            </div>
          </div>

          <button class="action-button-main" on:click={handleSignatureVisual} disabled={!selectedSignatureFile || !signatureRectInput || !signaturePointsInput}>
            Place Visual Signature
          </button>
        </div>

      {:else if activeTool === 'crypto'}
        <div class="settings-header">
          <h2>Crypto Signing</h2>
        </div>
        <div class="settings-body">
          <button class="primary-button" on:click={() => selectFile("crypto")}>Select PDF</button>
          <button class="secondary-button" on:click={selectCertFile}>{signCertPath ? 'Change Cert' : 'Select PFX/P12'}</button>

          {#if signCertPath}
            <p class="cert-status">✅ {signCertPath.split(/[/\\]/).pop()}</p>
          {/if}

          <div class="input-group">
            <label for="c-pass">Cert Password</label>
            <input id="c-pass" type="password" bind:value={signCertPassword} />
          </div>

          <div class="input-group">
            <label for="c-rect">Signature Area</label>
            <div class="input-with-action">
              <input id="c-rect" type="text" bind:value={signRectInput} />
              <button class="icon-button" on:click={() => openViewer("crypto", "rect")}>🎯</button>
            </div>
          </div>

          <button class="action-button-main" on:click={handleCryptoSign} disabled={!selectedCryptoFile || !signCertPath || !signCertPassword}>
            Finalize & Sign
          </button>
        </div>

      {:else if activeTool === 'split'}
        <div class="settings-header">
          <h2>Split & Extract</h2>
        </div>
        <div class="settings-body">
          <button class="primary-button" on:click={() => selectFile("split")}>Select PDF</button>
          <div class="input-group">
            <label for="sp-pages">Pages (e.g. 1, 3-5, 8)</label>
            <input id="sp-pages" type="text" bind:value={splitPagesInput} />
          </div>
          <button class="action-button-main" on:click={handleSplit} disabled={!selectedSplitFile || !splitPagesInput}>
            Extract Specific Pages
          </button>
        </div>

      {:else if activeTool === 'organize'}
        <div class="settings-header">
          <h2>Page Operations</h2>
        </div>
        <div class="settings-body">
          <button class="primary-button" on:click={() => selectFile("rotate")}>Select PDF</button>
          
          <div class="tool-section">
             <h3>Rotate</h3>
             <input type="text" bind:value={rotatePagesInput} placeholder="Pages to rotate" />
             <div class="button-row">
               <button on:click={() => handleRotate(90)}>90°</button>
               <button on:click={() => handleRotate(180)}>180°</button>
               <button on:click={() => handleRotate(270)}>270°</button>
             </div>
          </div>

          <div class="tool-section">
            <h3>Delete</h3>
            <input type="text" bind:value={deletePagesInput} placeholder="Pages to delete" />
            <button class="danger-button" on:click={handleDelete}>Remove Pages</button>
          </div>
        </div>
      {/if}
    </aside>

    <!-- Right Pane: Preview -->
    <section class="preview-pane">
      {#if viewerFilePath}
        <div class="preview-header">
          <span class="preview-filename">{viewerFilePath.split(/[/\\]/).pop()}</span>
          <div class="preview-actions">
             <button class="icon-button-small" on:click={() => viewerPageNumber = Math.max(1, viewerPageNumber - 1)}>◀</button>
             <input type="number" bind:value={viewerPageNumber} min="1" class="page-nav-input" />
             <button class="icon-button-small" on:click={() => viewerPageNumber++}>▶</button>
          </div>
        </div>
        <PdfViewer
          filePath={viewerFilePath}
          pageNumber={viewerPageNumber}
          mode={viewerMode}
          on:select={handleViewerSelect}
          on:close={() => (viewerFilePath = "")}
        />
      {:else}
        <div class="empty-preview">
          <div class="drop-target-visual">
            <span class="big-icon">📄</span>
            <h3>Workspace</h3>
            <p>Select a tool and file to begin.</p>
          </div>
        </div>
      {/if}
    </section>
  </main>

  <footer class="app-footer">
    <div class="footer-left">
      <p class="footer-text">Files never leave your device.</p>
      <p class="footer-text">Need help? Check logs in the status bar below.</p>
    </div>
    <button class="sponsor-button" on:click={openSponsor}>
      Sponsor
    </button>
  </footer>
</div>

<StatusDisplay />

<style>
  :root {
    --primary-color: #1d4ed8;
    --primary-accent: #22d3ee;
    --secondary-color: #0f172a;
    --background-color: #0b1021;
    --surface: rgba(255, 255, 255, 0.06);
    --card-background: rgba(255, 255, 255, 0.08);
    --text-color: #e2e8f0;
    --muted: #94a3b8;
    --border-color: rgba(255, 255, 255, 0.08);
    --hover-color: rgba(255, 255, 255, 0.12);
    --disabled-color: #475569;
    --shadow-strong: 0 20px 60px rgba(0, 0, 0, 0.35);
    --gradient: radial-gradient(circle at 20% 20%, rgba(34, 211, 238, 0.18), transparent 35%),
      radial-gradient(circle at 80% 0%, rgba(99, 102, 241, 0.2), transparent 30%),
      linear-gradient(135deg, rgba(13, 31, 77, 0.9), rgba(10, 15, 35, 0.95));
  }

  @media (max-width: 768px) {
    :root {
      --shadow-strong: 0 12px 30px rgba(0, 0, 0, 0.35);
    }
  }

  * {
    box-sizing: border-box;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .app-container {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background: var(--gradient);
    font-family: "Manrope", "SF Pro Display", "Segoe UI", sans-serif;
    color: var(--text-color);
  }

  .hero {
    height: 64px;
    padding: 0 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    flex-shrink: 0;
  }

  .hero h1 {
    margin: 0;
    font-size: 1.5rem;
    background: linear-gradient(120deg, #22d3ee, #818cf8);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .workspace {
    flex: 1;
    display: grid;
    grid-template-columns: 80px 320px 1fr;
    min-height: 0;
    overflow: hidden;
  }

  /* 1. Tool Nav (Leftmost) */
  .tool-nav {
    background: rgba(0, 0, 0, 0.4);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    padding: 1rem 0.5rem;
    gap: 0.5rem;
  }

  .tool-nav button {
    background: transparent;
    border: none;
    color: var(--muted);
    padding: 0.75rem 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    cursor: pointer;
    border-radius: 0.5rem;
    transition: all 0.2s;
  }

  .tool-nav button:hover {
    background: rgba(255, 255, 255, 0.05);
    color: white;
  }

  .tool-nav button.active {
    background: rgba(34, 211, 238, 0.15);
    color: var(--primary-accent);
  }

  .nav-icon {
    font-size: 1.5rem;
  }

  .nav-label {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* 2. Tool Settings (Middle) */
  .tool-settings {
    background: rgba(0, 0, 0, 0.2);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .settings-header {
    padding: 1.25rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
  }

  .settings-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .settings-hint {
    font-size: 0.85rem;
    color: var(--muted);
    margin: -0.5rem 0 0.5rem;
  }

  .action-button-main {
    margin-top: 1rem;
    padding: 0.85rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(29, 78, 216, 0.3);
  }

  .action-button-main:disabled {
    background: var(--disabled-color);
    box-shadow: none;
    cursor: not-allowed;
  }

  /* 3. Preview Pane (Right) */
  .preview-pane {
    background: rgba(0, 0, 0, 0.3);
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .files-list-vertical {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.2);
    padding: 0.5rem;
    border-radius: 0.5rem;
  }

  .file-item-mini {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .file-item-mini .num {
    font-size: 0.7rem;
    color: var(--muted);
    width: 15px;
  }

  .file-item-mini .name {
    background: transparent;
    border: none;
    color: #e2e8f0;
    font-size: 0.8rem;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .file-item-mini .name:hover {
    color: var(--primary-accent);
  }

  .tool-section {
    border-top: 1px solid var(--border-color);
    padding-top: 1rem;
  }

  .tool-section h3 {
    font-size: 0.9rem;
    margin-bottom: 0.75rem;
    color: var(--muted);
  }

  .button-row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .danger-button {
    background: rgba(239, 68, 68, 0.1);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
    padding: 0.5rem;
    border-radius: 0.4rem;
    cursor: pointer;
    width: 100%;
    margin-top: 0.5rem;
  }

  .cert-status {
    font-size: 0.8rem;
    color: #4ade80;
    margin: -1rem 0 0;
  }

  .preview-header {
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
  }

  .preview-filename {
    font-weight: 600;
    color: var(--primary-accent);
  }

  .preview-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .page-nav-input {
    width: 50px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-color);
    color: white;
    text-align: center;
    border-radius: 4px;
    padding: 0.2rem;
  }

  .empty-preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .drop-target-visual {
    text-align: center;
    padding: 3rem;
    border: 2px dashed var(--border-color);
    border-radius: 2rem;
    background: rgba(255, 255, 255, 0.02);
  }

  .big-icon {
    font-size: 4rem;
    display: block;
    margin-bottom: 1rem;
  }

  .card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .card-header {
    background: rgba(255, 255, 255, 0.05);
    padding: 0.75rem 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
  }

  .card-header h2 {
    font-size: 1rem;
    margin: 0;
  }

  .card-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .file-badge.active {
    border-color: var(--primary-accent);
    background: rgba(34, 211, 238, 0.1);
    cursor: pointer;
  }

  .icon-button-small {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: white;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .sponsor-button-small {
    background: linear-gradient(120deg, #ec4899, #8b5cf6);
    color: white;
    border: none;
    padding: 0.4rem 1rem;
    border-radius: 99px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
  }

  .full-width {
    width: 100%;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .primary-button,
  .secondary-button {
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    border: none;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    letter-spacing: 0.01em;
  }

  .primary-button {
    background: linear-gradient(120deg, #2563eb, #22d3ee);
    color: #f8fafc;
    box-shadow: 0 10px 30px rgba(34, 211, 238, 0.25);
  }

  .secondary-button {
    background: rgba(255, 255, 255, 0.08);
    color: #e2e8f0;
    border: 1px solid var(--border-color);
  }

  .primary-button:hover,
  .secondary-button:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .primary-button:disabled,
  .secondary-button:disabled {
    background: var(--disabled-color);
    cursor: not-allowed;
    transform: none;
  }

  .file-badge {
    background: rgba(255, 255, 255, 0.06);
    padding: 0.6rem 0.9rem;
    border-radius: 0.75rem;
    margin: 0.25rem 0;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    border: 1px solid var(--border-color);
  }

  .input-group {
    margin: 1rem 0;
    width: 100%;
    box-sizing: border-box;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.5rem;
    color: #cbd5e1;
    font-size: 0.9rem;
  }

  .input-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.65rem;
    font-size: 1rem;
    box-sizing: border-box;
    min-width: 0;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-color);
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .input-group select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.65rem;
    font-size: 1rem;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-color);
    box-sizing: border-box;
  }

  .input-group input[type="password"] {
    letter-spacing: 0.08em;
  }

  .input-group input:focus {
    outline: none;
    border-color: rgba(34, 211, 238, 0.7);
    box-shadow: 0 0 0 4px rgba(34, 211, 238, 0.12);
  }

  .app-footer {
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    background: rgba(0, 0, 0, 0.1);
    flex-shrink: 0;
  }

  .sponsor-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(120deg, #ec4899, #8b5cf6);
    color: white;
    padding: 0.65rem 1.6rem;
    border-radius: 9999px;
    text-decoration: none;
    font-weight: 600;
    transition: transform 0.2s, box-shadow 0.2s;
    box-shadow: 0 12px 30px rgba(236, 72, 153, 0.25);
  }

  .sponsor-button:hover {
    transform: translateY(-1px) scale(1.01);
    box-shadow: 0 16px 36px rgba(139, 92, 246, 0.3);
  }

  .footer-text {
    color: var(--muted);
    margin: 0.15rem 0;
  }

  .footer-left {
    display: flex;
    flex-direction: column;
  }

  /* Add new styles */
  .clear-button {
    background: transparent;
    color: white;
    border: 1px solid rgba(255, 255, 255, 0.5);
    padding: 0.25rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .clear-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: white;
  }

  .clear-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .input-with-action {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .icon-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 1.2rem;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .icon-button:hover:not(:disabled) {
    background: var(--hover-color);
    transform: scale(1.05);
  }

  .icon-button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .app-container {
      padding: 0.75rem;
    }

    .hero {
      grid-template-columns: 1fr;
      gap: 1rem;
    }

    .button-group {
      flex-direction: column;
    }

    .input-group input {
      padding: 0.5rem;
      font-size: 0.9rem;
    }
  }
</style>
