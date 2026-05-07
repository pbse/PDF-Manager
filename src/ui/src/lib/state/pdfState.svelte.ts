import { invoke } from "@tauri-apps/api/core";
import { appState } from "./appState.svelte";
import { historyState } from "./historyState.svelte";
import { db, type BookmarkRecord, type VersionRecord, type NoteRecord } from "./db";

export type ToolId = "merge" | "split" | "extract" | "annotate" | "signature" | "security" | "organize" | "compare" | "library" | "forms" | "versions" | "watermark" | "notepad" | "peek";
export type SelectionTarget = "parse" | "split" | "rotate" | "delete" | "annotate" | "signature" | "security" | "extract" | "crypto";

export class PdfState {
  activeTool = $state<ToolId>("merge");
  highlightedSnippet = $state<string | null>(null);
  
  // Bookmarks, Versions & Notes
  bookmarks = $state<BookmarkRecord[]>([]);
  versions = $state<VersionRecord[]>([]);
  notes = $state<NoteRecord[]>([]);

  async loadNotes() {
    this.notes = await db.notes.orderBy('timestamp').reverse().toArray();
  }

  async addNote(content: string, citation?: { docPath: string, pageNumber: number, text: string }) {
    await db.notes.add({
      content,
      timestamp: Date.now(),
      tags: [],
      citations: citation ? [citation] : []
    });
    await this.loadNotes();
    appState.showStatus("Note saved to Research Notepad.", false);
  }

  async deleteNote(id: number) {
    await db.notes.delete(id);
    await this.loadNotes();
  }

  // Tabs
  openTabs = $state<string[]>([]);
  
  async loadBookmarks(path: string) {
    this.bookmarks = await db.bookmarks.where('docPath').equals(path).sortBy('timestamp');
  }

  async loadVersions(path: string) {
    this.versions = await db.versions.where('docPath').equals(path).sortBy('timestamp');
  }

  async saveSnapshot(path: string, label: string) {
    const bytes = await invoke<number[]>("read_file_bytes", { path });
    await db.versions.add({
      docPath: path,
      label,
      timestamp: Date.now(),
      data: new Uint8Array(bytes)
    });
    await this.loadVersions(path);
  }

  async restoreSnapshot(version: VersionRecord, outputPath: string) {
    await invoke("write_file_bytes", { path: outputPath, bytes: Array.from(version.data) });
    appState.showStatus(`Restored to: ${version.label}`, false, outputPath);
  }

  // Workspace Sessions
  async saveSession(name: string) {
    const session = {
      name,
      tabs: this.openTabs,
      activeTab: this.viewerFilePath,
      timestamp: Date.now()
    };
    localStorage.setItem(`pdf_session_${name}`, JSON.stringify(session));
    appState.showStatus(`Session "${name}" saved.`, false);
  }

  async loadSession(name: string) {
    const saved = localStorage.getItem(`pdf_session_${name}`);
    if (saved) {
      const session = JSON.parse(saved);
      this.openTabs = session.tabs;
      this.viewerFilePath = session.activeTab;
      appState.showStatus(`Restored session: ${name}`, false);
    }
  }

  openTab(path: string) {
    if (!this.openTabs.includes(path)) {
      this.openTabs = [...this.openTabs, path];
    }
    this.viewerFilePath = path;
    historyState.addFile(path);
  }

  closeTab(path: string) {
    this.openTabs = this.openTabs.filter(t => t !== path);
    if (this.viewerFilePath === path) {
      this.viewerFilePath = this.openTabs[this.openTabs.length - 1] || "";
    }
  }

  async saveReadingProgress(path: string, page: number, total: number) {
    const item = await db.documents.where('path').equals(path).first();
    if (item?.id) {
      await db.documents.update(item.id, { lastPage: page, totalPages: total });
    }
  }

  async getReadingProgress(path: string): Promise<number> {
    const item = await db.documents.where('path').equals(path).first();
    return item?.lastPage || 1;
  }

  async addBookmark(path: string, page: number, label: string) {
    await db.bookmarks.add({
      docPath: path,
      pageNumber: page,
      label,
      timestamp: Date.now()
    });
    await this.loadBookmarks(path);
  }

  async deleteBookmark(id: number) {
    await db.bookmarks.delete(id);
    if (this.viewerFilePath) await this.loadBookmarks(this.viewerFilePath);
  }
  
  // File Selections
  selectedParseFile = $state<string | null>(null);
  selectedSplitFile = $state<string | null>(null);
  selectedRotateFile = $state<string | null>(null);
  selectedDeleteFile = $state<string | null>(null);
  selectedAnnotateFile = $state<string | null>(null);
  selectedSignatureFile = $state<string | null>(null);
  selectedCryptoFile = $state<string | null>(null);
  selectedExtractFile = $state<string | null>(null);
  selectedMergeFiles = $state<string[]>([]);

  // Viewer State
  viewerFilePath = $state("");
  viewerPageNumber = $state(1);
  viewerMode = $state<"rect" | "points" | "view">("view");
  viewerTarget = $state<SelectionTarget | null>(null);
  ocrTrigger = $state(0);

