import * as webllm from "@mlc-ai/web-llm";
import { appState } from "./appState.svelte";
import { pdfState } from "./pdfState.svelte";
import { invoke } from "@tauri-apps/api/core";
import { db } from "./db";

export type ChatMessage = { role: "user" | "assistant" | "system"; content: string };

export class ChatState {
  history = $state<ChatMessage[]>([]);
  input = $state("");
  isChatting = $state(false);
  activeDocPath = $state<string | null>(null);

  async loadHistory(path: string) {
    this.activeDocPath = path;
    const records = await db.chats.where('docPath').equals(path).sortBy('timestamp');
    this.history = records.map(r => ({ role: r.role as any, content: r.content }));
  }

  async saveMessage(role: "user" | "assistant", content: string) {
    if (!this.activeDocPath) return;
    await db.chats.add({
      docPath: this.activeDocPath,
      role,
      content,
      timestamp: Date.now()
    });
  }
  aiProvider = $state<"ollama" | "webllm">("ollama");
  ollamaStatus = $state<"checking" | "connected" | "not_running">("checking");
  ollamaModel = $state("llama3.2:1b");
  
  engine = $state<webllm.MLCEngine | null>(null);
  modelLoadingProgress = $state("");
  isModelLoading = $state(false);
  readonly MODEL_ID = "Llama-3.2-1B-Instruct-q4f16_1-MLC";

  async clusterDocuments(docs: { path: string, name: string, summary: string }[]): Promise<{ name: string, docPaths: string[] }[]> {
    if (docs.length === 0) return [];
    try {
      const system = "You are a knowledge architect. Group these documents into 3-5 logical thematic clusters based on their summaries. Return JSON with a 'clusters' array, each having a 'name' and 'docPaths' array. ONLY return JSON.";
      const prompt = `DOCUMENTS:\n${docs.map(d => `- ${d.name}: ${d.summary} (Path: ${d.path})`).join("\n")}`;
      const result = await this.runAiTask(system, prompt, { json: true });
      const parsed = JSON.parse(result);
      return parsed.clusters || [];
    } catch (e) { return []; }
  }

