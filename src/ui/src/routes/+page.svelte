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
  <header class="app-header">
    <h1>PDF Manager</h1>
    <p class="subtitle">A modern tool for managing your PDF files</p>
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
    <a
      href="https://github.com/sponsors/pbse"
      target="_blank"
      rel="noopener noreferrer"
      class="sponsor-button"
    >
      ❤️ Sponsor this project
    </a>
    <p class="footer-text">Made with ❤️ using Tauri and Svelte</p>
  </footer>
</div>

<StatusDisplay />

<style>
  :root {
    --primary-color: #2563eb;
    --secondary-color: #1e40af;
    --background-color: #f8fafc;
    --card-background: #ffffff;
    --text-color: #1e293b;
    --border-color: #e2e8f0;
    --hover-color: #dbeafe;
    --disabled-color: #94a3b8;
  }

  .app-container {
    min-height: 100vh;
    background-color: var(--background-color);
    padding: 2rem;
    font-family:
      system-ui,
      -apple-system,
      sans-serif;
  }

  .app-header {
    text-align: center;
    margin-bottom: 3rem;
  }

  .app-header h1 {
    font-size: 2.5rem;
    color: var(--primary-color);
    margin: 0;
  }

  .subtitle {
    color: var(--text-color);
    opacity: 0.8;
  }

  .content-grid {
    max-width: 1200px; /* Reduced from 1200px */
    margin: 0 auto;
    gap: 1.5rem; /* Reduced from 2rem */
    display: flex;
    flex-direction: column;
  }

  .tools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
  }

  .card {
    background: var(--card-background);
    border-radius: 1rem;
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    overflow: hidden;
    transition: transform 0.2s;
  }

  .card:hover {
    transform: translateY(-2px);
  }

  .card-header {
    background: var(--primary-color);
    color: white;
    padding: 1rem; /* Reduced from 1.5rem */
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
  }

  .primary-button {
    background: var(--primary-color);
    color: white;
  }

  .secondary-button {
    background: var(--secondary-color);
    color: white;
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
    background: var(--hover-color);
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    margin: 0.5rem 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .input-group {
    margin: 1rem 0;
    width: 100%; /* Ensure input group takes full width */
    box-sizing: border-box;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--text-color);
    font-size: 0.9rem;
  }

  .input-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    font-size: 1rem;
    box-sizing: border-box; /* Include padding in width calculation */
    min-width: 0; /* Prevent input from overflowing */
  }

  .metadata-container {
    margin-top: 1rem;
  }

  .metadata-grid {
    display: grid;
    gap: 1rem;
  }

  .metadata-item {
    padding: 0.75rem;
    background: var(--background-color);
    border-radius: 0.5rem;
    display: flex;
    justify-content: space-between;
  }

  .app-footer {
    text-align: center;
    margin-top: 3rem;
    padding: 2rem;
    border-top: 1px solid var(--border-color);
  }

  .sponsor-button {
    display: inline-block;
    background: #db2777;
    color: white;
    padding: 0.75rem 2rem;
    border-radius: 9999px;
    text-decoration: none;
    font-weight: 500;
    margin-bottom: 1rem;
    transition: all 0.2s;
  }

  .sponsor-button:hover {
    transform: scale(1.05);
    background: #be185d;
  }

  .footer-text {
    color: var(--text-color);
    opacity: 0.7;
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

    .content-grid {
      gap: 1rem; /* Reduced from 1.5rem */
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
