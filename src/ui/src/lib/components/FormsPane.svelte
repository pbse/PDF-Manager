<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let fields = $state<{ name: string, field_type: string, value: string }[]>([]);
  let fieldUpdates = $state<Record<string, string>>({});
  let createdFields = $state<{ name: string, field_type: string, page: number, rect: number[] }[]>([]);
  let isScanning = $state(false);
  let isBuilderMode = $state(false);
  let newFieldName = $state("");
  let newFieldType = $state("Tx");

  async function scanFields() {
    if (!pdfState.viewerFilePath) return;
    isScanning = true;
    try {
      fields = await invoke("get_form_fields", { path: pdfState.viewerFilePath });
      fieldUpdates = {};
      fields.forEach(f => fieldUpdates[f.name] = f.value);
      if (fields.length === 0) appState.showStatus("No interactive fields found. Try Builder Mode to add some!", false);
    } catch (e) {
      appState.showStatus(`Scan failed: ${e}`, true);
    } finally {
      isScanning = false;
    }
  }

  function toggleBuilderMode() {
    isBuilderMode = !isBuilderMode;
    if (isBuilderMode) {
      pdfState.viewerMode = "rect";
      pdfState.viewerTarget = "annotate"; // Reuse annotate target for rect selection
      appState.showStatus("Builder Mode: Select areas on the PDF to add fields.", false);
    } else {
      pdfState.viewerMode = "view";
    }
  }

  function addFieldFromSelection() {
    if (!pdfState.annotationRectInput) {
      appState.showStatus("Please select an area on the PDF first.", true);
      return;
    }
    const rect = pdfState.annotationRectInput.split(",").map(Number);
    createdFields = [...createdFields, {
      name: newFieldName || `Field_${createdFields.length + 1}`,
      field_type: newFieldType,
      page: pdfState.viewerPageNumber,
      rect
    }];
    newFieldName = "";
    pdfState.annotationRectInput = "";
    appState.showStatus("Field added to layout.", false);
  }

  async function handleSaveForm() {
    if (!pdfState.viewerFilePath) return;
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "document_with_form.pdf" });
    if (!outputPath) return;

    appState.startLoading("Saving form...");
    try {
      let currentPath = pdfState.viewerFilePath;

      // 1. If we have created fields, apply them first
      if (createdFields.length > 0) {
        await invoke("create_form_fields", {
          path: currentPath,
          newFields: createdFields,
          outputPath
        });
        currentPath = outputPath;
      }

      // 2. Apply field updates (filling)
      if (Object.keys(fieldUpdates).length > 0) {
        await invoke("set_form_fields", { 
          path: currentPath, 
          updates: fieldUpdates, 
          outputPath 
        });
      }

      appState.showStatus("Form saved successfully.", false, outputPath);
      createdFields = [];
      isBuilderMode = false;
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      appState.showStatus(`Failed to save: ${e}`, true);
    }
  }

  async function handleAiDetectFields() {
    if (!pdfState.viewerFilePath) return;
    appState.startLoading("AI is analyzing document structure...");
    try {
      // Mocking AI detection for now: in reality, we'd use a vision-based heuristic or local LLM
      // For Phase 2, we implement the "AI suggestion" logic
      const text = await invoke<string>("pdf_to_text_string", { path: pdfState.viewerFilePath });
      const lines = text.split("\n");
      const suggestions = [];
      
      // Heuristic: lines ending with ":" or containing "____" are often fields
      for (let i = 0; i < Math.min(lines.length, 20); i++) {
        if (lines[i].includes(":") || lines[i].includes("___")) {
          suggestions.push(lines[i].split(":")[0].trim());
        }
      }

      if (suggestions.length > 0) {
        appState.showStatus(`AI detected ${suggestions.length} potential fields. Click to place them.`, false);
        // In a real implementation, we'd return coordinates from the backend
      } else {
        appState.showStatus("AI couldn't find obvious fields. Manual placement required.", false);
      }
    } catch (e) {
      appState.showStatus("AI detection failed.", true);
    }
  }
</script>

