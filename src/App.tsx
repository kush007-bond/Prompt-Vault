import { useState, useEffect } from 'react';
import { useAppStore } from './store';
import { Sidebar } from './components/Sidebar';
import { Button } from './components/ui/button';
import { Input } from './components/ui/input';
import { Textarea } from './components/ui/textarea';
import { Plus, Pin, Copy, Files, Trash2, Play, Search, X } from 'lucide-react';
import { cn, formatDate, truncate } from './lib/utils';
import { aiApi } from './lib/tauri';

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
    setSearchQuery,
    setError,
  } = useAppStore();

  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [editBody, setEditBody] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newBody, setNewBody] = useState('');
  const [aiResponse, setAiResponse] = useState('');
  const [isRunningAI, setIsRunningAI] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState('ollama');
  const [selectedModel, setSelectedModel] = useState('llama3');
  
  // Command palette state
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [commandSearch, setCommandSearch] = useState('');

  useEffect(() => {
    loadPrompts();
    loadCollections();
    loadTags();
    loadSettings();
  }, [loadPrompts, loadCollections, loadTags, loadSettings]);

  // Reset model to a sensible default when provider changes
  useEffect(() => {
    const defaults: Record<string, string> = {
      ollama: 'llama3',
      openai: 'gpt-4o',
      anthropic: 'claude-sonnet-4-6',
      gemini: 'gemini-2.0-flash',
    };
    setSelectedModel(defaults[selectedProvider] ?? '');
  }, [selectedProvider]);

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

  // Filter prompts
  const filteredPrompts = prompts.filter(p => {
    if (selectedCollectionId && p.collection_id !== selectedCollectionId) return false;
    if (searchQuery && !p.title.toLowerCase().includes(searchQuery.toLowerCase()) && 
        !p.body.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const selectedPrompt = prompts.find(p => p.id === selectedPromptId);

  const handleCreate = async () => {
    if (newTitle.trim()) {
      await createPrompt(newTitle.trim(), newBody.trim(), selectedCollectionId || undefined);
      setNewTitle('');
      setNewBody('');
      setIsCreating(false);
    }
  };

  const handleSaveEdit = async () => {
    if (selectedPrompt) {
      await updatePrompt(selectedPrompt.id, editTitle, editBody);
      setIsEditing(false);
    }
  };

  const handleRunAI = async () => {
    if (!selectedPrompt) return;
    setIsRunningAI(true);
    try {
      const response = await aiApi.runPrompt({
        provider: selectedProvider,
        model: selectedModel,
        prompt: editBody || selectedPrompt.body,
      });
      setAiResponse(response.content);
    } catch (e) {
      console.error('AI error:', e);
      setAiResponse('Error: ' + String(e));
    }
    setIsRunningAI(false);
  };

  const handleCopy = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  // Command palette actions
  const handleCommandAction = async (action: string, data?: any) => {
    setIsCommandPaletteOpen(false);
    setCommandSearch('');

    switch (action) {
      case 'new-prompt':
        setIsCreating(true);
        break;
      case 'search':
        setSearchQuery(data?.query || '');
        break;
      case 'run-ai':
        if (selectedPrompt) {
          handleRunAI();
        }
        break;
      case 'copy-prompt':
        if (selectedPrompt) {
          handleCopy(selectedPrompt.body);
        }
        break;
      case 'toggle-pin':
        if (selectedPrompt) {
          await togglePin(selectedPrompt.id);
        }
        break;
      case 'duplicate':
        if (selectedPrompt) {
          await duplicatePrompt(selectedPrompt.id);
        }
        break;
      case 'delete':
        if (selectedPrompt) {
          await deletePrompt(selectedPrompt.id);
        }
        break;
    }
  };

  const commands = [
    { id: 'new-prompt', name: 'New Prompt', shortcut: 'Ctrl+N' },
    { id: 'search', name: 'Search Prompts', shortcut: 'Ctrl+K' },
    { id: 'run-ai', name: 'Run with AI', shortcut: 'Ctrl+Enter' },
    { id: 'copy-prompt', name: 'Copy to Clipboard', shortcut: 'Ctrl+C' },
    { id: 'toggle-pin', name: 'Toggle Pin' },
    { id: 'duplicate', name: 'Duplicate Prompt' },
    { id: 'delete', name: 'Delete Prompt' },
  ];

  const filteredCommands = commands.filter(cmd => 
    cmd.name.toLowerCase().includes(commandSearch.toLowerCase())
  );

  // Quick search in command palette
  const quickSearchResults = commandSearch.length > 1 
    ? prompts.filter(p => 
        p.title.toLowerCase().includes(commandSearch.toLowerCase()) ||
        p.body.toLowerCase().includes(commandSearch.toLowerCase())
      ).slice(0, 5)
    : [];

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <Sidebar />

      {/* Main content */}
      <div className="flex-1 flex flex-col">
        {/* Error banner */}
        {error && (
          <div className="flex items-center justify-between px-4 py-2 bg-destructive/10 text-destructive text-sm border-b">
            <span>{error}</span>
            <button onClick={() => setError(null)} className="ml-2 hover:opacity-70">
              <X className="h-4 w-4" />
            </button>
          </div>
        )}

        {/* Header */}
        <div className="h-14 border-b flex items-center justify-between px-4">
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
          <div className="w-80 border-r overflow-y-auto">
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
                      <span className="font-medium truncate">{prompt.title}</span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-1">
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

          {/* Editor / View */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {isCreating ? (
              <div className="flex-1 p-4 flex flex-col gap-4">
                <Input
                  placeholder="Prompt title"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  className="text-lg font-medium"
                />
                <Textarea
                  placeholder="Write your prompt here... Use {variable_name} for variables."
                  value={newBody}
                  onChange={(e) => setNewBody(e.target.value)}
                  className="flex-1 font-mono text-sm"
                />
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setIsCreating(false)}>
                    Cancel
                  </Button>
                  <Button onClick={handleCreate}>Create Prompt</Button>
                </div>
              </div>
            ) : selectedPrompt ? (
              <div className="flex-1 flex flex-col">
                {/* Prompt actions */}
                <div className="p-4 border-b flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Button variant="ghost" size="sm" title="Toggle pin" onClick={() => togglePin(selectedPrompt.id)}>
                      <Pin className={cn("h-4 w-4", selectedPrompt.is_pinned && "text-primary")} />
                    </Button>
                    <Button variant="ghost" size="sm" title="Duplicate prompt" onClick={() => duplicatePrompt(selectedPrompt.id)}>
                      <Files className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="sm" title="Copy body to clipboard" onClick={() => handleCopy(selectedPrompt.body)}>
                      <Copy className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="sm" title="Delete prompt" onClick={() => deletePrompt(selectedPrompt.id)}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                  <div className="flex items-center gap-2">
                    <select
                      value={selectedProvider}
                      onChange={(e) => setSelectedProvider(e.target.value)}
                      className="h-8 rounded-md border border-input bg-background px-2 text-sm"
                    >
                      <option value="ollama">Ollama</option>
                      <option value="openai">OpenAI</option>
                      <option value="anthropic">Anthropic</option>
                      <option value="gemini">Gemini</option>
                    </select>
                    <input
                      type="text"
                      value={selectedModel}
                      onChange={(e) => setSelectedModel(e.target.value)}
                      placeholder="model name"
                      className="h-8 w-36 rounded-md border border-input bg-background px-2 text-sm outline-none focus:ring-1 focus:ring-ring"
                    />
                    <Button onClick={handleRunAI} disabled={isRunningAI}>
                      <Play className="h-4 w-4 mr-2" />
                      {isRunningAI ? 'Running...' : 'Run'}
                    </Button>
                  </div>
                </div>

                {/* Title */}
                <div className="p-4 border-b flex items-center justify-between">
                  {isEditing ? (
                    <Input
                      value={editTitle}
                      onChange={(e) => setEditTitle(e.target.value)}
                      className="text-xl font-semibold"
                    />
                  ) : (
                    <h2 className="text-xl font-semibold">{selectedPrompt.title}</h2>
                  )}
                  {isEditing && (
                    <div className="flex gap-2">
                      <Button size="sm" variant="outline" onClick={() => setIsEditing(false)}>
                        Cancel
                      </Button>
                      <Button size="sm" onClick={handleSaveEdit}>
                        Save
                      </Button>
                    </div>
                  )}
                </div>

                {/* Body */}
                <div className="flex-1 p-4 overflow-auto">
                  {isEditing ? (
                    <Textarea
                      value={editBody}
                      onChange={(e) => setEditBody(e.target.value)}
                      className="h-full font-mono text-sm"
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

                {/* AI Response */}
                {aiResponse && (
                  <div className="border-t p-4 bg-muted/30">
                    <h3 className="font-medium mb-2">AI Response</h3>
                    <pre className="whitespace-pre-wrap text-sm">{aiResponse}</pre>
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      className="mt-2"
                      onClick={() => handleCopy(aiResponse)}
                    >
                      <Copy className="h-4 w-4 mr-2" />
                      Copy
                    </Button>
                  </div>
                )}
              </div>
            ) : (
              <div className="flex-1 flex items-center justify-center text-muted-foreground">
                Select a prompt or create a new one
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Command Palette Modal */}
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
                onChange={(e) => setCommandSearch(e.target.value)}
                className="flex-1 px-3 py-3 bg-transparent outline-none"
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
                      className="w-full text-left px-3 py-2 rounded hover:bg-muted"
                    >
                      <span className="font-medium">{prompt.title}</span>
                      <p className="text-sm text-muted-foreground truncate">{prompt.body}</p>
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
                    className="w-full text-left px-3 py-2 rounded hover:bg-muted flex items-center justify-between"
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
