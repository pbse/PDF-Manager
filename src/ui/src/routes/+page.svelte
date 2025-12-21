<script lang="ts">
  // Import invoke from the core namespace (which remains available)
  import { invoke } from "@tauri-apps/api/core";

  // Local imports
  import StatusDisplay from "$lib/components/StatusDisplay.svelte";
  import { parsePageString } from "$lib/utils";
  import { isLoading, showStatus, startLoading } from "$lib/stores";

  // --- Reactive State ---
  type SelectionTarget = "parse" | "split" | "extract" | "rotate" | "delete";

  let selectedParseFile: string | null = null;
  let selectedSplitFile: string | null = null;
  let selectedExtractFile: string | null = null;
  let selectedRotateFile: string | null = null;
  let selectedDeleteFile: string | null = null;
  let parseResult: Record<string, string> | null = null;
  let selectedMergeFiles: string[] = [];
  let splitPagesInput: string = ""; // e.g., "1, 3-5, 8"
  let extractPageInput: number | null = null;
  let rotatePagesInput: string = "";
  let deletePagesInput: string = "";

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
        }
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
  }

  function clearMergeSelection() {
    selectedMergeFiles = [];
  }

  function clearSplitSelection() {
    selectedSplitFile = null;
    splitPagesInput = "";
  }

  function clearExtractSelection() {
    selectedExtractFile = null;
    extractPageInput = null;
  }

  function clearRotateSelection() {
    selectedRotateFile = null;
    rotatePagesInput = "";
  }

  function clearDeleteSelection() {
    selectedDeleteFile = null;
    deletePagesInput = "";
  }
</script>

