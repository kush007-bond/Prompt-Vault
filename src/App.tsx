import { useState, useEffect, useRef, useCallback } from 'react';
import { Channel } from '@tauri-apps/api/core';
import { useAppStore } from './store';
import { Sidebar } from './components/Sidebar';
import { SettingsModal } from './components/SettingsModal';
import { ConversationPanel } from './components/ConversationPanel';
import { ABTestPanel } from './components/ABTestPanel';
import { Button } from './components/ui/button';
import { Input } from './components/ui/input';
import { Textarea } from './components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './components/ui/select';
import { Plus, Pin, Copy, Files, Trash2, Play, Search, X, Loader2, Paperclip, AlertTriangle, FileText, Image, MessageSquare, SplitSquareHorizontal } from 'lucide-react';
import { cn, formatDate, truncate } from './lib/utils';
import { aiApi, exportApi, type Attachment, type StreamEvent } from './lib/tauri';
import { MarkdownRenderer } from './components/MarkdownRenderer';

const PROVIDERS = [
  { id: 'ollama', label: 'Ollama' },
  { id: 'openai', label: 'OpenAI' },
  { id: 'anthropic', label: 'Anthropic' },
  { id: 'gemini', label: 'Gemini' },
  { id: 'mistral', label: 'Mistral' },
];

const DEFAULT_MODELS: Record<string, string> = {
  ollama: 'llama3',
  openai: 'gpt-4o',
  anthropic: 'claude-sonnet-4-6',
  gemini: 'gemini-2.0-flash',
  mistral: 'mistral-large-latest',
};

