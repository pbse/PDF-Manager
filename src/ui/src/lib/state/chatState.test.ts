import { describe, it, expect, vi, beforeEach } from 'vitest';
import { chatState } from './chatState.svelte';
import { db } from './db';

describe('ChatState RAG', () => {
  beforeEach(async () => {
    await db.documents.clear();
    await db.chats.clear();
    chatState.history = [];
    chatState.input = '';
    
    // Create mock documents
    await db.documents.bulkAdd([
      { path: '/1.pdf', name: 'Apples', timestamp: 1, tags: [], fullText: 'This document is all about apples and how red they are.' },
      { path: '/2.pdf', name: 'Bananas', timestamp: 2, tags: [], fullText: 'Bananas are yellow and monkeys love them.' },
      { path: '/3.pdf', name: 'Oranges', timestamp: 3, tags: [], fullText: 'Oranges are citrus fruits. Very juicy.' },
      { path: '/4.pdf', name: 'Grapes', timestamp: 4, tags: [], fullText: 'Grapes are purple and make good wine.' },
      { path: '/5.pdf', name: 'Apple Tree', timestamp: 5, tags: [], fullText: 'An apple tree produces apples every year.' },
    ]);
  });

  it('should rank and prioritize relevant documents for the library chat', async () => {
    // Mock the fetch call for ollama
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      body: {
        getReader: () => {
          let done = false;
          return {
            read: () => {
              if (done) return Promise.resolve({ done: true });
              done = true;
              return Promise.resolve({
                done: false,
                value: new TextEncoder().encode(JSON.stringify({ response: 'Mocked AI Response' }))
              });
            }
          };
        }
      }
    } as any);

    chatState.input = 'Tell me about apples';
    await chatState.handleAskLibrary();
    
    expect(chatState.history.length).toBe(2);
    expect(chatState.history[1].role).toBe('assistant');
    expect(chatState.history[1].content).toBe('Mocked AI Response');
    
    // The fetch should have been called with a prompt containing only Apple and Apple Tree docs, not Bananas
    expect(global.fetch).toHaveBeenCalled();
    const fetchArgs = vi.mocked(global.fetch).mock.calls[0][1] as any;
    const body = JSON.parse(fetchArgs.body);
    
    // 'apples' keyword should rank /1.pdf and /5.pdf the highest
    expect(body.prompt).toContain('Apples');
    expect(body.prompt).toContain('Apple Tree');
    expect(body.prompt).not.toContain('Bananas'); // Should be excluded since we only take top 3 and banana has 0 score.
  });
});
