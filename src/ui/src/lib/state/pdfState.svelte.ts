import { invoke } from "@tauri-apps/api/core";
import { appState } from "./appState.svelte";
import { historyState } from "./historyState.svelte";
import { db, type BookmarkRecord, type VersionRecord, type NoteRecord, type DocumentRecord } from "./db";

export type ToolId = "merge" | "split" | "extract" | "annotate" | "signature" | "security" | "organize" | "compare" | "library" | "forms" | "versions" | "watermark" | "notepad" | "peek" | "settings" | "insights" | "compress";
export type SelectionTarget = "parse" | "split" | "rotate" | "delete" | "annotate" | "signature" | "security" | "extract" | "crypto" | "organize";

const state = $state({
  activeTool: "insights" as ToolId,
  highlightedSnippet: null as string | null,
  bookmarks: [] as BookmarkRecord[],
  versions: [] as VersionRecord[],
  notes: [] as NoteRecord[],
  openTabs: [] as string[],
  selectedParseFile: null as string | null,
  selectedSplitFile: null as string | null,
  selectedRotateFile: null as string | null,
  selectedDeleteFile: null as string | null,
  selectedAnnotateFile: null as string | null,
  selectedSignatureFile: null as string | null,
  selectedCryptoFile: null as string | null,
  selectedExtractFile: null as string | null,
  selectedMergeFiles: [] as string[],
  viewerFilePath: "",
  viewerPageNumber: 1,
  viewerMode: "view" as "rect" | "points" | "view",
  viewerTarget: null as SelectionTarget | null,
  ocrTrigger: 0,
  splitPagesInput: "",
  rotatePagesInput: "",
  deletePagesInput: "",
  annotationRectInput: "",
  annotationStrokes: [] as [number, number][][],
  annotationType: "highlight" as "highlight" | "underline" | "strikeout" | "note" | "square" | "circle" | "ink",
  annotationText: "",
  annotationColor: "#facc15",
  signatureRectInput: "",
  signatureColor: "#0f172a",
  signatureWidth: 2 as number | null,
  signatureStrokes: [] as [number, number][][],
  signCertPath: "",
  signCertPassword: "",
  rememberPassword: false,
  signRectInput: "",
  signReason: "",
  signLocation: "",
  signContact: "",
  history: [] as any[],
  redoStack: [] as any[],
  comparisonFile2: null as string | null,

  switchTool(id: ToolId) {
    if (id === state.activeTool) return;
    state.activeTool = id;
    
    if (state.viewerFilePath) {
      if (id === 'split') state.selectedSplitFile = state.viewerFilePath;
      if (id === 'annotate') {
        state.selectedAnnotateFile = state.viewerFilePath;
        state.viewerMode = 'rect';
        state.viewerTarget = 'annotate';
      }
      if (id === 'signature') {
        state.selectedSignatureFile = state.viewerFilePath;
        state.viewerMode = 'points';
        state.viewerTarget = 'signature';
      }
      if (id === 'security') {
        state.selectedCryptoFile = state.viewerFilePath;
        state.viewerMode = 'rect';
        state.viewerTarget = 'security';
      }
      if (id === 'extract') state.selectedExtractFile = state.viewerFilePath;
      if (id === 'compress') state.selectedCryptoFile = state.viewerFilePath;
      if (id === 'organize') {
        state.selectedRotateFile = state.viewerFilePath;
        state.selectedDeleteFile = state.viewerFilePath;
      }
    } else {
      if (id === 'split' && state.selectedSplitFile) state.viewerFilePath = state.selectedSplitFile;
      if (id === 'annotate' && state.selectedAnnotateFile) {
        state.viewerFilePath = state.selectedAnnotateFile;
        state.viewerMode = 'rect';
        state.viewerTarget = 'annotate';
      }
      if (id === 'signature' && state.selectedSignatureFile) {
        state.viewerFilePath = state.selectedSignatureFile;
        state.viewerMode = 'points';
        state.viewerTarget = 'signature';
      }
      if (id === 'security' && state.selectedCryptoFile) {
        state.viewerFilePath = state.selectedCryptoFile;
        state.viewerMode = 'rect';
        state.viewerTarget = 'security';
      }
      if (id === 'extract' && state.selectedExtractFile) state.viewerFilePath = state.selectedExtractFile;
      if (id === 'organize' && state.selectedRotateFile) state.viewerFilePath = state.selectedRotateFile;
    }

    state.signatureStrokes = [];
    state.annotationStrokes = [];
    state.annotationRectInput = "";
    state.signatureRectInput = "";
    state.signRectInput = "";
    
    if (id === 'annotate' && state.annotationType === 'ink') {
      state.viewerMode = 'points';
    } else if (!['annotate', 'signature', 'security'].includes(id)) {
      state.viewerMode = "view";
    }
    
    appState.showStatus(`Switched to ${id.toUpperCase()}`, false);
  },

  openTab(path: string) {
    if (!path) return;
    if (!state.openTabs.includes(path)) {
      state.openTabs = [...state.openTabs, path];
    }
    state.viewerFilePath = path;
    historyState.addFile(path);
  },

  closeTab(path: string) {
    state.openTabs = state.openTabs.filter(t => t !== path);
    if (state.viewerFilePath === path) {
      state.viewerFilePath = state.openTabs[state.openTabs.length - 1] || "";
    }
  },

  async saveReadingProgress(path: string, page: number, total: number) {
    if (!path) return;
    const item = await db.documents.where('path').equals(path).first();
    if (item?.id) {
      await db.documents.update(item.id, { lastPage: page, totalPages: total });
    }
  },

  async getReadingProgress(path: string): Promise<number> {
    if (!path) return 1;
    const item = await db.documents.where('path').equals(path).first();
    return item?.lastPage || 1;
  },

  async addBookmark(path: string, page: number, label: string) {
    if (!path) return;
    await db.bookmarks.add({
      docPath: path,
      pageNumber: page,
      label,
      timestamp: Date.now()
    });
    await state.loadBookmarks(path);
  },

  async deleteBookmark(id: number) {
    await db.bookmarks.delete(id);
    if (state.viewerFilePath) await state.loadBookmarks(state.viewerFilePath);
  },

  async loadNotes() {
    state.notes = await db.notes.orderBy('timestamp').reverse().toArray();
  },

  async addNote(content: string, citation?: { docPath: string, pageNumber: number, text: string }) {
    await db.notes.add({
      content,
      timestamp: Date.now(),
      tags: [],
      citations: citation ? [citation] : []
    });
    await state.loadNotes();
    appState.showStatus("Note saved to Research Notepad.", false);
  },

  async deleteNote(id: number) {
    await db.notes.delete(id);
    await state.loadNotes();
  },

  async loadBookmarks(path: string) {
    state.bookmarks = await db.bookmarks.where('docPath').equals(path).sortBy('timestamp');
  },

  async loadVersions(path: string) {
    state.versions = await db.versions.where('docPath').equals(path).sortBy('timestamp');
  },

  async saveSnapshot(path: string, label: string) {
    const bytes = await invoke<Uint8Array>("read_file_bytes", { path });
    await db.versions.add({
      docPath: path,
      label,
      timestamp: Date.now(),
      data: bytes
    });
    await state.loadVersions(path);
  },

  async restoreSnapshot(version: VersionRecord, outputPath: string) {
    await invoke("write_file_bytes", { path: outputPath, bytes: version.data });
    appState.showStatus(`Restored to: ${version.label}`, false, outputPath);
  },

  async saveSession(name: string) {
    const session = {
      name,
      tabs: state.openTabs,
      activeTab: state.viewerFilePath,
      timestamp: Date.now()
    };
    localStorage.setItem(`pdf_session_${name}`, JSON.stringify(session));
    appState.showStatus(`Session "${name}" saved.`, false);
  },

  async loadSession(name: string) {
    const saved = localStorage.getItem(`pdf_session_${name}`);
    if (saved) {
      const session = JSON.parse(saved);
      state.openTabs = session.tabs;
      state.viewerFilePath = session.activeTab;
      appState.showStatus(`Restored session: ${name}`, false);
    }
  },
  
  pushHistory(hState: any) {
    state.history.push(JSON.parse(JSON.stringify(hState)));
    state.redoStack = []; 
  },

  undo() {
    if (state.history.length > 0) {
      const lastState = state.history.pop();
      state.redoStack.push(JSON.parse(JSON.stringify({
        signatureStrokes: state.signatureStrokes,
        annotationStrokes: state.annotationStrokes,
        annotationRectInput: state.annotationRectInput,
        signatureRectInput: state.signatureRectInput
      })));
      state.signatureStrokes = lastState.signatureStrokes;
      state.annotationStrokes = lastState.annotationStrokes;
      state.annotationRectInput = lastState.annotationRectInput;
      state.signatureRectInput = lastState.signatureRectInput;
    }
  },

  redo() {
    if (state.redoStack.length > 0) {
      const nextState = state.redoStack.pop();
      state.history.push(JSON.parse(JSON.stringify({
        signatureStrokes: state.signatureStrokes,
        annotationStrokes: state.annotationStrokes,
        annotationRectInput: state.annotationRectInput,
        signatureRectInput: state.signatureRectInput
      })));
      state.signatureStrokes = nextState.signatureStrokes;
      state.annotationStrokes = nextState.annotationStrokes;
      state.annotationRectInput = nextState.annotationRectInput;
      state.signatureRectInput = nextState.signatureRectInput;
    }
  },

  setFileForTarget(target: SelectionTarget, path: string) {
    if (!path) return;
    
    switch (target) {
      case "split": state.selectedSplitFile = path; break;
      case "rotate": state.selectedRotateFile = path; break;
      case "delete": state.selectedDeleteFile = path; break;
      case "organize": 
        state.selectedRotateFile = path;
        state.selectedDeleteFile = path;
        break;
      case "annotate": state.selectedAnnotateFile = path; break;
      case "signature": state.selectedSignatureFile = path; break;
      case "security": 
      case "crypto": state.selectedCryptoFile = path; break;
      case "extract": state.selectedExtractFile = path; break;
    }

    state.openTab(path);
  },

  async selectFile(target: SelectionTarget) {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      state.setFileForTarget(target, result[0]);
    }
  },

  async openNewDocument() {
    const result = await invoke<string[]>("open_file_dialog", { multiple: false });
    if (result && result.length > 0) {
      state.openTab(result[0]);
      state.activeTool = 'insights';
    }
  },

  handleDroppedFiles(paths: string[]) {
    const pdfs = paths.filter((p) => p.toLowerCase().endsWith(".pdf"));
    if (pdfs.length === 0) {
      appState.showStatus("Only PDF files are supported.", true);
      return;
    }
    if (pdfs.length > 1) {
      pdfs.forEach(p => historyState.addFile(p));
      state.selectedMergeFiles = [...new Set([...state.selectedMergeFiles, ...pdfs])];
      appState.showStatus(`Added ${pdfs.length} PDFs to Merge.`, false);
    } else {
      const path = pdfs[0];
      state.setFileForTarget('split', path);
      appState.showStatus(`Selected: ${path.split(/[/\\]/).pop()}`, false);
    }
  }
});

export const pdfState = state;

if (typeof window !== "undefined") {
  (window as any).__PINNACLE_PDF_STATE__ = pdfState;
}
