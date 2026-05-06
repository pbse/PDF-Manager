<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount, tick } from "svelte";
  import * as webllm from "@mlc-ai/web-llm";
  import Tesseract from "tesseract.js";

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
    | "security"
    | "extract"
    | "crypto";

  let selectedParseFile = $state<string | null>(null);
  let selectedSplitFile = $state<string | null>(null);
  let selectedRotateFile = $state<string | null>(null);
  let selectedDeleteFile = $state<string | null>(null);
  let selectedAnnotateFile = $state<string | null>(null);
  let selectedSignatureFile = $state<string | null>(null);
  let selectedCryptoFile = $state<string | null>(null);
  let selectedExtractFile = $state<string | null>(null);

  // --- AI Chat State ---
  let chatHistory = $state<{role: "user" | "assistant" | "system", content: string}[]>([]);
  let chatInput = $state("");
  let isChatting = $state(false);
  let aiProvider = $state<"ollama" | "webllm">("ollama");
  let ollamaStatus = $state<"checking" | "connected" | "not_running">("checking");
  let ollamaModel = $state("llama3.2:1b");
  
  let engine = $state<webllm.MLCEngine | null>(null);
  let modelLoadingProgress = $state("");
  let isModelLoading = $state(false);
  const MODEL_ID = "Llama-3.2-1B-Instruct-q4f16_1-MLC"; 

  async function checkOllama() {
    ollamaStatus = "checking";
    try {
      const resp = await fetch("http://localhost:11434/api/tags");
      if (resp.ok) ollamaStatus = "connected";
      else ollamaStatus = "not_running";
    } catch {
      ollamaStatus = "not_running";
    }
  }

  async function autoOcrPdf(path: string, maxPages = 3): Promise<string> {
    try {
      const bytes = await invoke<number[]>("read_file_bytes", { path });
      const uint8 = new Uint8Array(bytes);
      
      // @ts-ignore
      const pdfjs = await import("pdfjs-dist");
      pdfjs.GlobalWorkerOptions.workerSrc = new URL(
        "pdfjs-dist/build/pdf.worker.mjs",
        import.meta.url
      ).toString();

      const loadingTask = pdfjs.getDocument({ data: uint8 });
      const pdf = await loadingTask.promise;
      
      let combinedText = "";
      const numPages = Math.min(pdf.numPages, maxPages);

      for (let i = 1; i <= numPages; i++) {
        const page = await pdf.getPage(i);
        const viewport = page.getViewport({ scale: 1.5 });
        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d")!;
        canvas.height = viewport.height;
        canvas.width = viewport.width;

        await page.render({ canvasContext: context, viewport }).promise;
        const dataUrl = canvas.toDataURL("image/png");
        
        const { data: { text } } = await Tesseract.recognize(dataUrl, "eng");
        combinedText += `\n[Page ${i} OCR]:\n${text}\n`;
      }
      return combinedText;
    } catch (e) {
      console.error("Auto-OCR failed:", e);
      return "";
    }
  }

  async function resetEngine() {
    if (engine) {
      await engine.unload();
      engine = null;
    }
    chatHistory = [];
    showStatus("AI Engine reset. It will re-initialize on your next question.", false);
  }

  async function runAiTask(system: string, prompt: string, options: { json?: boolean } = {}): Promise<string> {
    if (aiProvider === "ollama") {
      const response = await fetch("http://localhost:11434/api/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model: ollamaModel,
          system,
          prompt,
          stream: false,
          format: options.json ? "json" : undefined,
          options: {
            num_ctx: 32768,
            temperature: 0.2
          }
        })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(`Ollama Error: ${errorData.error || "Failed to connect"}`);
      }

      const data = await response.json();
      return data.response;
    } else {
      // WebLLM
      if (!engine) {
        isModelLoading = true;
        engine = await webllm.CreateMLCEngine(MODEL_ID, {
          initProgressCallback: (report) => { modelLoadingProgress = report.text; }
        });
        isModelLoading = false;
      }

      const messages: webllm.ChatCompletionMessageParam[] = [
        { role: "system", content: system },
        { role: "user", content: prompt }
      ];

      const reply = await engine.chat.completions.create({ 
        messages,
        response_format: options.json ? { type: "json_object" } : undefined
      });
      return reply.choices[0].message.content || "";
    }
  }

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

  // --- Viral Features State ---
  let autoRenameFiles = $state<{ path: string, originalName: string, newName: string, status: 'pending' | 'processing' | 'done' | 'error' }[]>([]);
  let isRenaming = $state(false);

  let batchExtractFiles = $state<{ path: string, status: 'pending' | 'processing' | 'done' | 'error', data?: any }[]>([]);
  let batchExtractPrompt = $state("Extract Vendor Name, Date, Total Amount");
  let batchExtractResults = $state<any[]>([]);
  let isBatchExtracting = $state(false);

  let piiItems = $state<{ type: string, value: string, page: number, rect: number[], selected: boolean }[]>([]);
  let isFindingPii = $state(false);

  // --- Active Tool State ---
  type ToolId = "merge" | "annotate" | "signature" | "security" | "split" | "organize" | "extract";
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
  let ocrTrigger = $state(0);

  let currentPreviewRect = $derived(
    activeTool === "annotate"
      ? parseRect(annotationRectInput)
      : activeTool === "security"
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
        : activeTool === "security"
          ? "#3b82f6"
          : "#6366f1"
  );

  function openViewer(target: SelectionTarget, mode: "rect" | "points" | "view" = "rect") {
    let path = "";
    switch (target) {
      case "annotate": path = selectedAnnotateFile || ""; break;
      case "signature": path = selectedSignatureFile || ""; break;
      case "security": path = selectedCryptoFile || ""; break;
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
        case "security": signRectInput = rectStr; break;
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

  async function handleExtractText() {
    if (!selectedExtractFile) { showStatus("Please select a PDF file first.", true); return; }
    const outputPath = await getSavePath("extracted.txt");
    if (!outputPath) return;
    startLoading("Extracting text from PDF...");
    try {
      await invoke("pdf_to_text", { path: selectedExtractFile, outputPath });
      showStatus(`Text extracted successfully to ${outputPath}.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error extracting text: ${err}`, true); }
  }

  async function handleAskPdf() {
    if (!selectedExtractFile) { showStatus("Please select a PDF file first.", true); return; }
    if (!chatInput.trim()) return;

    chatHistory = [...chatHistory, { role: "user", content: chatInput }];
    const question = chatInput;

    if (question.toLowerCase() === "debug text") {
      isChatting = true;
      try {
        const pdfText = await invoke<string>("pdf_to_text_string", { path: selectedExtractFile });
        chatHistory = [...chatHistory, { role: "assistant", content: `DEBUG: Extracted ${pdfText.length} characters. Preview: ${pdfText.substring(0, 500)}...` }];
      } catch (e: any) {
        chatHistory = [...chatHistory, { role: "assistant", content: `DEBUG ERROR: ${e.message}` }];
      } finally {
        isChatting = false;
        chatInput = "";
      }
      return;
    }

    chatInput = "";
    isChatting = true;

    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: selectedExtractFile });
      
      if (pdfText.trim().length === 0) {
        chatHistory = [...chatHistory, { 
          role: "assistant", 
          content: "This document appears to be a scan. I'm running OCR on the first few pages to understand it... please wait." 
        }];
        pdfText = await autoOcrPdf(selectedExtractFile);
        
        if (pdfText.trim().length === 0) {
          chatHistory = [...chatHistory, { 
            role: "assistant", 
            content: "Sorry, I couldn't extract any text even with OCR. This document might be too blurry or encrypted." 
          }];
          return;
        }
      }

      const system = "You are a precise document analysis assistant. Your ONLY source of information is the 'CONTEXT FROM PDF' provided below. Answer the user's question accurately using that context. If the answer is not in the context, say: 'I cannot find that information in the document.' Do not use outside knowledge.";
      const prompt = `### CONTEXT FROM PDF:\n${pdfText}\n\n### USER QUESTION:\n${question}`;
      
      const assistantMessage = await runAiTask(system, prompt);
      chatHistory = [...chatHistory, { role: "assistant", content: assistantMessage }];
    } catch (e: any) {
      console.error(e);
      chatHistory = [...chatHistory, { role: "assistant", content: `Error: ${e.message}` }];
    } finally {
      isChatting = false;
      isModelLoading = false;
    }
  }

  async function handleSanitize() {
    if (!selectedCryptoFile) { showStatus("Please select a PDF file first.", true); return; }
    const outputPath = await getSavePath("sanitized.pdf");
    if (!outputPath) return;
    startLoading("Sanitizing PDF (Removing Metadata)...");
    try {
      await invoke("sanitize_pdf", { path: selectedCryptoFile, outputPath });
      showStatus(`PDF sanitized successfully.`, false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (err) { showStatus(`Error sanitizing PDF: ${err}`, true); }
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

  async function handleSecureShare() {
    const file = selectedSignatureFile || selectedCryptoFile;
    if (!file) return;
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: file });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);

      const system = "Draft a professional email sharing a signed document. Mention privacy. Return ONLY the Subject and Body.";
      const draft = await runAiTask(system, `TEXT:\n${pdfText}`);
      
      const subject = `Signed Document: ${file.split(/[/\\]/).pop()}`;
      await invoke("shell_open", { filePath: `mailto:?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(draft)}` });
      showStatus("Opening mail client...", false);
    } catch (e) {
      console.error(e);
      showStatus("Error sharing.", true);
    }
  }

  async function getSavePath(defaultFilename: string): Promise<string | null> {
    return await invoke("save_file_dialog", { defaultPath: defaultFilename });
  }

  async function handleAutoRename() {
    if (autoRenameFiles.length === 0) return;
    isRenaming = true;
    for (let i = 0; i < autoRenameFiles.length; i++) {
      const file = autoRenameFiles[i];
      if (file.status === 'done') continue;
      
      autoRenameFiles[i].status = 'processing';
      try {
        let pdfText = await invoke<string>("pdf_to_text_string", { path: file.path });
        if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000); 

        const system = "You are a file naming assistant. Based on the document text, suggest a concise, descriptive filename ending in .pdf. Use YYYY-MM-DD_Subject.pdf format if possible. Return ONLY the filename string.";
        const prompt = `TEXT FROM PDF:\n${pdfText}`;
        
        let newName = await runAiTask(system, prompt);
        newName = newName.trim().replace(/["']/g, "").replace(/[\/:*?"<>|]/g, "_");
        if (!newName.toLowerCase().endsWith(".pdf")) newName += ".pdf";
        
        const lastSlash = Math.max(file.path.lastIndexOf("/"), file.path.lastIndexOf("\\"));
        const oldDir = lastSlash !== -1 ? file.path.substring(0, lastSlash + 1) : "";
        const newPath = oldDir + newName;
        
        await invoke("rename_file", { oldPath: file.path, newPath });
        autoRenameFiles[i].newName = newName;
        autoRenameFiles[i].path = newPath; // Update path for next operations
        autoRenameFiles[i].status = 'done';
      } catch (e) {
        console.error(e);
        autoRenameFiles[i].status = 'error';
      }
    }
    isRenaming = false;
    showStatus("Auto-rename process finished.", false);
  }

  async function selectAutoRenameFiles() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: true });
    if (result && result.length > 0) {
      autoRenameFiles = result.map(p => ({
        path: p,
        originalName: p.split(/[/\\]/).pop() || p,
        newName: "",
        status: 'pending' as const
      }));
    }
  }

  async function handleBatchExtract() {
    if (batchExtractFiles.length === 0) return;
    isBatchExtracting = true;
    batchExtractResults = [];
    
    for (let i = 0; i < batchExtractFiles.length; i++) {
      batchExtractFiles[i].status = 'processing';
      try {
        let pdfText = await invoke<string>("pdf_to_text_string", { path: batchExtractFiles[i].path });
        if (pdfText.length > 50000) pdfText = pdfText.substring(0, 50000);

        const system = `You are a data extraction assistant. Extract information as requested by the user. Return a JSON object. ONLY return JSON.`;
        const prompt = `USER REQUEST: ${batchExtractPrompt}\n\nTEXT FROM PDF:\n${pdfText}`;
        
        const result = await runAiTask(system, prompt, { json: true });
        const data = JSON.parse(result);
        
        batchExtractFiles[i].data = data;
        batchExtractFiles[i].status = 'done';
        batchExtractResults = [...batchExtractResults, { filename: batchExtractFiles[i].path.split(/[/\\]/).pop(), ...data }];
      } catch (e) {
        console.error(e);
        batchExtractFiles[i].status = 'error';
      }
    }
    isBatchExtracting = false;
    showStatus("Batch extraction complete.", false);
  }

  async function selectBatchFiles() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: true });
    if (result && result.length > 0) {
      batchExtractFiles = result.map(p => ({ path: p, status: 'pending' }));
    }
  }

  function exportBatchToCsv() {
    if (batchExtractResults.length === 0) return;
    const headers = Object.keys(batchExtractResults[0]);
    const csvRows = [];
    csvRows.push(headers.join(","));
    for (const row of batchExtractResults) {
      const values = headers.map(header => {
        const val = row[header];
        return `"${String(val).replace(/"/g, '""')}"`;
      });
      csvRows.push(values.join(","));
    }
    const csvContent = csvRows.join("\n");
    getSavePath("extraction_results.csv").then(path => {
      if (path) invoke("write_text_file", { path, contents: csvContent }).then(() => openPathInExplorer(path));
    });
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
        case "security": 
          if (selectedCryptoFile !== result) piiItems = [];
          selectedCryptoFile = result; 
          break;
        case "extract": 
          if (selectedExtractFile !== result) chatHistory = [];
          selectedExtractFile = result; 
          break;
        case "crypto": selectedCryptoFile = result; break;
      }
      viewerFilePath = result;
    }
  }

  async function handleFindPii() {
    if (!selectedCryptoFile) return;
    isFindingPii = true;
    piiItems = [];
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: selectedCryptoFile });
      if (pdfText.length > 50000) pdfText = pdfText.substring(0, 50000); 

      const system = "You are a privacy expert. Identify all PII (Personally Identifiable Information) in the text below. return a JSON object with a 'pii' array containing objects with 'type' (e.g. SSN, Email, Name, Phone) and 'value'. ONLY return JSON.";
      const prompt = `TEXT FROM PDF:\n${pdfText}`;
      
      const result = await runAiTask(system, prompt, { json: true });
      const parsed = JSON.parse(result);
      const foundStrings = (parsed.pii || []).map((item: any) => item.value);
      
      if (foundStrings.length > 0) {
        const bytes = await invoke<number[]>("read_file_bytes", { path: selectedCryptoFile });
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
      if (piiItems.length === 0) showStatus("No PII detected.", false);
    } catch (e) {
      console.error(e);
      showStatus("Error finding PII.", true);
    } finally {
      isFindingPii = false;
    }
  }

  async function handleRedactAll() {
    if (piiItems.length === 0 || !selectedCryptoFile) return;
    const selected = piiItems.filter(i => i.selected);
    if (selected.length === 0) return;
    
    const outputPath = await getSavePath("redacted.pdf");
    if (!outputPath) return;
    
    startLoading("Redacting PII...");
    try {
      // We apply them sequentially to the same file for now
      // A batch redact command in backend would be more efficient, but let's reuse add_annotation
      let currentPath = selectedCryptoFile;
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
        currentPath = outputPath; // Apply next on the output
      }
      showStatus("PII redacted successfully.", false, outputPath);
      await openPathInExplorer(outputPath);
    } catch (e) {
      console.error(e);
      showStatus("Redaction failed.", true);
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
      if (selectedExtractFile !== path) chatHistory = [];
      selectedSplitFile = selectedRotateFile = selectedDeleteFile = selectedAnnotateFile = selectedSignatureFile = selectedCryptoFile = selectedExtractFile = path;
      viewerFilePath = path;
      showStatus(`Selected: ${path.split(/[/\\]/).pop()}`, false);
    }
  }

  function handleViewerClear() {
    switch (viewerTarget) {
      case "annotate": annotationRectInput = ""; break;
      case "signature": signatureStrokes = []; signatureRectInput = ""; break;
      case "security": signRectInput = ""; break;
    }
  }

  onMount(() => {
    const handleGlobalKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && viewerFilePath) viewerFilePath = "";
    };
    window.addEventListener("keydown", handleGlobalKey);
    
    let unlisten: (() => void) | undefined;
    
    // In Tauri v2, we use onDragDropEvent
    getCurrentWebviewWindow().onDragDropEvent((event) => {
      if (event.payload.type === "drop") handleDroppedFiles(event.payload.paths);
    }).then(fn => { unlisten = fn; });

    return () => {
      window.removeEventListener("keydown", handleGlobalKey);
      if (unlisten) unlisten();
    };
  });

  const tools: { id: ToolId; label: string; icon: string }[] = [
    { id: "merge", label: "Merge", icon: "M" },
    { id: "split", label: "Split", icon: "S" },
    { id: "extract", label: "Extract & AI", icon: "✨" },
    { id: "annotate", label: "Annotate", icon: "A" },
    { id: "signature", label: "Sign", icon: "I" },
    { id: "security", label: "Security", icon: "🔒" },
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

      {:else if activeTool === 'extract'}
        <div class="space-y-6 flex flex-col h-[calc(100vh-140px)]">
          <div class="shrink-0 space-y-4">
            <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
              <h3 class="text-[10px] font-bold text-blue-600 dark:text-blue-400 uppercase tracking-widest transition-colors">Batch AI Data Extraction</h3>
              <button onclick={selectBatchFiles} class="w-full py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded text-xs font-medium shadow-sm">Select Batch Files</button>
              
              {#if batchExtractFiles.length > 0}
                <div class="space-y-1.5">
                  <label class="text-[9px] font-bold text-slate-500 uppercase tracking-tighter">Extraction Prompt</label>
                  <input type="text" bind:value={batchExtractPrompt} class="w-full p-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" />
                </div>
                <button onclick={handleBatchExtract} disabled={isBatchExtracting} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-xs uppercase tracking-widest transition-colors shadow-md shadow-blue-500/20">
                  {isBatchExtracting ? 'Extracting...' : 'Run Batch Extract'}
                </button>
                {#if batchExtractResults.length > 0}
                  <button onclick={exportBatchToCsv} class="w-full py-1.5 border border-green-600 text-green-600 dark:text-green-400 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-green-50 dark:hover:bg-green-900/20 transition-all">Export CSV ({batchExtractResults.length})</button>
                {/if}
                <button onclick={() => { batchExtractFiles = []; batchExtractResults = []; }} class="w-full text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter">Clear</button>
              {/if}
            </div>

            <div class="pt-2 border-t border-slate-100 dark:border-slate-900">
              <button onclick={() => selectFile('extract')} class="w-full py-2 px-4 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:border-blue-500 dark:hover:border-blue-400 rounded-md transition-all text-sm font-medium truncate shadow-sm">
                {selectedExtractFile ? selectedExtractFile.split(/[/\\]/).pop() : 'Select PDF to Extract/Chat'}
              </button>

              <div class="mt-4 space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
                <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Extract Content</h3>
                <div class="flex gap-2">
                  <button onclick={handleExtractText} disabled={!selectedExtractFile} class="flex-1 py-2 border border-blue-600 text-blue-600 dark:text-blue-400 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all">Text to .txt</button>
                  <button onclick={() => ocrTrigger = Date.now()} disabled={!selectedExtractFile} class="flex-1 py-2 border border-blue-600 text-white bg-blue-600 rounded font-bold text-[10px] uppercase tracking-widest hover:bg-blue-700 transition-all shadow-md shadow-blue-500/20">OCR Page {viewerPageNumber}</button>
                </div>
              </div>
            </div>
          </div>

          <div class="flex-1 min-h-0 flex flex-col pt-4 border-t border-slate-100 dark:border-slate-900">
            <div class="flex items-center justify-between mb-3 shrink-0">
              <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">Chat with PDF</h3>
              <div class="flex gap-2">
                <select bind:value={aiProvider} onchange={() => { if(aiProvider === "ollama") checkOllama(); }} class="bg-transparent text-[9px] font-bold text-slate-400 outline-none cursor-pointer hover:text-blue-500 transition-colors">
                  <option value="ollama">Ollama (Stable)</option>
                  <option value="webllm">In-App (Experimental)</option>
                </select>
                {#if aiProvider === "webllm"}
                  <button onclick={resetEngine} class="text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter transition-colors">Reset</button>
                {/if}
              </div>
            </div>
            
            <div class="flex-1 overflow-y-auto mb-4 space-y-3 pr-2 scroll-smooth">
              {#if aiProvider === "ollama" && ollamaStatus !== "connected"}
                <div class="p-4 rounded-xl bg-slate-50 dark:bg-slate-900/60 border border-slate-200 dark:border-slate-800 space-y-3">
                  <div class="flex items-center gap-2">
                    <span class="w-2 h-2 rounded-full {ollamaStatus === 'checking' ? 'bg-amber-500 animate-pulse' : 'bg-red-500'}"></span>
                    <h4 class="text-[11px] font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider">Ollama Setup Required</h4>
                  </div>
                  <p class="text-[10px] text-slate-500 leading-relaxed">For a reliable experience with large documents, please ensure Ollama is installed and running.</p>
                  <div class="space-y-2">
                    <div class="flex items-start gap-2 text-[10px] text-slate-600 dark:text-slate-400">
                      <span class="font-bold text-blue-500">1.</span>
                      <span>Download from <a href="https://ollama.com" target="_blank" class="text-blue-500 hover:underline">ollama.com</a></span>
                    </div>
                    <div class="flex items-start gap-2 text-[10px] text-slate-600 dark:text-slate-400">
                      <span class="font-bold text-blue-500">2.</span>
                      <span>Run <code class="bg-slate-200 dark:bg-slate-800 px-1 rounded text-pink-500">ollama run llama3.2:1b</code> in your terminal.</span>
                    </div>
                  </div>
                  <button onclick={checkOllama} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-[10px] font-bold uppercase tracking-widest transition-colors shadow-sm">Check Connection</button>
                  <button onclick={() => aiProvider = "webllm"} class="w-full py-1 text-[9px] text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 font-bold uppercase tracking-tighter transition-colors">Use In-App AI instead (Zero Install)</button>
                </div>
              {:else if chatHistory.length === 0 && !isModelLoading}
                <div class="text-xs text-slate-400 italic text-center mt-4">
                  {aiProvider === "ollama" ? "Connected to Ollama. Ask a question!" : "Ask a question to start the conversation..."}
                </div>
              {/if}
              
              {#if aiProvider === "webllm" && isModelLoading}
                <div class="space-y-2 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-100 dark:border-blue-900/40">
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 rounded-full bg-blue-500 animate-pulse"></div>
                    <span class="text-[10px] font-bold text-blue-600 dark:text-blue-400 uppercase tracking-widest">Loading In-App AI...</span>
                  </div>
                  <div class="text-[9px] text-slate-500 dark:text-slate-400 font-mono leading-tight whitespace-pre-wrap">{modelLoadingProgress}</div>
                </div>
              {/if}

              {#each chatHistory as msg}
                <div class="text-xs p-3 rounded-lg {msg.role === 'user' ? 'bg-blue-100 dark:bg-blue-900/40 text-blue-900 dark:text-blue-100 ml-4 rounded-tr-none' : 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200 mr-4 rounded-tl-none'} shadow-sm">
                  {msg.content}
                </div>
              {/each}
              {#if isChatting}
                <div class="text-xs p-3 rounded-lg bg-slate-100 dark:bg-slate-800 text-slate-500 w-16 shadow-sm">...</div>
              {/if}
            </div>

            <div class="shrink-0 flex gap-2">
              <input type="text" bind:value={chatInput} onkeydown={(e) => e.key === 'Enter' && handleAskPdf()} placeholder="Ask about this document..." class="flex-1 p-2.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-sm text-slate-900 dark:text-white outline-none transition-colors focus:ring-2 focus:ring-blue-500 shadow-sm" disabled={!selectedExtractFile || isChatting} />
              <button onclick={handleAskPdf} disabled={!selectedExtractFile || isChatting || !chatInput.trim()} class="px-4 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-sm transition-colors shadow-md disabled:opacity-50">↑</button>
            </div>
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
            <div class="flex gap-2">
              <button onclick={handleSignatureVisual} disabled={!selectedSignatureFile || signatureStrokes.length === 0} class="flex-[2] py-3 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg font-bold text-xs uppercase tracking-widest transition-all shadow-xl hover:scale-[1.02]">Apply Signature</button>
              <button onclick={handleSecureShare} disabled={!selectedSignatureFile} class="flex-1 py-3 border border-blue-600 text-blue-600 dark:text-blue-400 rounded-lg font-bold text-[10px] uppercase tracking-widest hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all shadow-md">Share</button>
            </div>
          </div>
        </div>

      {:else if activeTool === 'security'}
        <div class="space-y-6">
          <button onclick={() => selectFile('security')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
            {selectedCryptoFile ? selectedCryptoFile.split(/[/\\]/).pop() : 'Select PDF to Redact/Sign'}
          </button>

          <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
            <h3 class="text-[10px] font-bold text-red-600 dark:text-red-400 uppercase tracking-widest transition-colors">AI Auto-Redact PII</h3>
            <p class="text-[10px] text-slate-500 leading-tight">Finds and blackouts Social Security Numbers, emails, phone numbers, and names.</p>
            
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
              <button onclick={handleRedactAll} class="w-full py-2 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded font-bold text-[10px] uppercase tracking-widest shadow-md">Apply Redactions</button>
              <button onclick={() => piiItems = []} class="w-full text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter">Clear Results</button>
            {:else}
              <button onclick={handleFindPii} disabled={!selectedCryptoFile || isFindingPii} class="w-full py-2 border border-red-600 text-red-600 dark:text-red-400 rounded font-bold text-xs uppercase tracking-widest hover:bg-red-50 dark:hover:bg-red-900/20 transition-all shadow-sm shadow-red-500/10">
                {isFindingPii ? 'Finding PII...' : 'Find & Redact PII'}
              </button>
            {/if}
          </div>

          <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900 transition-colors">
            <h3 class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest transition-colors">Digital Signature</h3>
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
        </div>

      {:else if activeTool === 'organize'}
        <div class="space-y-10">
          <div class="space-y-4 p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 transition-colors">
            <h3 class="text-[10px] font-bold text-blue-600 dark:text-blue-400 uppercase tracking-widest transition-colors">AI Auto-Rename (Batch)</h3>
            <button onclick={selectAutoRenameFiles} class="w-full py-2 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-xs font-medium shadow-sm">Select Files to Rename</button>
            
            {#if autoRenameFiles.length > 0}
              <div class="max-h-40 overflow-y-auto space-y-2 border-t border-slate-200 dark:border-slate-800 pt-3">
                {#each autoRenameFiles as file}
                  <div class="flex flex-col gap-0.5 p-2 rounded bg-white dark:bg-slate-800 border border-slate-100 dark:border-slate-700 shadow-sm">
                    <span class="text-[9px] text-slate-400 truncate">{file.originalName}</span>
                    {#if file.status === 'done'}
                      <span class="text-[10px] text-green-500 font-bold truncate">→ {file.newName}</span>
                    {:else if file.status === 'processing'}
                      <span class="text-[9px] text-blue-500 animate-pulse">Renaming...</span>
                    {:else if file.status === 'error'}
                      <span class="text-[9px] text-red-500">Error renaming</span>
                    {/if}
                  </div>
                {/each}
              </div>
              <button onclick={handleAutoRename} disabled={isRenaming} class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-bold text-xs uppercase tracking-widest transition-colors shadow-md shadow-blue-500/20">Run AI Auto-Rename</button>
              <button onclick={() => autoRenameFiles = []} class="w-full text-[9px] font-bold text-slate-400 hover:text-red-500 uppercase tracking-tighter">Clear List</button>
            {/if}
          </div>

          <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-900">
            <button onclick={() => selectFile('rotate')} class="w-full py-2 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-md text-sm font-medium transition-colors shadow-sm">
              {selectedRotateFile ? selectedRotateFile.split(/[/\\]/).pop() : 'Select PDF to Rotate/Delete'}
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
          ocrTrigger={ocrTrigger}
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
