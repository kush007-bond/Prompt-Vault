import { invoke, Channel } from '@tauri-apps/api/core';

// Types
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

export interface PromptVersion {
  id: string;
  prompt_id: string;
  title_snapshot: string;
  body_snapshot: string;
  changed_at: string;
  change_note: string | null;
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

export interface Attachment {
  /** File name, e.g. "report.txt" or "photo.png" */
  name: string;
  /** Plain text for text files; raw base64 (no data-URL prefix) for images */
  content: string;
  /** MIME type, e.g. "text/plain" or "image/jpeg" */
  mime_type: string;
}

export interface RunPromptRequest {
  provider: string;
  model: string;
  prompt: string;
  variables?: Record<string, string>;
  temperature?: number;
  max_tokens?: number;
  system_prompt?: string;
  attachments?: Attachment[];
}

export interface ProviderResponse {
  content: string;
  model: string;
  tokens_input: number | null;
  tokens_output: number | null;
  duration_ms: number | null;
}

export interface ModelInfo {
  id: string;
  name: string;
  context_window: number | null;
}

export interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface RunConversationRequest {
  provider: string;
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
}

export type StreamEvent =
  | { event: 'token'; data: string }
  | { event: 'done' }
  | { event: 'error'; data: string };

export interface ApiKeyStatus {
  provider: string;
  configured: boolean;
  has_key: boolean;
}

export interface HealthStatus {
  provider: string;
  available: boolean;
  error: string | null;
}

export interface InjectRequest {
  prompt_id: string;
  content: string;
  project_path: string;
  apply_globally?: boolean;
}

export interface InjectionResult {
  success: boolean;
  target_path: string;
  message: string;
}

export interface InjectionPreview {
  target_path: string;
  content_preview: string;
  will_overwrite: boolean;
}

export interface InjectionLog {
  id: string;
  prompt_id: string | null;
  cli_target: string;
  project_path: string;
  injected_content: string;
  injected_at: string;
}

export interface ExportData {
  version: string;
  exported_at: string;
  prompts: any[];
  collections: any[];
  tags: any[];
}

export interface ImportRequest {
  json_content: string;
}

export interface ImportResult {
  success: boolean;
  prompts_imported: number;
  collections_imported: number;
  tags_imported: number;
  message: string;
}

// Prompts API
export const promptsApi = {
  getAll: () => invoke<Prompt[]>('get_all_prompts'),
  get: (id: string) => invoke<Prompt>('get_prompt', { id }),
  create: (request: { title: string; body: string; model_target?: string; collection_id?: string }) =>
    invoke<Prompt>('create_prompt', { request }),
  update: (request: { id: string; title: string; body: string; model_target?: string; collection_id?: string; is_pinned?: boolean; is_archived?: boolean }) =>
    invoke<Prompt>('update_prompt', { request }),
  delete: (id: string) => invoke<void>('delete_prompt', { id }),
  search: (request: { query: string; collection_id?: string; is_pinned?: boolean; is_archived?: boolean; limit?: number; offset?: number }) =>
    invoke<Prompt[]>('search_prompts', { request }),
  getVersions: (promptId: string) => invoke<PromptVersion[]>('get_prompt_versions', { promptId }),
  restoreVersion: (versionId: string) => invoke<Prompt>('restore_prompt_version', { versionId }),
  duplicate: (id: string) => invoke<Prompt>('duplicate_prompt', { id }),
  togglePin: (id: string) => invoke<Prompt>('toggle_pin_prompt', { id }),
};

// Collections API
export const collectionsApi = {
  getAll: () => invoke<Collection[]>('get_all_collections'),
  create: (request: { name: string; parent_id?: string; color?: string; icon?: string }) =>
    invoke<Collection>('create_collection', { request }),
  update: (request: { id: string; name: string; parent_id?: string; color?: string; icon?: string }) =>
    invoke<Collection>('update_collection', { request }),
  delete: (id: string) => invoke<void>('delete_collection', { id }),
};

// Tags API
export const tagsApi = {
  getAll: () => invoke<Tag[]>('get_all_tags'),
  create: (request: { name: string; color?: string }) => invoke<Tag>('create_tag', { request }),
  update: (request: { id: string; name: string; color?: string }) => invoke<Tag>('update_tag', { request }),
  delete: (id: string) => invoke<void>('delete_tag', { id }),
  addToPrompt: (promptId: string, tagId: string) => invoke<void>('add_tag_to_prompt', { promptId, tagId }),
  removeFromPrompt: (promptId: string, tagId: string) => invoke<void>('remove_tag_from_prompt', { promptId, tagId }),
  getPromptTags: (promptId: string) => invoke<Tag[]>('get_prompt_tags', { promptId }),
};

// Settings API
export const settingsApi = {
  get: (key: string) => invoke<string | null>('get_setting', { key }),
  set: (key: string, value: string) => invoke<void>('set_setting', { request: { key, value } }),
  getAll: () => invoke<Setting[]>('get_all_settings'),
};

// AI API
export const aiApi = {
  runPrompt: (request: RunPromptRequest) => invoke<ProviderResponse>('run_prompt', { request }),
  streamPromptWithChannel: (request: RunPromptRequest, channel: Channel<StreamEvent>) =>
    invoke<void>('run_prompt_streaming', { channel, request }),
  runConversation: (request: RunConversationRequest) =>
    invoke<ProviderResponse>('run_conversation', { request }),
  listModels: (provider: string) => invoke<ModelInfo[]>('list_models', { provider }),
  healthCheck: (provider: string) => invoke<HealthStatus>('health_check', { provider }),
  storeApiKey: (provider: string, apiKey: string) => invoke<void>('store_api_key', { provider, apiKey }),
  getApiKeyStatus: (provider: string) => invoke<ApiKeyStatus>('get_api_key_status', { provider }),
};

// CLI API
export const cliApi = {
  injectToClaudeCode: (request: InjectRequest) => invoke<InjectionResult>('inject_to_claude_code', { request }),
  injectToCursor: (request: InjectRequest) => invoke<InjectionResult>('inject_to_cursor', { request }),
  injectToContinue: (request: InjectRequest) => invoke<InjectionResult>('inject_to_continue', { request }),
  injectToAider: (request: InjectRequest) => invoke<InjectionResult>('inject_to_aider', { request }),
  previewInjection: (target: string, projectPath: string, content: string, applyGlobally?: boolean) =>
    invoke<InjectionPreview>('preview_injection', { target, projectPath, content, applyGlobally }),
  getHistory: (limit?: number) => invoke<InjectionLog[]>('get_injection_history', { limit }),
};

// Export API
export const exportApi = {
  toJson: () => invoke<string>('export_to_json'),
  toMarkdown: (outputPath: string) => invoke<string>('export_to_markdown', { outputPath }),
  fromJson: (request: ImportRequest) => invoke<ImportResult>('import_from_json', { request }),
};