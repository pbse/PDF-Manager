import { browser } from "$app/environment";

import { invoke } from "@tauri-apps/api/core";

import { db, type DocumentRecord, type CollectionRecord, type ActionRecord, type WatcherRecord } from "./db";

export type HistoryItem = DocumentRecord;

export class HistoryState {
  recentFiles = $state<HistoryItem[]>([]);
  collections = $state<CollectionRecord[]>([]);
  actions = $state<ActionRecord[]>([]);
  watchers = $state<WatcherRecord[]>([]);
  searchResults = $state<{ file: string, matches: string[], path: string }[]>([]);
  isSearching = $state(false);

  constructor() {
    this.loadHistory();
    this.loadCollections();
    this.loadActions();
    this.loadWatchers();
  }

  async loadWatchers() {
    this.watchers = await db.watchers.orderBy('timestamp').reverse().toArray();
  }

  async createWatcher(query: string) {
    await db.watchers.add({ query, active: true, timestamp: Date.now() });
    await this.loadWatchers();
  }

  async toggleWatcher(id: number, active: boolean) {
    await db.watchers.update(id, { active });
    await this.loadWatchers();
  }

  async deleteWatcher(id: number) {
    await db.watchers.delete(id);
    await this.loadWatchers();
  }

  async loadHistory() {
    this.recentFiles = await db.documents.orderBy('timestamp').reverse().limit(20).toArray();
  }

  async loadCollections() {
    this.collections = await db.collections.orderBy('name').toArray();
  }

  async loadActions() {
    this.actions = await db.actions.orderBy('timestamp').reverse().limit(50).toArray();
  }

  async logAction(type: string, description: string, resultPath?: string) {
    await db.actions.add({ type, description, timestamp: Date.now(), resultPath });
    await this.loadActions();
  }

  async createCollection(name: string) {
    await db.collections.add({ name, timestamp: Date.now() });
    await this.loadCollections();
  }

  async deleteCollection(id: number) {
    await db.collections.delete(id);
    // Unset collectionId for documents in this collection
    await db.documents.where('collectionId').equals(id).modify({ collectionId: undefined });
    await this.loadCollections();
    await this.loadHistory();
  }

  async addToCollection(docPath: string, collectionId: number) {
    const item = await db.documents.where('path').equals(docPath).first();
    if (item?.id) {
      await db.documents.update(item.id, { collectionId });
      await this.loadHistory();
    }
  }

  async indexEntities(path: string, insights: { dates: string[], amounts: string[], orgs: string[] }) {
    const processList = async (list: string[], type: 'org' | 'person' | 'date' | 'location') => {
      if (!Array.isArray(list)) return;
      for (const name of list) {
        if (!name) continue;
        const existing = await db.entities.where('name').equals(name).first();
        if (existing) {
          if (!existing.docPaths.includes(path)) {
            await db.entities.update(existing.id!, { docPaths: [...existing.docPaths, path] });
          }
        } else {
          await db.entities.add({ name, type, docPaths: [path] });
        }
      }
    };

    await processList(insights?.orgs || [], 'org');
    await processList(insights?.dates || [], 'date');
    // person and location could be added to insights extraction in a future refinement
  }

  async getRelatedDocuments(path: string): Promise<HistoryItem[]> {
    const doc = await db.documents.where('path').equals(path).first();
    if (!doc) return [];
    
    // Find documents that share the same entities
    const entities = await db.entities.where('docPaths').equals(path).toArray();
    const relatedPaths = new Set<string>();
    for (const ent of entities) {
      ent.docPaths.forEach(p => { if (p !== path) relatedPaths.add(p); });
    }

    return await db.documents.where('path').anyOf([...relatedPaths]).toArray();
  }

  async searchLibrary(query: string) {
    if (!query.trim()) { this.searchResults = []; return; }
    this.isSearching = true;
    
    // Fast in-memory/indexedDB search
    const results = await db.documents
      .filter(doc => 
        doc.name.toLowerCase().includes(query.toLowerCase()) || 
        (doc.fullText?.toLowerCase().includes(query.toLowerCase()) || false)
      )
      .toArray();

    this.searchResults = results.map(doc => {
      const matches: string[] = [];
      if (doc.fullText) {
        const lines = doc.fullText.split("\n");
        const match = lines.find(l => l.toLowerCase().includes(query.toLowerCase()));
        if (match) matches.push(match.trim());
      }
      return { file: doc.name, path: doc.path, matches };
    });
    
    this.isSearching = false;
  }

  async addFile(path: string) {
    const name = path.split(/[/\\]/).pop() || path;
    const existing = await db.documents.where('path').equals(path).first();
    
    // Get full text, thumbnail, and hash
    let fullText = existing?.fullText;
    let thumbnail = existing?.thumbnail;
    let hash = existing?.hash;
    
    if (!fullText || !thumbnail || !hash) {
      try {
        if (!fullText) fullText = await invoke<string>("pdf_to_text_string", { path });
        if (!hash) hash = await invoke<string>("get_file_hash", { path });
        
        // Semantic Chunking
        // Future refinement: use a more sophisticated NLP chunker
        const chunks = fullText.split(/\n\s*\n/).filter(c => c.trim().length > 50);
        // Indexing chunks would go into a new table if needed, for now we keep fullText
      } catch (e) { console.error("Data extraction failed", e); }
    }

    const record: DocumentRecord = {
      path,
      name,
      timestamp: Date.now(),
      tags: existing?.tags || [],
      fullText,
      thumbnail,
      hash
    };

    if (existing?.id) record.id = existing.id;
    
    await db.documents.put(record);
    await this.loadHistory();
  }

  async findDuplicates(): Promise<HistoryItem[][]> {
    const docs = await db.documents.toArray();
    const groups: Record<string, HistoryItem[]> = {};
    for (const doc of docs) {
      if (doc.hash) {
        if (!groups[doc.hash]) groups[doc.hash] = [];
        groups[doc.hash].push(doc);
      }
    }
    return Object.values(groups).filter(g => g.length > 1);
  }

  async updateTags(path: string, tags: string[]) {
    const item = await db.documents.where('path').equals(path).first();
    if (item?.id) {
      await db.documents.update(item.id, { tags });
      await this.loadHistory();
    }
  }

  async clear() {
    await db.documents.clear();
    await db.chats.clear();
    this.recentFiles = [];
  }

  async exportLibrary(): Promise<string> {
    const docs = await db.documents.toArray();
    const chats = await db.chats.toArray();
    return JSON.stringify({ docs, chats, version: 1 });
  }

  async importLibrary(jsonStr: string) {
    try {
      const data = JSON.parse(jsonStr);
      if (data.docs) {
        await db.documents.clear();
        await db.documents.bulkAdd(data.docs);
      }
      if (data.chats) {
        await db.chats.clear();
        await db.chats.bulkAdd(data.chats);
      }
      await this.loadHistory();
      return true;
    } catch (e) {
      console.error("Import failed", e);
      return false;
    }
  }
}

export const historyState = new HistoryState();
