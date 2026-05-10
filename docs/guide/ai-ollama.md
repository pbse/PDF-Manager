# AI & Ollama Integration

Pinnacle leverages local Artificial Intelligence to provide deep document insights without compromising your privacy.

## Why Local AI?

Most document tools send your PDFs to cloud-based LLMs (like GPT-4). Pinnacle takes a different path:
- **Private by Default:** Your data never leaves your machine.
- **Cost Effective:** No subscription fees or API costs.
- **Offline Capable:** Summarize and chat with your docs even without internet.

## Setting Up Ollama

Pinnacle uses [Ollama](https://ollama.com/) as the primary backend for local intelligence.

1. **Download & Install:** Visit [ollama.com](https://ollama.com/) to download and install Ollama for your OS.
2. **Pull the Model:** Open your terminal and pull the recommended model for Pinnacle:
   ```bash
   ollama run llama3
   ```
   *Note: You can also use other models like `mistral` or `phi3` depending on your machine's resources.*
3. **Pinnacle Connection:** Once Ollama is running, Pinnacle will automatically detect it on `localhost:11434`.

## Features Powered by AI

- **Insight Pane:** Automatically generates summaries, extracts key dates, and identifies organizations from your PDFs.
- **Contextual Chat:** Ask questions about your currently open document.
- **Document Naming:** Uses AI to suggest descriptive titles based on content analysis.
- **Knowledge Graph:** Powers the entity extraction engine that builds relationships between your documents.