  // Tools specific inputs
  splitPagesInput = $state("");
  rotatePagesInput = $state("");
  deletePagesInput = $state("");
  annotationRectInput = $state("");
  annotationType = $state<"highlight" | "underline" | "strikeout" | "note">("highlight");
  annotationText = $state("");
  annotationColor = $state("#facc15");
  signatureRectInput = $state("");
  signatureColor = $state("#0f172a");
  signatureWidth = $state<number | null>(2);
  signatureStrokes = $state<[number, number][][]>([]);
  signCertPath = $state("");
  signCertPassword = $state("");
  rememberPassword = $state(false);
  signRectInput = $state("");
  signReason = $state("");
  signLocation = $state("");
  signContact = $state("");

  // History / Undo Stack
  history = $state<any[]>([]);
  redoStack = $state<any[]>([]);

  pushHistory(state: any) {
    this.history.push(JSON.parse(JSON.stringify(state)));
    this.redoStack = []; // Clear redo on new action
  }

  undo() {
    if (this.history.length > 0) {
      const lastState = this.history.pop();
      this.redoStack.push(JSON.parse(JSON.stringify(this.getCurrentUndoState())));
      this.applyUndoState(lastState);
    }
  }

  redo() {
    if (this.redoStack.length > 0) {
      const nextState = this.redoStack.pop();
      this.history.push(JSON.parse(JSON.stringify(this.getCurrentUndoState())));
      this.applyUndoState(nextState);
    }
  }

  private getCurrentUndoState() {
    return {
      signatureStrokes: this.signatureStrokes,
      annotationRectInput: this.annotationRectInput,
      signatureRectInput: this.signatureRectInput
    };
  }

  private applyUndoState(state: any) {
    this.signatureStrokes = state.signatureStrokes;
    this.annotationRectInput = state.annotationRectInput;
    this.signatureRectInput = state.signatureRectInput;
  }

  switchTool(id: ToolId) {
    if (id === this.activeTool) return;
    this.activeTool = id;
    
    // Auto-target current file if viewing one
    if (this.viewerFilePath) {
      if (id === 'split') this.selectedSplitFile = this.viewerFilePath;
      if (id === 'annotate') {
        this.selectedAnnotateFile = this.viewerFilePath;
        this.viewerMode = 'rect';
        this.viewerTarget = 'annotate';
      }
      if (id === 'signature') {
        this.selectedSignatureFile = this.viewerFilePath;
        this.viewerMode = 'points';
        this.viewerTarget = 'signature';
      }
      if (id === 'security') {
        this.selectedCryptoFile = this.viewerFilePath;
        this.viewerMode = 'rect';
        this.viewerTarget = 'security';
      }
      if (id === 'extract') this.selectedExtractFile = this.viewerFilePath;
      if (id === 'organize') {
        this.selectedRotateFile = this.viewerFilePath;
        this.selectedDeleteFile = this.viewerFilePath;
      }
    }

    // Reset interaction states
    this.signatureStrokes = [];
    this.annotationRectInput = "";
    this.signatureRectInput = "";
    this.signRectInput = "";
    
    // Only reset mode if not already set by tool logic above
    if (!['annotate', 'signature', 'security'].includes(id)) {
      this.viewerMode = "view";
    }
    
    appState.showStatus(`Switched to ${id.toUpperCase()}`, false);
  }

  async selectFile(target: SelectionTarget) {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      const path = result[0];
      this.setFileForTarget(target, path);
      this.openTab(path);
    }
  }

  setFileForTarget(target: SelectionTarget, path: string) {
    historyState.addFile(path);
    switch (target) {
      case "split": this.selectedSplitFile = path; break;
      case "rotate": this.selectedRotateFile = path; break;
      case "delete": this.selectedDeleteFile = path; break;
      case "annotate": this.selectedAnnotateFile = path; break;
      case "signature": this.selectedSignatureFile = path; break;
      case "security": this.selectedCryptoFile = path; break;
      case "extract": this.selectedExtractFile = path; break;
      case "crypto": this.selectedCryptoFile = path; break;
    }
  }

  handleDroppedFiles(paths: string[]) {
    const pdfs = paths.filter((p) => p.toLowerCase().endsWith(".pdf"));
    if (pdfs.length === 0) {
      appState.showStatus("Only PDF files are supported.", true);
      return;
    }
    if (pdfs.length > 1) {
      pdfs.forEach(p => historyState.addFile(p));
      this.selectedMergeFiles = [...new Set([...this.selectedMergeFiles, ...pdfs])];
      appState.showStatus(`Added ${pdfs.length} PDFs to Merge.`, false);
    } else {
      const path = pdfs[0];
      this.setFileForTarget("split", path);
      this.setFileForTarget("rotate", path);
      this.setFileForTarget("delete", path);
      this.setFileForTarget("annotate", path);
      this.setFileForTarget("signature", path);
      this.setFileForTarget("security", path);
      this.setFileForTarget("extract", path);
      this.viewerFilePath = path;
      appState.showStatus(`Selected: ${path.split(/[/\\]/).pop()}`, false);
    }
  }
}

export const pdfState = new PdfState();
