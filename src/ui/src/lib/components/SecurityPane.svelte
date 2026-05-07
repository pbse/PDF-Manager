<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { pdfState } from "$lib/state/pdfState.svelte";
  import { chatState } from "$lib/state/chatState.svelte";
  import { appState } from "$lib/state/appState.svelte";
  import ToolPane from "./ToolPane.svelte";

  let piiItems = $state<{ type: string, value: string, page: number, rect: number[], selected: boolean }[]>([]);
  let redactPatterns = $state({ ssn: true, email: true, phone: true, financial: false });
  let isFindingPii = $state(false);

  async function handleFindPii() {
    if (!pdfState.selectedCryptoFile) return;
    isFindingPii = true;
    piiItems = [];
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: pdfState.selectedCryptoFile });
      if (pdfText.length > 50000) pdfText = pdfText.substring(0, 50000); 

      const activePatterns = Object.entries(redactPatterns).filter(([_, v]) => v).map(([k, _]) => k).join(", ");
      const system = `You are a privacy expert. Identify ONLY these PII types: ${activePatterns}. Return a JSON object with a 'pii' array containing objects with 'type' and 'value'. ONLY return JSON.`;
      const prompt = `TEXT FROM PDF:\n${pdfText}`;
      
      const result = await chatState.runAiTask(system, prompt, { json: true });
      const parsed = JSON.parse(result);
      const foundStrings = (parsed.pii || []).map((item: any) => item.value);
      
      if (foundStrings.length > 0) {
        const bytes = await invoke<number[]>("read_file_bytes", { path: pdfState.selectedCryptoFile });
        const uint8 = new Uint8Array(bytes);
        // @ts-ignore
        const pdfjs = await import("pdfjs-dist");
        const pdf = await pdfjs.getDocument({ data: uint8 }).promise;
        
        let matchedItems = [];
        for (let i = 1; i <= pdf.numPages; i++) {
          const page = await pdf.getPage(i);
          const textContent = await page.getTextContent();
          for (const item of textContent.items as any[]) {
            for (const pii of parsed.pii) {
              if (item.str.includes(pii.value)) {
                const [sx, sy, skx, sky, tx, ty] = item.transform;
                matchedItems.push({
                  type: pii.type,
                  value: pii.value,
                  page: i,
                  rect: [tx, ty, tx + item.width, ty + item.height],
                  selected: true
                });
              }
            }
          }
        }
        piiItems = matchedItems;
      }
      if (piiItems.length === 0) appState.showStatus("No PII detected.", false);
    } catch (e) {
      console.error(e);
      appState.showStatus("Error finding PII.", true);
    } finally {
      isFindingPii = false;
    }
  }

  async function handleRedactAll() {
    if (piiItems.length === 0 || !pdfState.selectedCryptoFile) return;
    const selected = piiItems.filter(i => i.selected);
    if (selected.length === 0) return;
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "redacted.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Redacting PII...");
    try {
      let currentPath = pdfState.selectedCryptoFile;
      for (const item of selected) {
        await invoke("add_annotation", {
          path: currentPath,
          page: item.page,
          rect: item.rect,
          kind: "redact",
          contents: null,
          color: null,
          output_path: outputPath
        });
        currentPath = outputPath;
      }
      appState.showStatus("PII redacted successfully.", false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) {
      console.error(e);
      appState.showStatus("Redaction failed.", true);
    }
  }

  async function handleCryptoSign() {
    if (!pdfState.selectedCryptoFile || !pdfState.signCertPath || !pdfState.signCertPassword) { 
      appState.showStatus("Missing certificate or password.", true); 
      return; 
    }
    const rectArray = pdfState.signRectInput.split(",").map(n => Number(n.trim()));
    if (rectArray.length !== 4) { appState.showStatus("Invalid signature area.", true); return; }
    
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "signed-crypto.pdf" });
    if (!outputPath) return;
    
    appState.startLoading("Signing (crypto)...");
    try {
      await invoke("sign_pdf_pfx", { 
        path: pdfState.selectedCryptoFile, 
        page: pdfState.viewerPageNumber, 
        rect: rectArray, 
        pfxPath: pdfState.signCertPath, 
        pfxPassword: pdfState.signCertPassword, 
        reason: pdfState.signReason || null, 
        location: pdfState.signLocation || null, 
        contact: pdfState.signContact || null, 
        outputPath 
      });
      appState.showStatus("Crypto signing completed.", false, outputPath);
      if (!pdfState.rememberPassword) pdfState.signCertPassword = "";
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Signing failed: ${err}`, true); }
  }

  let decryptPassword = $state("");

  async function handleFlatten() {
    if (!pdfState.selectedCryptoFile) { appState.showStatus("Please select a PDF file first.", true); return; }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "flattened.pdf" });
    if (!outputPath) return;
    appState.startLoading("Flattening annotations...");
    try {
      await invoke("flatten_annotations", { path: pdfState.selectedCryptoFile, outputPath });
      appState.showStatus(`PDF flattened successfully.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Flatten failed: ${err}`, true); }
  }

  async function handleForensicRedact() {
    if (!pdfState.selectedCryptoFile || !pdfState.annotationRectInput) {
      appState.showStatus("Please select an area to redact first.", true);
      return;
    }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "redacted_forensic.pdf" });
    if (!outputPath) return;

    appState.startLoading("Burning redaction (Forensic)...");
    try {
      const rectArray = pdfState.annotationRectInput.split(",").map(n => Number(n.trim()));
      await invoke("forensic_redact", {
        path: pdfState.selectedCryptoFile,
        pageNum: pdfState.viewerPageNumber,
        rect: rectArray,
        outputPath
      });
      appState.showStatus("Data permanently removed from document.", false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) { appState.showStatus(`Forensic Redact failed: ${e}`, true); }
  }

  async function handleDecrypt() {
    if (!pdfState.selectedCryptoFile || !decryptPassword) { appState.showStatus("Password required for decryption.", true); return; }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "decrypted.pdf" });
    if (!outputPath) return;
    appState.startLoading("Removing password protection...");
    try {
      await invoke("decrypt_pdf", { path: pdfState.selectedCryptoFile, password: decryptPassword, outputPath });
      appState.showStatus("Password removed successfully.", false, outputPath);
      decryptPassword = "";
      await invoke("shell_open", { filePath: outputPath });
    } catch (e) { appState.showStatus(`Decryption failed: ${e}`, true); }
  }

  async function handleCompress() {
    if (!pdfState.selectedCryptoFile) { appState.showStatus("Please select a PDF file first.", true); return; }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "compressed.pdf" });
    if (!outputPath) return;
    appState.startLoading("Compressing PDF...");
    try {
      await invoke("compress_pdf", { path: pdfState.selectedCryptoFile, outputPath });
      appState.showStatus(`PDF compressed successfully.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Compression failed: ${err}`, true); }
  }

  async function handleSanitize() {
    if (!pdfState.selectedCryptoFile) { appState.showStatus("Please select a PDF file first.", true); return; }
    const outputPath = await invoke<string | null>("save_file_dialog", { defaultPath: "sanitized.pdf" });
    if (!outputPath) return;
    appState.startLoading("Sanitizing PDF (Removing Metadata)...");
    try {
      await invoke("sanitize_pdf", { path: pdfState.selectedCryptoFile, outputPath });
      appState.showStatus(`PDF sanitized successfully.`, false, outputPath);
      await invoke("shell_open", { filePath: outputPath });
    } catch (err) { appState.showStatus(`Error sanitizing PDF: ${err}`, true); }
  }

  async function handleVerifySignatures() {
    if (!pdfState.selectedCryptoFile) return;
    appState.startLoading("Verifying signatures...");
    try {
      const results = await invoke<string[]>("verify_signatures", { path: pdfState.selectedCryptoFile });
      appState.showStatus(results.join(". "), false);
    } catch (e) {
      appState.showStatus(`Verification failed: ${e}`, true);
    }
  }

  function targetCurrent() {
    if (pdfState.viewerFilePath) {
      pdfState.setFileForTarget('security', pdfState.viewerFilePath);
    }
  }

  function openViewer(mode: "rect" | "view" = "rect") {
    const target = pdfState.selectedCryptoFile || pdfState.viewerFilePath;
    if (!target) {
      appState.showStatus("Please select a PDF file first.", true);
      return;
    }
    pdfState.viewerFilePath = target;
    pdfState.viewerMode = mode;
    pdfState.viewerTarget = "security";
  }

  async function selectCertFile() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) pdfState.signCertPath = result[0];
  }
</script>

<ToolPane title="Security">
  <div class="space-y-6">
    <div class="flex flex-col gap-2">
      {#if pdfState.viewerFilePath && pdfState.selectedCryptoFile !== pdfState.viewerFilePath}
        <button onclick={targetCurrent} class="w-full py-2 px-4 bg-blue-600 text-white rounded-md text-sm font-bold transition-all shadow-md hover:bg-blue-700 uppercase tracking-tight">
          Target Current Document
        </button>
      {/if}
      <button onclick={() => pdfState.selectFile('security')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
        {pdfState.selectedCryptoFile ? pdfState.selectedCryptoFile.split(/[/\\]/).pop() : 'Select PDF to Redact/Sign'}
      </button>
    </div>

    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Utilities</h3>
      <div class="flex gap-2">
        <button onclick={handleSanitize} disabled={!pdfState.selectedCryptoFile} class="flex-1 py-2 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-white dark:hover:bg-slate-800 transition-all shadow-sm">{!pdfState.selectedCryptoFile ? 'Select PDF' : 'Sanitize'}</button>
        <button onclick={handleFlatten} disabled={!pdfState.selectedCryptoFile} class="flex-1 py-2 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-white dark:hover:bg-slate-800 transition-all shadow-sm">{!pdfState.selectedCryptoFile ? 'Select PDF' : 'Flatten'}</button>
        <button onclick={handleCompress} disabled={!pdfState.selectedCryptoFile} class="flex-1 py-2 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-white dark:hover:bg-slate-800 transition-all shadow-sm">{!pdfState.selectedCryptoFile ? 'Select PDF' : 'Compress'}</button>
        <button onclick={handleVerifySignatures} disabled={!pdfState.selectedCryptoFile} class="flex-1 py-2 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-white dark:hover:bg-slate-800 transition-all shadow-sm">{!pdfState.selectedCryptoFile ? 'Select PDF' : 'Verify'}</button>
      </div>
    </div>

    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Master Decryption</h3>
      <div class="space-y-2">
        <label for="decrypt-pass" class="text-[9px] font-bold text-slate-500 uppercase">File Password</label>
        <div class="flex gap-2">
          <input id="decrypt-pass" type="password" bind:value={decryptPassword} placeholder="Enter password..." class="flex-1 p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs outline-none focus:ring-2 focus:ring-blue-500" />
          <button onclick={handleDecrypt} disabled={!pdfState.selectedCryptoFile || !decryptPassword} class="px-3 py-2 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded font-bold text-[9px] uppercase tracking-widest shadow-md">
            {!pdfState.selectedCryptoFile ? 'Select PDF' : !decryptPassword ? 'Enter Pwd' : 'Unlock'}
          </button>
        </div>
      </div>
    </div>

    <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
      <h3 class="text-[10px] font-bold text-red-600 dark:text-red-400 uppercase tracking-widest transition-colors">AI Auto-Redact PII</h3>
      <div class="grid grid-cols-2 gap-2">
         <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={redactPatterns.ssn} class="w-2.5 h-2.5 rounded text-red-600" />
            <span class="text-[8px] font-black text-slate-500 uppercase">SSN</span>
         </label>
         <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={redactPatterns.email} class="w-2.5 h-2.5 rounded text-red-600" />
            <span class="text-[8px] font-black text-slate-500 uppercase">Email</span>
         </label>
         <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={redactPatterns.phone} class="w-2.5 h-2.5 rounded text-red-600" />
            <span class="text-[8px] font-black text-slate-500 uppercase">Phone</span>
         </label>
         <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={redactPatterns.financial} class="w-2.5 h-2.5 rounded text-red-600" />
            <span class="text-[8px] font-black text-slate-500 uppercase">Money</span>
         </label>
      </div>
      
      {#if piiItems.length > 0}
        <div class="max-h-40 overflow-y-auto space-y-2 border-t border-slate-200 dark:border-slate-800 pt-3">
          {#each piiItems as item, i}
            <label class="flex items-center gap-2 p-1.5 rounded hover:bg-white dark:hover:bg-slate-800 transition-colors cursor-pointer">
              <input type="checkbox" bind:checked={piiItems[i].selected} class="rounded text-blue-600 w-3 h-3" />
              <div class="flex flex-col min-w-0">
                <span class="text-[9px] font-bold text-slate-400 uppercase">{item.type}</span>
                <span class="text-[10px] text-slate-700 dark:text-slate-200 truncate">{item.value}</span>
              </div>
            </label>
          {/each}
        </div>
        <button onclick={handleRedactAll} class="w-full py-2 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded font-bold text-[10px] uppercase tracking-widest shadow-md">
          {!pdfState.selectedCryptoFile ? 'Select PDF' : 'Apply AI Redactions'}
        </button>
        <button onclick={() => piiItems = []} class="w-full text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter">Clear List</button>
      {:else}
        <div class="flex gap-2">
           <button onclick={handleFindPii} disabled={!pdfState.selectedCryptoFile || isFindingPii} class="flex-[2] py-2 border border-red-600 text-red-600 dark:text-red-400 rounded font-bold text-[9px] uppercase tracking-widest hover:bg-red-50 dark:hover:bg-red-900/20 transition-all shadow-sm">
             {!pdfState.selectedCryptoFile ? 'Select PDF' : isFindingPii ? 'Finding PII...' : 'Auto-Redact PII'}
           </button>
           <button 
             onclick={() => { 
                if(!pdfState.selectedCryptoFile) return;
                pdfState.viewerFilePath = pdfState.selectedCryptoFile;
                pdfState.viewerMode = 'rect';
                pdfState.viewerTarget = 'security';
             }}
             class="flex-1 py-2 bg-red-600 text-white rounded font-bold text-[9px] uppercase tracking-widest shadow-md"
           >
             {!pdfState.selectedCryptoFile ? 'Select PDF' : 'Forensic Burn'}
           </button>
        </div>
      {/if}
      {#if pdfState.annotationRectInput && pdfState.viewerTarget === 'security'}
         <button onclick={handleForensicRedact} class="w-full py-2 bg-red-600 text-white rounded-lg font-black text-[9px] uppercase tracking-widest animate-pulse">Confirm Area Burn</button>
      {/if}
    </div>

    <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900 transition-colors">
      <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Digital Signature</h3>
      <button onclick={selectCertFile} class="w-full py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 rounded text-[10px] font-bold uppercase tracking-wider truncate px-3 transition-colors shadow-sm">
        {pdfState.signCertPath ? pdfState.signCertPath.split(/[/\\]/).pop() : 'Select PFX Certificate'}
      </button>

      <div class="space-y-4">
        <div class="space-y-1.5">
          <label for="cert-password" class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Cert Password</label>
          <input id="cert-password" type="password" bind:value={pdfState.signCertPassword} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" />
          <label class="flex items-center gap-2 cursor-pointer mt-1">
            <input type="checkbox" bind:checked={pdfState.rememberPassword} class="rounded border-slate-300 text-blue-600 focus:ring-blue-500 w-3 h-3 transition-colors" />
            <span class="text-[9px] text-slate-500 font-bold uppercase tracking-tighter transition-colors">Remember for this session</span>
          </label>
        </div>

        <div class="space-y-1.5">
          <label for="security-rect" class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Signature Area</label>
          <div class="flex gap-2">
            <input id="security-rect" type="text" bind:value={pdfState.signRectInput} class="flex-1 p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors font-mono shadow-sm" placeholder="x1, y1, x2, y2" />
            <button onclick={() => openViewer('rect')} class="p-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded border border-blue-100 transition-colors hover:bg-blue-100">🎯</button>
          </div>
        </div>


        <button onclick={handleCryptoSign} disabled={!pdfState.selectedCryptoFile || !pdfState.signCertPath || !pdfState.signCertPassword} class="w-full py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest shadow-xl transition-all hover:scale-[1.02]">
          {!pdfState.selectedCryptoFile ? 'Select PDF' : !pdfState.signCertPath ? 'Select Cert' : !pdfState.signCertPassword ? 'Enter Pwd' : 'Sign Permanently'}
        </button>
      </div>
    </div>
  </div>
</ToolPane>