  async getBibtex(path: string): Promise<string> {
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);
      const system = "You are a research assistant. Create a standard BibTeX entry for this document. Guess the BibTeX key. Return ONLY the BibTeX code block.";
      return await this.runAiTask(system, pdfText);
    } catch (e) { return "@article{key, title={Unknown}, author={Unknown}}"; }
  }

  async generateOutline(path: string): Promise<{ title: string, page: number }[]> {
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path });
      if (pdfText.length > 10000) pdfText = pdfText.substring(0, 10000);
      const system = "You are a structural editor. Analyze the document text and identify 5-8 main sections or chapters. Return JSON with an 'outline' array containing objects with 'title' and 'page' (estimate page based on text position). ONLY return JSON.";
      const result = await this.runAiTask(system, pdfText, { json: true });
      const parsed = JSON.parse(result);
      return (parsed.outline || []).map((it: any) => ({ title: it.title, page: parseInt(it.page) || 1, children: [] }));
    } catch (e) { return []; }
  }

  async suggestTags(path: string): Promise<string[]> {
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);
      const system = "You are a professional librarian. Suggest 3-5 concise semantic tags for this document text. Return JSON with a 'tags' array. ONLY return JSON.";
      const result = await this.runAiTask(system, pdfText, { json: true });
      const parsed = JSON.parse(result);
      return parsed.tags || [];
    } catch (e) { return []; }
  }

  async getDocumentInsights(path: string): Promise<{ dates: string[], amounts: string[], orgs: string[] }> {
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);
      const system = "Extract top 3 Dates, top 3 Organizations, and top 3 Currency Amounts from the text. Return JSON with 'dates', 'orgs', 'amounts' arrays. ONLY return JSON.";
      const result = await this.runAiTask(system, pdfText, { json: true });
      return JSON.parse(result);
    } catch (e) { return { dates: [], amounts: [], orgs: [] }; }
  }

  async nameDocument(path: string): Promise<string> {
    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path });
      if (pdfText.length > 5000) pdfText = pdfText.substring(0, 5000);
      const system = "You are a naming assistant. Suggest a 1-sentence summary for this document. Be extremely brief (max 15 words).";
      return await this.runAiTask(system, `TEXT:\n${pdfText}`);
    } catch (e) { return "Ready to help with your document."; }
  }

  async checkOllama() {
    this.ollamaStatus = "checking";
    try {
      const resp = await fetch("http://localhost:11434/api/tags");
      if (resp.ok) this.ollamaStatus = "connected";
      else this.ollamaStatus = "not_running";
    } catch {
      this.ollamaStatus = "not_running";
    }
  }

  async resetEngine() {
    if (this.engine) {
      await this.engine.unload();
      this.engine = null;
    }
    this.history = [];
    appState.showStatus("AI Engine reset. It will re-initialize on your next question.", false);
  }

  async runAiTask(system: string, prompt: string, options: { json?: boolean } = {}): Promise<string> {
    // Keep non-streaming version for internal tasks like naming/summarizing
    return this.runAiTaskInternal(system, prompt, options);
  }

  private async runAiTaskInternal(system: string, prompt: string, options: { json?: boolean } = {}): Promise<string> {
    if (this.aiProvider === "ollama") {
      const response = await fetch("http://localhost:11434/api/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model: this.ollamaModel,
          system,
          prompt,
          stream: false,
          format: options.json ? "json" : undefined,
          options: { num_ctx: 32768, temperature: 0.2 }
        })
      });
      if (!response.ok) throw new Error("Ollama Error");
      const data = await response.json();
      return data.response;
    } else {
      if (!this.engine) await this.initWebLlm();
      const messages: webllm.ChatCompletionMessageParam[] = [{ role: "system", content: system }, { role: "user", content: prompt }];
      const reply = await this.engine!.chat.completions.create({ messages, response_format: options.json ? { type: "json_object" } : undefined });
      return reply.choices[0].message.content || "";
    }
  }

  private async initWebLlm() {
    this.isModelLoading = true;
    this.engine = await webllm.CreateMLCEngine(this.MODEL_ID, {
      initProgressCallback: (report) => { this.modelLoadingProgress = report.text; }
    });
    this.isModelLoading = false;
  }

  async handleAskPdf() {
    if (!pdfState.selectedExtractFile || !this.input.trim()) return;

    const question = this.input;
    this.history = [...this.history, { role: "user", content: question }];
    await this.saveMessage("user", question);
    this.input = "";
    this.isChatting = true;

    // Add empty assistant message for streaming
    const assistantMsgIndex = this.history.length;
    this.history = [...this.history, { role: "assistant", content: "" }];

    try {
      let pdfText = await invoke<string>("pdf_to_text_string", { path: pdfState.selectedExtractFile });
      // Add page markers if possible, or just instruct AI to cite line snippets.
      const system = "You are a precise document assistant. Use ONLY the context below. IMPORTANT: When you state a fact, always provide a citation in brackets at the end of the sentence, e.g. [p. 3] or ['exact quote']. If unknown, say so.";
      const prompt = `### CONTEXT:\n${pdfText}\n\n### QUESTION:\n${question}`;

      if (this.aiProvider === "ollama") {
        const response = await fetch("http://localhost:11434/api/generate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model: this.ollamaModel, system, prompt, stream: true })
        });

        if (!response.ok) throw new Error("Ollama connection failed.");
        const reader = response.body?.getReader();
        if (!reader) throw new Error("No reader");

        let accumulated = "";
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          const chunk = new TextDecoder().decode(value);
          const lines = chunk.split("\n").filter(l => l.trim());
          for (const line of lines) {
            const data = JSON.parse(line);
            accumulated += data.response;
            this.history[assistantMsgIndex].content = accumulated;
          }
        }
        await this.saveMessage("assistant", accumulated);
      } else {
        if (!this.engine) await this.initWebLlm();
        const messages: webllm.ChatCompletionMessageParam[] = [
          { role: "system", content: system },
          { role: "user", content: prompt }
        ];
        const chunks = await this.engine!.chat.completions.create({ messages, stream: true });
        let accumulated = "";
        for await (const chunk of chunks) {
          accumulated += chunk.choices[0]?.delta?.content || "";
          this.history[assistantMsgIndex].content = accumulated;
        }
        await this.saveMessage("assistant", accumulated);
      }
    } catch (e: any) {
      this.history[assistantMsgIndex].content = `Error: ${e.message}`;
    } finally {
      this.isChatting = false;
    }
  }

  async handleAskLibrary() {
    if (!this.input.trim()) return;
    const question = this.input;
    this.history = [...this.history, { role: "user", content: question }];
    this.input = "";
    this.isChatting = true;
    const assistantMsgIndex = this.history.length;
    this.history = [...this.history, { role: "assistant", content: "" }];

    try {
      const docs = await db.documents.toArray();
      let combinedContext = "";
      for (const doc of docs) {
        if (doc.fullText) {
           combinedContext += `--- DOCUMENT: ${doc.name} ---\n${doc.fullText.substring(0, 5000)}\n\n`;
        }
      }
      
      const system = "You are a library intelligence assistant. Synthesize an answer based on ALL documents provided in the context. Cite document names.";
      const prompt = `### LIBRARY CONTEXT:\n${combinedContext}\n\n### QUESTION:\n${question}`;

      if (this.aiProvider === "ollama") {
        const resp = await fetch("http://localhost:11434/api/generate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model: this.ollamaModel, system, prompt, stream: true })
        });
        const reader = resp.body?.getReader();
        let accumulated = "";
        while (true) {
          const { done, value } = await reader!.read();
          if (done) break;
          const chunk = new TextDecoder().decode(value);
          const lines = chunk.split("\n").filter(l => l.trim());
          for (const line of lines) {
            const data = JSON.parse(line);
            accumulated += data.response;
            this.history[assistantMsgIndex].content = accumulated;
          }
        }
      } else {
        this.history[assistantMsgIndex].content = "Library chat currently optimized for Ollama.";
      }
    } catch (e: any) { this.history[assistantMsgIndex].content = `Error: ${e.message}`; }
    finally { this.isChatting = false; }
  }
}

export const chatState = new ChatState();