export default function App() {
  const {
    prompts,
    collections,
    selectedCollectionId,
    selectedPromptId,
    searchQuery,
    isLoading,
    error,
    loadPrompts,
    loadCollections,
    loadTags,
    loadSettings,
    createPrompt,
    updatePrompt,
    deletePrompt,
    togglePin,
    duplicatePrompt,
    setSelectedPrompt,
    setError,
  } = useAppStore();

  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [editBody, setEditBody] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newBody, setNewBody] = useState('');

  // AI panel mode: run | chat | ab
  type AiMode = 'run' | 'chat' | 'ab';
  const [aiMode, setAiMode] = useState<AiMode>('run');

  // AI state
  const [aiResponse, setAiResponse] = useState('');
  const [isRunningAI, setIsRunningAI] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState('ollama');
  const [selectedModel, setSelectedModel] = useState('llama3');
  const [availableModels, setAvailableModels] = useState<{ id: string; name: string }[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [modelFetchError, setModelFetchError] = useState<string | null>(null);
  const [modelRetryTick, setModelRetryTick] = useState(0);

  // File attachments
  interface AttachedFile {
    id: string;
    name: string;
    content: string;    // plain text OR raw base64 (no data-URL prefix) for images
    mimeType: string;
    isImage: boolean;
    previewUrl?: string;
    sizeLabel: string;
  }
  const [attachedFiles, setAttachedFiles] = useState<AttachedFile[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // UI state
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [commandSearch, setCommandSearch] = useState('');

  // Layout resize state
  const [sidebarWidth, setSidebarWidth] = useState(256);
  const [promptListWidth, setPromptListWidth] = useState(288);
  const [aiPanelHeight, setAiPanelHeight] = useState(220);

  const sidebarWidthRef = useRef(sidebarWidth);
  const promptListWidthRef = useRef(promptListWidth);
  const aiPanelHeightRef = useRef(aiPanelHeight);
  sidebarWidthRef.current = sidebarWidth;
  promptListWidthRef.current = promptListWidth;
  aiPanelHeightRef.current = aiPanelHeight;

  const startResize = useCallback((
    getRef: () => number,
    setter: (n: number) => void,
    direction: 'h' | 'v',
    min: number,
    max: number,
    invert = false
  ) => (e: React.MouseEvent) => {
    e.preventDefault();
    const startPos = direction === 'h' ? e.clientX : e.clientY;
    const startSize = getRef();
    const onMove = (ev: MouseEvent) => {
      const delta = (direction === 'h' ? ev.clientX : ev.clientY) - startPos;
      setter(Math.max(min, Math.min(max, startSize + (invert ? -delta : delta))));
    };
    const onUp = () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      document.body.style.removeProperty('cursor');
      document.body.style.removeProperty('user-select');
    };
    document.body.style.cursor = direction === 'h' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, []);

  useEffect(() => {
    loadPrompts();
    loadCollections();
    loadTags();
    loadSettings();
  }, [loadPrompts, loadCollections, loadTags, loadSettings]);

  // Load models whenever provider changes or retry is triggered
  useEffect(() => {
    let cancelled = false;
    const fetchModels = async () => {
      setModelsLoading(true);
      setAvailableModels([]);
      setModelFetchError(null);
      try {
        const models = await aiApi.listModels(selectedProvider);
        if (!cancelled) {
          if (models.length > 0) {
            setAvailableModels(models);
            setSelectedModel(prev =>
              models.find(m => m.id === prev) ? prev : models[0].id
            );
          } else if (selectedProvider === 'ollama') {
            setModelFetchError('Ollama connected but no models installed. Run: ollama pull llama3');
          }
        }
      } catch (e) {
        if (!cancelled) {
          if (selectedProvider === 'ollama') {
            setModelFetchError(String(e).replace(/^Error: /, ''));
          }
          // For other providers without a list API, just keep the text input
        }
      }
      if (!cancelled) setModelsLoading(false);
    };
    fetchModels();
    return () => { cancelled = true; };
  }, [selectedProvider, modelRetryTick]);

  // Keyboard shortcut for command palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen(true);
      }
      if (e.key === 'Escape') {
        setIsCommandPaletteOpen(false);
        setIsCreating(false);
        setIsEditing(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const filteredPrompts = prompts.filter(p => {
    if (selectedCollectionId && p.collection_id !== selectedCollectionId) return false;
    if (searchQuery && !p.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !p.body.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const selectedPrompt = prompts.find(p => p.id === selectedPromptId);

  const handleCreate = async () => {
    if (newTitle.trim()) {
      const prompt = await createPrompt(newTitle.trim(), newBody.trim(), selectedCollectionId || undefined);
      setNewTitle('');
      setNewBody('');
      setIsCreating(false);
      setSelectedPrompt(prompt.id);
      setEditTitle(prompt.title);
      setEditBody(prompt.body);
    }
  };

  const handleSaveEdit = async () => {
    if (selectedPrompt) {
      await updatePrompt(selectedPrompt.id, editTitle, editBody, selectedPrompt.collection_id || undefined);
      setIsEditing(false);
    }
  };

  const handleRunAI = async () => {
    const promptText = isEditing ? editBody : selectedPrompt?.body;
    if (!promptText) return;
    setIsRunningAI(true);
    setAiResponse('');

    const attachments: Attachment[] = attachedFiles.map(f => ({
      name: f.name,
      content: f.content,
      mime_type: f.mimeType,
    }));

    const channel = new Channel<StreamEvent>();
    channel.onmessage = (event) => {
      if (event.event === 'token') {
        setAiResponse(prev => prev + event.data);
      } else if (event.event === 'done') {
        setIsRunningAI(false);
      } else if (event.event === 'error') {
        setAiResponse(prev => prev || ('Error: ' + event.data));
        setIsRunningAI(false);
      }
    };

    try {
      await aiApi.streamPromptWithChannel(
        {
          provider: selectedProvider,
          model: selectedModel,
          prompt: promptText,
          attachments: attachments.length ? attachments : undefined,
        },
        channel,
      );
    } catch (e) {
      setAiResponse(prev => prev || ('Error: ' + String(e)));
      setIsRunningAI(false);
    }
  };

  // ── Import / Export ────────────────────────────────────────────────────────
  const handleExport = async () => {
    try {
      const json = await exportApi.toJson();
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `promptvault-export-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError('Export failed: ' + String(e));
    }
  };

  const importFileRef = useRef<HTMLInputElement>(null);
  const handleImport = () => importFileRef.current?.click();

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    e.target.value = '';
    try {
      const text = await file.text();
      const result = await exportApi.fromJson({ json_content: text });
      if (result.success) {
        await loadPrompts();
        await loadCollections();
        await loadTags();
        setError(null);
        // Brief success toast via error slot (green)
        setError(`Imported ${result.prompts_imported} prompts, ${result.collections_imported} collections, ${result.tags_imported} tags.`);
        setTimeout(() => setError(null), 4000);
      } else {
        setError('Import failed: ' + result.message);
      }
    } catch (err) {
      setError('Import failed: ' + String(err));
    }
  };

  const handleCopy = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  const handleProviderChange = (provider: string) => {
    setSelectedProvider(provider);
    setSelectedModel(DEFAULT_MODELS[provider] ?? '');
    setAiResponse('');
    setModelFetchError(null);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files ?? []);
    if (!files.length) return;
    files.forEach(file => {
      const isImage = file.type.startsWith('image/');
      const sizeLabel = file.size < 1024
        ? `${file.size} B`
        : file.size < 1024 * 1024
          ? `${(file.size / 1024).toFixed(1)} KB`
          : `${(file.size / (1024 * 1024)).toFixed(1)} MB`;
      const id = `${file.name}-${Date.now()}-${Math.random()}`;

      if (isImage) {
        const reader = new FileReader();
        reader.onload = ev => {
          const dataUrl = ev.target?.result as string;
          // strip "data:<mime>;base64," prefix to get raw base64
          const base64 = dataUrl.split(',')[1] ?? '';
          setAttachedFiles(prev => [...prev, {
            id, name: file.name, content: base64, mimeType: file.type,
            isImage: true, previewUrl: dataUrl, sizeLabel,
          }]);
        };
        reader.readAsDataURL(file);
      } else {
        const reader = new FileReader();
        reader.onload = ev => {
          const text = ev.target?.result as string;
          setAttachedFiles(prev => [...prev, {
            id, name: file.name, content: text, mimeType: file.type || 'text/plain',
            isImage: false, sizeLabel,
          }]);
        };
        reader.readAsText(file);
      }
    });
    // reset so same file can be re-added
    e.target.value = '';
  };

  const removeAttachment = (id: string) => {
    setAttachedFiles(prev => prev.filter(f => f.id !== id));
  };

  // Command palette
  const commands = [
    { id: 'new-prompt', name: 'New Prompt', shortcut: 'Ctrl+N' },
    { id: 'search', name: 'Search Prompts', shortcut: 'Ctrl+K' },
    { id: 'run-ai', name: 'Run with AI', shortcut: 'Ctrl+Enter' },
    { id: 'copy-prompt', name: 'Copy to Clipboard' },
    { id: 'toggle-pin', name: 'Toggle Pin' },
    { id: 'duplicate', name: 'Duplicate Prompt' },
    { id: 'delete', name: 'Delete Prompt' },
  ];

  const filteredCommands = commands.filter(cmd =>
    cmd.name.toLowerCase().includes(commandSearch.toLowerCase())
  );

  const quickSearchResults = commandSearch.length > 1
    ? prompts.filter(p =>
        p.title.toLowerCase().includes(commandSearch.toLowerCase()) ||
        p.body.toLowerCase().includes(commandSearch.toLowerCase())
      ).slice(0, 5)
    : [];

  const handleCommandAction = async (action: string) => {
    setIsCommandPaletteOpen(false);
    setCommandSearch('');
    switch (action) {
      case 'new-prompt': setIsCreating(true); break;
      case 'search': break;
      case 'run-ai': if (selectedPrompt) handleRunAI(); break;
      case 'copy-prompt': if (selectedPrompt) handleCopy(selectedPrompt.body); break;
      case 'toggle-pin': if (selectedPrompt) await togglePin(selectedPrompt.id); break;
      case 'duplicate': if (selectedPrompt) await duplicatePrompt(selectedPrompt.id); break;
      case 'delete': if (selectedPrompt) await deletePrompt(selectedPrompt.id); break;
    }
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        onOpenSettings={() => setIsSettingsOpen(true)}
        onExport={handleExport}
        onImport={handleImport}
        width={sidebarWidth}
      />
      {/* Sidebar resize handle */}
      <div
        className="w-1 h-full cursor-col-resize flex-shrink-0 hover:bg-primary/40 active:bg-primary/60 transition-colors"
        onMouseDown={startResize(() => sidebarWidthRef.current, setSidebarWidth, 'h', 160, 400)}
      />

      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Error banner */}
        {error && (
          <div className="flex items-center justify-between px-4 py-2 bg-destructive/10 text-destructive text-sm border-b flex-shrink-0">
            <span>{error}</span>
            <button onClick={() => setError(null)} className="ml-2 hover:opacity-70">
              <X className="h-4 w-4" />
            </button>
          </div>
        )}

        {/* Header */}
        <div className="h-14 border-b flex items-center justify-between px-4 flex-shrink-0">
          <h1 className="text-lg font-semibold">
            {selectedCollectionId
              ? collections.find(c => c.id === selectedCollectionId)?.name
              : 'All Prompts'}
          </h1>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => setIsCommandPaletteOpen(true)}>
              <Search className="h-4 w-4 mr-2" />
              <span className="text-xs text-muted-foreground">Ctrl+K</span>
            </Button>
            <Button onClick={() => setIsCreating(true)}>
              <Plus className="h-4 w-4 mr-2" />
              New Prompt
            </Button>
          </div>
        </div>

        <div className="flex-1 flex overflow-hidden">
          {/* Prompt list */}
          <div
            className="border-r overflow-y-auto flex-shrink-0"
            style={{ width: promptListWidth, minWidth: promptListWidth, maxWidth: promptListWidth }}
          >
            {isLoading ? (
              <div className="p-4 text-center text-muted-foreground">Loading...</div>
            ) : filteredPrompts.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground">No prompts yet</div>
            ) : (
              <div className="p-2">
                {filteredPrompts.map(prompt => (
                  <button
                    key={prompt.id}
                    onClick={() => {
                      setSelectedPrompt(prompt.id);
                      setEditTitle(prompt.title);
                      setEditBody(prompt.body);
                      setIsEditing(false);
                      setAiResponse('');
                    }}
                    className={cn(
                      "w-full text-left p-3 rounded-lg mb-1 transition-colors",
                      selectedPromptId === prompt.id
                        ? "bg-primary/10 border border-primary/20"
                        : "hover:bg-muted"
                    )}
                  >
                    <div className="flex items-center gap-2">
                      {prompt.is_pinned && <Pin className="h-3 w-3 text-primary" />}
                      <span className="font-medium truncate text-sm">{prompt.title}</span>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      {truncate(prompt.body, 60)}
                    </p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {formatDate(prompt.updated_at)}
                    </p>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Prompt list resize handle */}
          <div
            className="w-1 h-full cursor-col-resize flex-shrink-0 hover:bg-primary/40 active:bg-primary/60 transition-colors"
            onMouseDown={startResize(() => promptListWidthRef.current, setPromptListWidth, 'h', 160, 600)}
          />

          {/* Main editor panel */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {isCreating ? (
              <div className="flex-1 p-4 flex flex-col gap-4">
                <Input
                  placeholder="Prompt title"
                  value={newTitle}
                  onChange={e => setNewTitle(e.target.value)}
                  className="text-lg font-medium"
                  autoFocus
                />
                <Textarea
                  placeholder="Write your prompt here... Use {variable_name} for variables."
                  value={newBody}
                  onChange={e => setNewBody(e.target.value)}
                  className="flex-1 font-mono text-sm resize-none"
                />
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setIsCreating(false)}>Cancel</Button>
                  <Button onClick={handleCreate}>Create Prompt</Button>
                </div>
              </div>
            ) : selectedPrompt ? (
              <div className="flex-1 flex flex-col overflow-hidden">
                {/* Action bar */}
                <div className="px-4 py-2 border-b flex items-center gap-2 flex-shrink-0 flex-wrap">
                  <Button variant="ghost" size="sm" title="Toggle pin" onClick={() => togglePin(selectedPrompt.id)}>
                    <Pin className={cn("h-4 w-4", selectedPrompt.is_pinned && "text-primary fill-primary")} />
                  </Button>
                  <Button variant="ghost" size="sm" title="Duplicate" onClick={() => duplicatePrompt(selectedPrompt.id)}>
                    <Files className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="sm" title="Copy" onClick={() => handleCopy(selectedPrompt.body)}>
                    <Copy className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="sm" title="Delete" onClick={() => deletePrompt(selectedPrompt.id)}>
                    <Trash2 className="h-4 w-4 text-destructive" />
                  </Button>
                </div>

                {/* Title */}
                <div className="px-4 py-3 border-b flex items-center justify-between gap-2 flex-shrink-0">
                  {isEditing ? (
                    <Input
                      value={editTitle}
                      onChange={e => setEditTitle(e.target.value)}
                      className="text-xl font-semibold"
                    />
                  ) : (
                    <h2 className="text-xl font-semibold flex-1">{selectedPrompt.title}</h2>
                  )}
                  <div className="flex gap-2 flex-shrink-0">
                    {isEditing ? (
                      <>
                        <Button size="sm" variant="outline" onClick={() => setIsEditing(false)}>Cancel</Button>
                        <Button size="sm" onClick={handleSaveEdit}>Save</Button>
                      </>
                    ) : (
                      <Button size="sm" variant="outline" onClick={() => setIsEditing(true)}>Edit</Button>
                    )}
                  </div>
                </div>

                {/* Body */}
                <div className="flex-1 overflow-auto p-4">
                  {isEditing ? (
                    <Textarea
                      value={editBody}
                      onChange={e => setEditBody(e.target.value)}
                      className="w-full h-full font-mono text-sm resize-none min-h-[200px]"
                    />
                  ) : (
                    <div
                      className="whitespace-pre-wrap font-mono text-sm cursor-text"
                      onClick={() => setIsEditing(true)}
                    >
                      {selectedPrompt.body}
                    </div>
                  )}
                </div>

                {/* AI Runner panel — resizable via top drag handle */}
                <div
                  className="border-t flex-shrink-0 flex flex-col"
                  style={{ height: aiPanelHeight, minHeight: aiPanelHeight, maxHeight: aiPanelHeight }}
                >
                  {/* Vertical resize handle */}
                  <div
                    className="h-1 w-full cursor-row-resize flex-shrink-0 hover:bg-primary/40 active:bg-primary/60 transition-colors"
                    onMouseDown={startResize(() => aiPanelHeightRef.current, setAiPanelHeight, 'v', 52, 600, true)}
                  />

                  {/* Mode tabs */}
                  <div className="px-4 pt-1.5 pb-0 flex items-center gap-1 flex-shrink-0 border-b">
                    {(
                      [
                        { id: 'run' as const, label: 'Run', icon: <Play className="h-3 w-3" /> },
                        { id: 'chat' as const, label: 'Chat', icon: <MessageSquare className="h-3 w-3" /> },
                        { id: 'ab' as const, label: 'A/B Test', icon: <SplitSquareHorizontal className="h-3 w-3" /> },
                      ] as const
                    ).map(tab => (
                      <button
                        key={tab.id}
                        onClick={() => setAiMode(tab.id)}
                        className={cn(
                          "flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-t-md transition-colors -mb-px border-b-2",
                          aiMode === tab.id
                            ? "border-primary text-primary font-medium bg-primary/5"
                            : "border-transparent text-muted-foreground hover:text-foreground"
                        )}
                      >
                        {tab.icon}{tab.label}
                      </button>
                    ))}
                  </div>

                  {/* ── Run mode ── */}
                  {aiMode === 'run' && (
                    <>
                      {/* Controls row */}
                      <div className="px-4 py-2 flex items-center gap-2 flex-wrap flex-shrink-0">
                        <Select value={selectedProvider} onValueChange={handleProviderChange}>
                          <SelectTrigger className="h-8 w-36 text-xs">
                            <SelectValue placeholder="Provider" />
                          </SelectTrigger>
                          <SelectContent>
                            {PROVIDERS.map(p => (
                              <SelectItem key={p.id} value={p.id}>{p.label}</SelectItem>
                            ))}
                          </SelectContent>
                        </Select>

                        {modelsLoading ? (
                          <div className="h-8 w-44 flex items-center px-3 rounded-md border border-input bg-background">
                            <Loader2 className="h-3 w-3 animate-spin text-muted-foreground mr-2" />
                            <span className="text-xs text-muted-foreground">Loading models…</span>
                          </div>
                        ) : modelFetchError ? (
                          <div className="flex items-center gap-1">
                            <div className="h-8 w-44 flex items-center px-2 rounded-md border border-destructive/50 bg-destructive/5" title={modelFetchError}>
                              <span className="text-xs text-destructive truncate">{modelFetchError.length > 28 ? 'Ollama not reachable' : modelFetchError}</span>
                            </div>
                            <Button variant="outline" size="sm" className="h-8 px-2 text-xs" onClick={() => setModelRetryTick(t => t + 1)}>
                              Retry
                            </Button>
                          </div>
                        ) : availableModels.length > 0 ? (
                          <Select value={selectedModel} onValueChange={setSelectedModel}>
                            <SelectTrigger className="h-8 w-44 text-xs">
                              <SelectValue placeholder="Select model" />
                            </SelectTrigger>
                            <SelectContent>
                              {availableModels.map(m => (
                                <SelectItem key={m.id} value={m.id}>{m.name}</SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <Input
                            value={selectedModel}
                            onChange={e => setSelectedModel(e.target.value)}
                            placeholder="Model name"
                            className="h-8 w-44 text-xs"
                          />
                        )}

                        <Button size="sm" className="h-8" onClick={handleRunAI} disabled={isRunningAI}>
                          {isRunningAI
                            ? <><Loader2 className="h-3.5 w-3.5 mr-2 animate-spin" />Running…</>
                            : <><Play className="h-3.5 w-3.5 mr-2" />Run</>
                          }
                        </Button>

                        <Button variant="outline" size="sm" className="h-8 px-2" title="Attach file" onClick={() => fileInputRef.current?.click()}>
                          <Paperclip className="h-3.5 w-3.5" />
                        </Button>

                        <button
                          onClick={() => setIsSettingsOpen(true)}
                          className={cn(
                            "text-xs hover:text-foreground ml-auto",
                            modelFetchError && selectedProvider === 'ollama'
                              ? "text-destructive font-medium"
                              : "text-muted-foreground"
                          )}
                        >
                          {modelFetchError && selectedProvider === 'ollama' ? 'Fix Ollama URL →' : 'Configure keys →'}
                        </button>
                      </div>

                      {/* Attached files chips */}
                      {attachedFiles.length > 0 && (
                        <div className="px-4 pb-1 flex flex-wrap gap-1.5">
                          {attachedFiles.map(file => (
                            <span
                              key={file.id}
                              className={cn(
                                "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border",
                                file.isImage
                                  ? "bg-amber-500/10 border-amber-500/30 text-amber-600 dark:text-amber-400"
                                  : "bg-muted border-border text-muted-foreground"
                              )}
                            >
                              {file.isImage ? <Image className="h-3 w-3 shrink-0" /> : <FileText className="h-3 w-3 shrink-0" />}
                              <span className="max-w-[120px] truncate">{file.name}</span>
                              <span className="opacity-60">({file.sizeLabel})</span>
                              <button onClick={() => removeAttachment(file.id)} className="ml-0.5 hover:text-destructive" title="Remove">
                                <X className="h-3 w-3" />
                              </button>
                            </span>
                          ))}
                        </div>
                      )}

                      {/* Image warning */}
                      {attachedFiles.some(f => f.isImage) && (
                        <div className="mx-4 mb-1 flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-600 dark:text-amber-400">
                          <AlertTriangle className="h-3.5 w-3.5 mt-0.5 shrink-0" />
                          <span>
                            Image attached — make sure you're using a <strong>vision-capable model</strong>{' '}
                            (e.g. <em>gpt-4o</em>, <em>claude-3</em>, <em>gemini-2.0-flash</em>, or <em>llava</em> for Ollama).
                            Mistral does not support images.
                          </span>
                        </div>
                      )}

                      {/* Streaming response */}
                      {aiResponse && (
                        <div className="mx-4 mb-3 flex-1 rounded-md border border-border bg-muted/30 p-3 overflow-y-auto min-h-0">
                          <div className="flex items-center justify-between mb-2 flex-shrink-0">
                            <span className="text-xs font-medium text-muted-foreground">
                              {selectedProvider} · {selectedModel}
                              {isRunningAI && <span className="ml-2 animate-pulse text-primary">●</span>}
                            </span>
                            <button
                              onClick={() => handleCopy(aiResponse)}
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                            >
                              <Copy className="h-3 w-3" /> Copy
                            </button>
                          </div>
                          <MarkdownRenderer content={aiResponse} />
                        </div>
                      )}

                      {isRunningAI && !aiResponse && (
                        <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm gap-2">
                          <Loader2 className="h-4 w-4 animate-spin" />
                          Generating…
                        </div>
                      )}
                    </>
                  )}

                  {/* ── Chat mode ── */}
                  {aiMode === 'chat' && (
                    <div className="flex-1 overflow-hidden flex flex-col min-h-0">
                      {/* Provider/model row for chat */}
                      <div className="px-4 py-2 flex items-center gap-2 flex-wrap flex-shrink-0 border-b bg-muted/20">
                        <Select value={selectedProvider} onValueChange={handleProviderChange}>
                          <SelectTrigger className="h-8 w-36 text-xs">
                            <SelectValue placeholder="Provider" />
                          </SelectTrigger>
                          <SelectContent>
                            {PROVIDERS.map(p => (
                              <SelectItem key={p.id} value={p.id}>{p.label}</SelectItem>
                            ))}
                          </SelectContent>
                        </Select>

                        {availableModels.length > 0 ? (
                          <Select value={selectedModel} onValueChange={setSelectedModel}>
                            <SelectTrigger className="h-8 w-44 text-xs">
                              <SelectValue placeholder="Select model" />
                            </SelectTrigger>
                            <SelectContent>
                              {availableModels.map(m => (
                                <SelectItem key={m.id} value={m.id}>{m.name}</SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <Input
                            value={selectedModel}
                            onChange={e => setSelectedModel(e.target.value)}
                            placeholder="Model name"
                            className="h-8 w-44 text-xs"
                          />
                        )}

                        <button
                          onClick={() => setIsSettingsOpen(true)}
                          className="text-xs text-muted-foreground hover:text-foreground ml-auto"
                        >
                          Configure keys →
                        </button>
                      </div>
                      <ConversationPanel
                        systemPrompt={isEditing ? editBody : (selectedPrompt?.body ?? '')}
                        provider={selectedProvider}
                        model={selectedModel}
                      />
                    </div>
                  )}

                  {/* ── A/B Test mode ── */}
                  {aiMode === 'ab' && (
                    <div className="flex-1 overflow-hidden min-h-0">
                      <ABTestPanel
                        promptText={isEditing ? editBody : (selectedPrompt?.body ?? '')}
                        onOpenSettings={() => setIsSettingsOpen(true)}
                      />
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="flex-1 flex items-center justify-center text-muted-foreground">
                Select a prompt or create a new one
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Hidden file input for attachments */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        className="hidden"
        onChange={handleFileSelect}
      />

      {/* Hidden file input for import */}
      <input
        ref={importFileRef}
        type="file"
        accept=".json,application/json"
        className="hidden"
        onChange={handleImportFile}
      />

      {/* Settings modal */}
      <SettingsModal open={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />

      {/* Command palette */}
      {isCommandPaletteOpen && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-24">
          <div className="absolute inset-0 bg-black/50" onClick={() => setIsCommandPaletteOpen(false)} />
          <div className="relative w-full max-w-lg bg-background rounded-lg shadow-lg border">
            <div className="flex items-center border-b px-3">
              <Search className="h-4 w-4 text-muted-foreground" />
              <input
                type="text"
                placeholder="Search prompts or type a command..."
                value={commandSearch}
                onChange={e => setCommandSearch(e.target.value)}
                className="flex-1 px-3 py-3 bg-transparent outline-none text-sm"
                autoFocus
              />
              <button onClick={() => setIsCommandPaletteOpen(false)}>
                <X className="h-4 w-4 text-muted-foreground" />
              </button>
            </div>
            <div className="max-h-96 overflow-y-auto p-2">
              {quickSearchResults.length > 0 && (
                <div className="mb-2">
                  <p className="px-2 py-1 text-xs text-muted-foreground">Prompts</p>
                  {quickSearchResults.map(prompt => (
                    <button
                      key={prompt.id}
                      onClick={() => {
                        setSelectedPrompt(prompt.id);
                        setEditTitle(prompt.title);
                        setEditBody(prompt.body);
                        setIsCommandPaletteOpen(false);
                        setCommandSearch('');
                      }}
                      className="w-full text-left px-3 py-2 rounded hover:bg-muted text-sm"
                    >
                      <span className="font-medium">{prompt.title}</span>
                      <p className="text-xs text-muted-foreground truncate">{prompt.body}</p>
                    </button>
                  ))}
                </div>
              )}
              <div>
                <p className="px-2 py-1 text-xs text-muted-foreground">Commands</p>
                {filteredCommands.map(cmd => (
                  <button
                    key={cmd.id}
                    onClick={() => handleCommandAction(cmd.id)}
                    className="w-full text-left px-3 py-2 rounded hover:bg-muted flex items-center justify-between text-sm"
                  >
                    <span>{cmd.name}</span>
                    {cmd.shortcut && <span className="text-xs text-muted-foreground">{cmd.shortcut}</span>}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