<ToolPane title="Forms" subtitle="Interactive Filler">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      <button 
        onclick={() => pdfState.selectFile('annotate')} 
        class="w-full py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold truncate transition-colors"
      >
        {pdfState.viewerFilePath ? pdfState.viewerFilePath.split(/[/\\]/).pop() : 'Select PDF Form'}
      </button>

      <div class="flex gap-2">
        <button 
          onclick={() => !pdfState.viewerFilePath ? pdfState.selectFile('annotate') : scanFields()} 
          disabled={!!pdfState.viewerFilePath && isScanning}
          class="flex-1 py-2 bg-blue-600 text-white rounded font-bold text-[10px] uppercase tracking-widest shadow-md"
        >
          {isScanning ? 'Scanning...' : 'Scan Fields'}
        </button>
        <button 
          onclick={toggleBuilderMode}
          class="flex-1 py-2 border rounded font-bold text-[10px] uppercase tracking-widest transition-all {isBuilderMode ? 'bg-slate-900 dark:bg-white text-white dark:text-slate-900 border-slate-900' : 'bg-white dark:bg-slate-900 text-slate-400 border-slate-200 dark:border-slate-800'}"
        >
          {isBuilderMode ? 'Stop Builder' : 'Builder Mode'}
        </button>
      </div>
    </div>

    {#if isBuilderMode}
      <div class="space-y-4 p-4 bg-blue-50/50 dark:bg-blue-900/10 rounded-xl border border-blue-100 dark:border-blue-900/30 animate-in fade-in slide-in-from-top-2">
        <div class="flex justify-between items-center">
          <h3 class="text-[10px] font-black text-blue-600 uppercase tracking-widest">Add New Field</h3>
          <button onclick={handleAiDetectFields} class="text-[8px] font-black text-blue-500 uppercase tracking-tighter">✨ AI Detect</button>
        </div>
        
        <div class="space-y-3">
          <div class="space-y-1">
            <label for="new-field-name" class="text-[9px] font-bold text-slate-500 uppercase">Field Name</label>
            <input id="new-field-name" type="text" bind:value={newFieldName} placeholder="e.g. FirstName" class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
          </div>

          <div class="space-y-1">
            <label for="new-field-type" class="text-[9px] font-bold text-slate-500 uppercase">Type</label>
            <select id="new-field-type" bind:value={newFieldType} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500">
              <option value="Tx">Text Field</option>
              <option value="Btn">Checkbox</option>
              <option value="Ch">Choice / Dropdown</option>
            </select>
          </div>

          <div class="space-y-1">
             <label for="new-field-rect" class="text-[9px] font-bold text-slate-500 uppercase">Selected Area</label>
             <div class="flex gap-2">
                <input id="new-field-rect" type="text" readonly value={pdfState.annotationRectInput} placeholder="Select on PDF..." class="flex-1 p-2 bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded text-[10px] font-mono text-slate-500" />
                <button onclick={addFieldFromSelection} class="px-3 bg-blue-600 text-white rounded font-bold text-[10px] uppercase">Add</button>
             </div>
          </div>
        </div>

        {#if createdFields.length > 0}
          <div class="pt-4 border-t border-blue-100 dark:border-blue-900/30">
            <div class="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-2">To be created ({createdFields.length})</div>
            <div class="max-h-32 overflow-y-auto space-y-2">
              {#each createdFields as cf, i}
                <div class="flex items-center justify-between p-2 bg-white dark:bg-slate-800 rounded border border-slate-100 dark:border-slate-700">
                  <span class="text-[10px] font-bold text-slate-700 dark:text-slate-300 truncate max-w-[120px]">{cf.name}</span>
                  <button onclick={() => createdFields = createdFields.filter((_, idx) => idx !== i)} class="text-red-500 text-[10px]">✕</button>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    {#if fields.length > 0 || createdFields.length > 0}
      <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
        <div class="flex justify-between items-center">
          <h3 class="text-[10px] font-black text-slate-400 uppercase tracking-widest">Fields Found ({fields.length})</h3>
          <button onclick={() => appState.showStatus("Auto-filling based on profile...", false)} class="text-[8px] font-black text-blue-500 uppercase tracking-tighter">✨ AI Fill</button>
        </div>

        {#if fields.length > 0}
          <div class="max-h-96 overflow-y-auto space-y-3 pr-1">
            {#each fields as field}
              <div class="space-y-1">
                <label for="field-{field.name}" class="text-[9px] font-bold text-slate-500 uppercase truncate block">{field.name}</label>
                {#if field.field_type === 'Btn'}
                  <label class="flex items-center gap-2 cursor-pointer">
                      <input type="checkbox" checked={fieldUpdates[field.name] === 'Yes'} onchange={(e) => fieldUpdates[field.name] = (e.target as HTMLInputElement).checked ? 'Yes' : 'Off'} class="w-3 h-3 rounded text-blue-600" />
                      <span class="text-[10px] text-slate-600 dark:text-slate-400">Checked</span>
                  </label>
                {:else}
                  <input 
                    id="field-{field.name}"
                    type="text" 
                    bind:value={fieldUpdates[field.name]} 
                    class="w-full p-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" 
                  />
                {/if}
              </div>
            {/each}
          </div>
        {/if}

        <button 
          onclick={() => !pdfState.viewerFilePath ? pdfState.selectFile('annotate') : handleSaveForm()} 
          class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-xl font-bold text-[10px] uppercase tracking-widest shadow-xl"
        >
          {!pdfState.viewerFilePath ? 'Select PDF' : 'Save & Export Document'}
        </button>
      </div>
    {/if}
  </div>
</ToolPane>
