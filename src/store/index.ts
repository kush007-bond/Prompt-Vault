import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Prompt {
  id: string;
  title: string;
  body: string;
  model_target: string | null;
  collection_id: string | null;
  is_pinned: boolean;
  is_archived: boolean;
  use_count: number;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Collection {
  id: string;
  name: string;
  parent_id: string | null;
  color: string | null;
  icon: string | null;
  is_smart: boolean;
  smart_filter: string | null;
  sort_order: number;
  created_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
}

export interface Setting {
  key: string;
  value: string;
}

interface AppState {
  // Data
  prompts: Prompt[];
  collections: Collection[];
  tags: Tag[];
  settings: Record<string, string>;
  
  // UI State
  selectedPromptId: string | null;
  selectedCollectionId: string | null;
  searchQuery: string;
  isLoading: boolean;
  error: string | null;
  theme: 'light' | 'dark' | 'system';
  
  // Actions
  loadPrompts: () => Promise<void>;
  loadCollections: () => Promise<void>;
  loadTags: () => Promise<void>;
  loadSettings: () => Promise<void>;
  
  createPrompt: (title: string, body: string, collectionId?: string) => Promise<Prompt>;
  updatePrompt: (id: string, title: string, body: string, collectionId?: string) => Promise<void>;
  deletePrompt: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  duplicatePrompt: (id: string) => Promise<Prompt>;
  
  createCollection: (name: string, color?: string) => Promise<Collection>;
  updateCollection: (id: string, name: string, color?: string) => Promise<void>;
  deleteCollection: (id: string) => Promise<void>;
  
  createTag: (name: string, color?: string) => Promise<Tag>;
  deleteTag: (id: string) => Promise<void>;
  
  setSetting: (key: string, value: string) => Promise<void>;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
  
  setSelectedPrompt: (id: string | null) => void;
  setSelectedCollection: (id: string | null) => void;
  setSearchQuery: (query: string) => void;
  setError: (error: string | null) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  prompts: [],
  collections: [],
  tags: [],
  settings: {},
  selectedPromptId: null,
  selectedCollectionId: null,
  searchQuery: '',
  isLoading: false,
  error: null,
  theme: 'system',
  
  loadPrompts: async () => {
    set({ isLoading: true });
    try {
      const prompts = await invoke<Prompt[]>('get_all_prompts');
      set({ prompts, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },
  
  loadCollections: async () => {
    try {
      const collections = await invoke<Collection[]>('get_all_collections');
      set({ collections });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  
  loadTags: async () => {
    try {
      const tags = await invoke<Tag[]>('get_all_tags');
      set({ tags });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  
  loadSettings: async () => {
    try {
      const settings = await invoke<Setting[]>('get_all_settings');
      const settingsMap: Record<string, string> = {};
      settings.forEach(s => { settingsMap[s.key] = s.value; });
      set({ settings: settingsMap });
      // Apply the loaded theme to DOM — must go through setTheme so classList is updated
      get().setTheme((settingsMap.theme as 'light' | 'dark' | 'system') || 'system');
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createPrompt: async (title, body, collectionId) => {
    try {
      const prompt = await invoke<Prompt>('create_prompt', {
        request: { title, body, model_target: null, collection_id: collectionId || null }
      });
      set(state => ({ prompts: [prompt, ...state.prompts] }));
      return prompt;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  updatePrompt: async (id, title, body, collectionId) => {
    const updated = await invoke<Prompt>('update_prompt', {
      request: { id, title, body, model_target: null, collection_id: collectionId || null, is_pinned: null, is_archived: null }
    });
    set(state => ({ prompts: state.prompts.map(p => p.id === id ? updated : p) }));
  },
  
  deletePrompt: async (id) => {
    await invoke('delete_prompt', { id });
    set(state => ({ prompts: state.prompts.filter(p => p.id !== id), selectedPromptId: null }));
  },
  
  togglePin: async (id) => {
    const updated = await invoke<Prompt>('toggle_pin_prompt', { id });
    set(state => ({ prompts: state.prompts.map(p => p.id === id ? updated : p) }));
  },
  
  duplicatePrompt: async (id) => {
    const prompt = await invoke<Prompt>('duplicate_prompt', { id });
    set(state => ({ prompts: [prompt, ...state.prompts] }));
    return prompt;
  },
  
  createCollection: async (name, color) => {
    try {
      const collection = await invoke<Collection>('create_collection', {
        request: { name, parent_id: null, color: color || null, icon: null }
      });
      set(state => ({ collections: [...state.collections, collection] }));
      return collection;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
  
  updateCollection: async (id, name, color) => {
    await invoke('update_collection', {
      request: { id, name, parent_id: null, color: color || null, icon: null }
    });
    await get().loadCollections();
  },
  
  deleteCollection: async (id) => {
    await invoke('delete_collection', { id });
    set(state => ({ collections: state.collections.filter(c => c.id !== id) }));
  },
  
  createTag: async (name, color) => {
    try {
      const tag = await invoke<Tag>('create_tag', {
        request: { name, color: color || null }
      });
      set(state => ({ tags: [...state.tags, tag] }));
      return tag;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
  
  deleteTag: async (id) => {
    await invoke('delete_tag', { id });
    set(state => ({ tags: state.tags.filter(t => t.id !== id) }));
  },
  
  setSetting: async (key, value) => {
    await invoke('set_setting', { request: { key, value } });
    set(state => ({ settings: { ...state.settings, [key]: value } }));
  },
  
  setTheme: (theme) => {
    set({ theme });
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else if (theme === 'light') {
      document.documentElement.classList.remove('dark');
    } else {
      // System
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }
  },
  
  setSelectedPrompt: (id) => set({ selectedPromptId: id }),
  setSelectedCollection: (id) => set({ selectedCollectionId: id }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setError: (error) => set({ error }),
}));