<div class="app-container">
  <header class="hero">
    <div>
      <p class="eyebrow">Workspace / PDF Ops</p>
      <h1>PDF Manager</h1>
      <p class="subtitle">
        Reliable desktop-grade PDF tools for merging, splitting, rotating, and inspecting files. All processing is done locally on your machine.
      </p>
      <div class="hero-actions">
        <span class="chip success">Ready</span>
        <span class="chip neutral">Local Processing</span>
        <span class="chip ghost">No cloud upload</span>
      </div>
    </div>
    <div class="hero-card">
      <p class="hero-label">Quick Start</p>
      <div class="hero-steps">
        <div>
          <span class="step-num">1</span>
          <span>Select PDF(s)</span>
        </div>
        <div>
          <span class="step-num">2</span>
          <span>Pick an action</span>
        </div>
        <div>
          <span class="step-num">3</span>
          <span>Save & preview</span>
        </div>
      </div>
      <p class="muted">Supports macOS, Windows, Linux. Files stay on your machine.</p>
    </div>
  </header>

  <main class="content-grid">
    <!-- Merge Section -->
    <section class="card">
      <div class="card-header">
        <h2>🔗 Merge PDFs</h2>
        <button
          class="clear-button"
          on:click={clearMergeSelection}
          disabled={selectedMergeFiles.length === 0}
        >
          Clear
        </button>
      </div>
      <div class="card-content">
        <div class="button-group">
          <button
            class="primary-button"
            on:click={selectMergeFiles}
            disabled={$isLoading}
          >
            Select Files
          </button>
          <button
            class="secondary-button"
            on:click={handleMerge}
            disabled={selectedMergeFiles.length < 2 || $isLoading}
          >
            Merge Selected
          </button>
        </div>
        {#if selectedMergeFiles.length > 0}
          <div class="files-list">
            {#each selectedMergeFiles as file, i}
              <div class="file-badge">
                <span class="file-number">{i + 1}</span>
                <span class="file-name">{file}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Split and Extract Grid -->
    <div class="tools-grid">
      <section class="card">
        <div class="card-header">
          <h2>✂️ Split PDF</h2>
          <button
            class="clear-button"
            on:click={clearSplitSelection}
            disabled={!selectedSplitFile}
          >
            Clear
          </button>
        </div>
        <div class="card-content">
          <button
            class="primary-button"
            on:click={() => selectFile("split")}
            disabled={$isLoading}
          >
            Select PDF
          </button>
          {#if selectedSplitFile}
            <div class="file-badge">
              <span class="file-icon">📄</span>
              <span class="file-name">{selectedSplitFile}</span>
            </div>
          {/if}
          <div class="input-group">
            <label for="split-pages">Pages to Extract</label>
            <input
              id="split-pages"
              type="text"
              bind:value={splitPagesInput}
              disabled={$isLoading}
              placeholder="e.g., 1, 3-5, 8"
            />
          </div>
          <button
            class="secondary-button full-width"
            on:click={handleSplit}
            disabled={!selectedSplitFile || !splitPagesInput || $isLoading}
          >
            Split PDF
          </button>
        </div>
      </section>

      <section class="card">
        <div class="card-header">
          <h2>DELETE</h2>
          <button
            class="clear-button"
            on:click={clearDeleteSelection}
            disabled={!selectedDeleteFile}
          >
            Clear
          </button>
        </div>
        <div class="card-content">
          <button
            class="primary-button"
            on:click={() => selectFile("delete")}
            disabled={$isLoading}
          >
            Select PDF
          </button>
          {#if selectedDeleteFile}
            <div class="file-badge">
              <span class="file-icon">📄</span>
              <span class="file-name">{selectedDeleteFile}</span>
            </div>
          {/if}
          <div class="input-group">
            <label for="delete-pages">Pages to Delete</label>
            <input
              id="delete-pages"
              type="text"
              bind:value={deletePagesInput}
              disabled={$isLoading}
              placeholder="e.g., 1, 3-5, 8"
            />
          </div>
          <button
            class="secondary-button full-width"
            on:click={handleDelete}
            disabled={!selectedDeleteFile || !deletePagesInput || $isLoading}
          >
            Delete Pages
          </button>
        </div>
      </section>

      <section class="card">
        <div class="card-header">
          <h2>ROTATE</h2>
          <button
            class="clear-button"
            on:click={clearRotateSelection}
            disabled={!selectedRotateFile}
          >
            Clear
          </button>
        </div>
        <div class="card-content">
          <button
            class="primary-button"
            on:click={() => selectFile("rotate")}
            disabled={$isLoading}
          >
            Select PDF
          </button>
          {#if selectedRotateFile}
            <div class="file-badge">
              <span class="file-icon">📄</span>
              <span class="file-name">{selectedRotateFile}</span>
            </div>
          {/if}
          <div class="input-group">
            <label for="rotate-pages">Pages to Rotate</label>
            <input
              id="rotate-pages"
              type="text"
              bind:value={rotatePagesInput}
              disabled={$isLoading}
              placeholder="e.g., 1, 3-5, 8"
            />
          </div>
          <div class="button-group">
            <button
              class="secondary-button"
              on:click={() => handleRotate(90)}
              disabled={!selectedRotateFile || $isLoading}
            >
              90°
            </button>
            <button
              class="secondary-button"
              on:click={() => handleRotate(180)}
              disabled={!selectedRotateFile || $isLoading}
            >
              180°
            </button>
            <button
              class="secondary-button"
              on:click={() => handleRotate(270)}
              disabled={!selectedRotateFile || $isLoading}
            >
              270°
            </button>
          </div>
        </div>
      </section>

      <section class="card">
        <div class="card-header">
          <h2>📑 Extract Page</h2>
          <button
            class="clear-button"
            on:click={clearExtractSelection}
            disabled={!selectedExtractFile}
          >
            Clear
          </button>
        </div>
        <div class="card-content">
          <button
            class="primary-button"
            on:click={() => selectFile("extract")}
            disabled={$isLoading}
          >
            Select PDF
          </button>
          {#if selectedExtractFile}
            <div class="file-badge">
              <span class="file-icon">📄</span>
              <span class="file-name">{selectedExtractFile}</span>
            </div>
          {/if}
          <div class="input-group">
            <label for="extract-page">Page Number</label>
            <input
              id="extract-page"
              type="number"
              bind:value={extractPageInput}
              min="1"
              disabled={$isLoading}
              placeholder="e.g., 3"
            />
          </div>
          <button
            class="secondary-button full-width"
            on:click={handleExtract}
            disabled={!selectedExtractFile || !extractPageInput || $isLoading}
          >
            Extract Page
          </button>
        </div>
      </section>
    </div>

    <!-- Parse Section (moved to bottom) -->
    <section class="card">
      <div class="card-header">
        <h2>📄 Parse Metadata</h2>
        <button
          class="clear-button"
          on:click={clearParseSelection}
          disabled={!selectedParseFile}
        >
          Clear
        </button>
      </div>
      <div class="card-content">
        <div class="button-group">
          <button
            class="primary-button"
            on:click={() => selectFile("parse")}
            disabled={$isLoading}
          >
            Select PDF
          </button>
          <button
            class="secondary-button"
            on:click={handleParse}
            disabled={!selectedParseFile || $isLoading}
          >
            Parse
          </button>
        </div>
        {#if selectedParseFile}
          <div class="file-badge">
            <span class="file-icon">📎</span>
            <span class="file-name">{selectedParseFile}</span>
          </div>
        {/if}
        {#if parseResult}
          <div class="metadata-container">
            <h3>📋 Metadata</h3>
            {#if Object.keys(parseResult).length > 0}
              <div class="metadata-grid">
                {#each Object.entries(parseResult) as [key, value]}
                  <div class="metadata-item">
                    <strong>{key}</strong>
                    <span>{value}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="empty-state">No metadata found in PDF</p>
            {/if}
          </div>
        {/if}
      </div>
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

  .app-container {
    min-height: 100vh;
    background: var(--gradient);
    padding: 2.5rem 1.5rem 2rem;
    font-family: "Manrope", "SF Pro Display", "Segoe UI", sans-serif;
    color: var(--text-color);
    position: relative;
    overflow: hidden;
  }

  .app-container::before,
  .app-container::after {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .app-container::before {
    background: radial-gradient(600px circle at 10% 20%, rgba(34, 211, 238, 0.08), transparent 40%);
  }

  .app-container::after {
    background: radial-gradient(700px circle at 90% 10%, rgba(99, 102, 241, 0.08), transparent 45%);
  }

  .hero {
    display: grid;
    grid-template-columns: 1.4fr 0.9fr;
    gap: 1.25rem;
    align-items: stretch;
    margin: 0 auto 2rem;
    max-width: 1200px;
    position: relative;
    z-index: 1;
  }

  .hero h1 {
    margin: 0 0 0.35rem;
    font-size: 2.6rem;
    letter-spacing: -0.02em;
  }

  .subtitle {
    color: var(--muted);
    line-height: 1.5;
    max-width: 720px;
  }

  .eyebrow {
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 0.8rem;
    color: var(--primary-accent);
    margin: 0 0 0.35rem;
  }

  .hero-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
    flex-wrap: wrap;
  }

  .chip {
    padding: 0.35rem 0.75rem;
    border-radius: 999px;
    font-size: 0.85rem;
    border: 1px solid var(--border-color);
    background: var(--surface);
    color: var(--text-color);
  }

  .chip.success {
    border-color: rgba(34, 197, 94, 0.4);
    color: #bbf7d0;
  }

  .chip.neutral {
    border-color: rgba(148, 163, 184, 0.35);
  }

  .chip.ghost {
    border-color: transparent;
    background: rgba(255, 255, 255, 0.06);
    color: var(--muted);
  }

  .hero-card {
    background: linear-gradient(160deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    border: 1px solid var(--border-color);
    border-radius: 1rem;
    padding: 1.25rem;
    box-shadow: var(--shadow-strong);
    backdrop-filter: blur(12px);
  }

  .hero-label {
    margin: 0 0 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 0.8rem;
    color: var(--muted);
  }

  .hero-steps {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .hero-steps div {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.7rem 0.85rem;
    border-radius: 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-color);
  }

  .step-num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--primary-color);
    color: #e0f2fe;
    font-weight: 700;
    font-size: 0.9rem;
    box-shadow: 0 12px 24px rgba(37, 99, 235, 0.25);
  }

  .muted {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .content-grid {
    max-width: 1200px;
    margin: 0 auto;
    gap: 1.5rem;
    display: flex;
    flex-direction: column;
    position: relative;
    z-index: 1;
  }

  .tools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
  }

  .card {
    background: var(--card-background);
    border-radius: 1rem;
    box-shadow: var(--shadow-strong);
    overflow: hidden;
    transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
    border: 1px solid var(--border-color);
    backdrop-filter: blur(12px);
  }

  .card:hover {
    transform: translateY(-3px);
    border-color: rgba(255, 255, 255, 0.16);
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.35);
  }

  .card-header {
    background: linear-gradient(120deg, rgba(37, 99, 235, 0.95), rgba(34, 211, 238, 0.7));
    color: #e2f3ff;
    padding: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .card-header h2 {
    margin: 0;
    font-size: 1.25rem;
  }

  .card-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
    box-sizing: border-box;
    background: rgba(255, 255, 255, 0.02);
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

  .input-group input:focus {
    outline: none;
    border-color: rgba(34, 211, 238, 0.7);
    box-shadow: 0 0 0 4px rgba(34, 211, 238, 0.12);
  }

  .metadata-container {
    margin-top: 1rem;
  }

  .metadata-grid {
    display: grid;
    gap: 1rem;
  }

  .metadata-item {
    padding: 0.9rem;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 0.65rem;
    display: flex;
    justify-content: space-between;
    border: 1px solid var(--border-color);
    color: var(--text-color);
  }

  .app-footer {
    margin-top: 2.5rem;
    padding: 1.25rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
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

  @media (max-width: 768px) {
    .app-container {
      padding: 0.75rem;
    }

    .hero {
      grid-template-columns: 1fr;
      gap: 1rem;
    }

    .content-grid {
      gap: 1rem;
    }

    .tools-grid {
      grid-template-columns: 1fr;
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
