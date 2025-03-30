<script lang="ts">
    // Import invoke from the core namespace (which remains available)
    import { invoke } from '@tauri-apps/api/core';
  
    // Local imports
    import StatusDisplay from '$lib/components/StatusDisplay.svelte';
    import { parsePageString } from '$lib/utils';
    import { isLoading, showStatus, startLoading } from '$lib/stores';
  
    // --- Reactive State ---
    let selectedParseFile: string | null = null;
    let parseResult: Record<string, string> | null = null;
  
    let selectedMergeFiles: string[] = [];
  
    let selectedSplitFile: string | null = null;
    let splitPagesInput: string = ''; // e.g., "1, 3-5, 8"
  
    let selectedExtractFile: string | null = null;
    let extractPageInput: number | null = null;
  
    // --- Helper functions using invoke (wrap native functionality) ---
  
    async function openFileDialog(multiple: boolean = false): Promise<string | string[] | null> {
        const result = await invoke('open_file_dialog', { multiple });
        if (Array.isArray(result)) {
            return multiple ? result : result[0] || null;
        }
        return null;
    }
  
    async function saveFileDialog(defaultPath: string): Promise<string | null> {
      // Similarly, implement a `save_file_dialog` command in Rust that returns the chosen save path
      return await invoke('save_file_dialog', { defaultPath });
    }
  
    async function getOsType(): Promise<string> {
      // Implement this command in Rust (for example, by calling std::env or tauri::api::os::)
      return await invoke('get_os_type');
    }
  
    async function shellOpen(filePath: string): Promise<void> {
      // Implement a Rust command (e.g. `shell_open`) that uses Tauri’s shell open functionality
      await invoke('shell_open', { filePath });
    }
  
    async function openPathInExplorer(filePath: string) {
      try {
        const osType = await getOsType();
        if (osType === 'Darwin' || osType === 'Linux') {
          await shellOpen(filePath);
        } else {
          // On Windows, open the parent directory
          const parentDir = filePath.split('/').slice(0, -1).join('/');
          await shellOpen(parentDir);
        }
      } catch (err) {
        console.error("Failed to open path:", err);
        showStatus(`Processing complete. Could not automatically open ${filePath}.`, false);
      }
    }
  
    // --- Event Handlers ---
  
    async function selectFile(target: 'parse' | 'split' | 'extract') {
      try {
        const result = await openFileDialog(false);
        if (result && typeof result === 'string') {
          if (target === 'parse') selectedParseFile = result;
          if (target === 'split') selectedSplitFile = result;
          if (target === 'extract') selectedExtractFile = result;
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
        const result: Record<string, string> = await invoke('parse_pdf', { path: selectedParseFile });
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
      const outputPath = await getSavePath('merged.pdf');
      if (!outputPath) return;
      startLoading("Merging PDFs...");
      try {
        await invoke('merge_pdfs', { paths: selectedMergeFiles, outputPath });
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
        showStatus("Invalid page format. Use numbers, commas, and hyphens (e.g., 1, 3-5, 8).", true);
        return;
      }
      if (pagesArray.length === 0) {
        showStatus("Please enter page numbers to extract.", true);
        return;
      }
      const outputPath = await getSavePath('split.pdf');
      if (!outputPath) return;
      startLoading("Splitting PDF...");
      try {
        await invoke('split_pdf', { path: selectedSplitFile, pages: pagesArray, outputPath });
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
        await invoke('extract_pdf_page', { path: selectedExtractFile, pageNumber: extractPageInput, outputPath });
        showStatus(`Page ${extractPageInput} extracted successfully to ${outputPath}.`, false);
        await openPathInExplorer(outputPath);
      } catch (err) {
        showStatus(`Error extracting page: ${err}`, true);
      }
    }
  </script>
  
  <main>
    <h1>PDF Manager</h1>
  
    <!-- Parse Section -->
    <section>
      <h2>Parse Metadata</h2>
      <button on:click={() => selectFile('parse')} disabled={$isLoading}>Select PDF</button>
      {#if selectedParseFile}
        <span>Selected: {selectedParseFile}</span>
      {/if}
      <button on:click={handleParse} disabled={!selectedParseFile || $isLoading}>Parse</button>
      {#if parseResult}
        <div class="result-box">
          <h3>Metadata:</h3>
          {#if Object.keys(parseResult).length > 0}
            <ul>
              {#each Object.entries(parseResult) as [key, value]}
                <li><strong>{key}:</strong> {value}</li>
              {/each}
            </ul>
          {:else}
            <p>(No metadata found in Info dictionary)</p>
          {/if}
        </div>
      {/if}
    </section>
  
    <!-- Merge Section -->
    <section>
      <h2>Merge PDFs</h2>
      <button on:click={selectMergeFiles} disabled={$isLoading}>Select PDFs to Merge</button>
      {#if selectedMergeFiles.length > 0}
        <ul class="file-list">
          {#each selectedMergeFiles as file}
            <li>{file}</li>
          {/each}
        </ul>
      {/if}
      <button on:click={handleMerge} disabled={selectedMergeFiles.length < 2 || $isLoading}>Merge</button>
    </section>
  
    <!-- Split Section -->
    <section>
      <h2>Split PDF</h2>
      <button on:click={() => selectFile('split')} disabled={$isLoading}>Select PDF</button>
      {#if selectedSplitFile}
        <span>Selected: {selectedSplitFile}</span>
      {/if}
      <div>
        <label for="split-pages">Pages (e.g., 1, 3-5, 8):</label>
        <input id="split-pages" type="text" bind:value={splitPagesInput} disabled={$isLoading} placeholder="1, 3-5, 8"/>
      </div>
      <button on:click={handleSplit} disabled={!selectedSplitFile || !splitPagesInput || $isLoading}>Split</button>
    </section>
  
    <!-- Extract Section -->
    <section>
      <h2>Extract Page</h2>
      <button on:click={() => selectFile('extract')} disabled={$isLoading}>Select PDF</button>
      {#if selectedExtractFile}
        <span>Selected: {selectedExtractFile}</span>
      {/if}
      <div>
        <label for="extract-page">Page Number:</label>
        <input id="extract-page" type="number" bind:value={extractPageInput} min="1" disabled={$isLoading} placeholder="e.g., 3"/>
      </div>
      <button on:click={handleExtract} disabled={!selectedExtractFile || !extractPageInput || $isLoading}>Extract</button>
    </section>
  </main>
  
  <StatusDisplay />
  
  <style>
    main {
      max-width: 800px;
      margin: 2em auto;
      font-family: sans-serif;
      padding: 1em;
    }
    section {
      border: 1px solid #ccc;
      border-radius: 5px;
      padding: 1em;
      margin-bottom: 1.5em;
    }
    h1, h2 {
      margin-top: 0;
      color: #333;
    }
    button {
      padding: 8px 15px;
      margin-right: 10px;
      margin-bottom: 10px;
      cursor: pointer;
      border: 1px solid #555;
      border-radius: 4px;
      background-color: #f0f0f0;
    }
    button:disabled {
      cursor: not-allowed;
      opacity: 0.6;
    }
    button:hover:not(:disabled) {
      background-color: #e0e0e0;
    }
    input[type="text"], input[type="number"] {
      padding: 8px;
      margin-left: 5px;
      border: 1px solid #ccc;
      border-radius: 4px;
    }
    label {
      margin-right: 5px;
    }
    span {
      margin-left: 10px;
      font-style: italic;
      color: #555;
      font-size: 0.9em;
    }
    .result-box {
      margin-top: 1em;
      padding: 0.8em;
      background-color: #f9f9f9;
      border: 1px solid #eee;
      border-radius: 4px;
      max-height: 200px;
      overflow-y: auto;
    }
    .result-box ul, .file-list {
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.9em;
    }
    .file-list li, .result-box li {
      margin-bottom: 5px;
      color: #555;
    }
    .result-box strong {
      color: #000;
    }
  </style>