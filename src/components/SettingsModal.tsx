import { useState, useEffect } from 'react';
import { Eye, EyeOff, CheckCircle2, XCircle, Loader2, Wifi } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { aiApi, settingsApi } from '../lib/tauri';

interface ProviderConfig {
  id: string;
  label: string;
  requiresKey: boolean;
  keyPlaceholder?: string;
  docsUrl?: string;
}

const PROVIDERS: ProviderConfig[] = [
  { id: 'ollama', label: 'Ollama', requiresKey: false },
  { id: 'openai', label: 'OpenAI', requiresKey: true, keyPlaceholder: 'sk-...' },
  { id: 'anthropic', label: 'Anthropic', requiresKey: true, keyPlaceholder: 'sk-ant-...' },
  { id: 'gemini', label: 'Google Gemini', requiresKey: true, keyPlaceholder: 'AIza...' },
  { id: 'mistral', label: 'Mistral AI', requiresKey: true, keyPlaceholder: 'Enter API key' },
];

type Status = 'idle' | 'checking' | 'ok' | 'error';

interface ProviderState {
  status: Status;
  error?: string;
  apiKey: string;
  showKey: boolean;
  saving: boolean;
  saved: boolean;
}

interface Props {
  open: boolean;
  onClose: () => void;
}

export function SettingsModal({ open, onClose }: Props) {
  const [providers, setProviders] = useState<Record<string, ProviderState>>(() =>
    Object.fromEntries(
      PROVIDERS.map(p => [p.id, { status: 'idle', apiKey: '', showKey: false, saving: false, saved: false }])
    )
  );
  const [ollamaUrl, setOllamaUrl] = useState('');
  const [ollamaUrlSaved, setOllamaUrlSaved] = useState(false);

  useEffect(() => {
    if (!open) return;
    PROVIDERS.forEach(p => checkProvider(p.id));
    settingsApi.get('ollama_base_url').then(val => {
      setOllamaUrl(val ?? '');
    }).catch(() => {});
  }, [open]);

  const checkProvider = async (id: string) => {
    setProviders(prev => ({ ...prev, [id]: { ...prev[id], status: 'checking', error: undefined } }));
    try {
      const health = await aiApi.healthCheck(id);
      setProviders(prev => ({
        ...prev,
        [id]: {
          ...prev[id],
          status: health.available ? 'ok' : 'error',
          error: health.error ?? undefined,
        },
      }));
    } catch (e) {
      setProviders(prev => ({
        ...prev,
        [id]: { ...prev[id], status: 'error', error: String(e) },
      }));
    }
  };

  const saveKey = async (id: string) => {
    const key = providers[id].apiKey.trim();
    if (!key) return;
    setProviders(prev => ({ ...prev, [id]: { ...prev[id], saving: true, saved: false } }));
    try {
      await aiApi.storeApiKey(id, key);
      setProviders(prev => ({ ...prev, [id]: { ...prev[id], saving: false, saved: true, apiKey: '' } }));
      setTimeout(() => {
        setProviders(prev => ({ ...prev, [id]: { ...prev[id], saved: false } }));
        checkProvider(id);
      }, 1500);
    } catch (e) {
      setProviders(prev => ({ ...prev, [id]: { ...prev[id], saving: false } }));
    }
  };

  const saveOllamaUrl = async () => {
    const url = ollamaUrl.trim() || 'http://localhost:11434';
    await settingsApi.set('ollama_base_url', url).catch(() => {});
    setOllamaUrl(url);
    setOllamaUrlSaved(true);
    setTimeout(() => {
      setOllamaUrlSaved(false);
      checkProvider('ollama');
    }, 1500);
  };

  const toggleShowKey = (id: string) => {
    setProviders(prev => ({ ...prev, [id]: { ...prev[id], showKey: !prev[id].showKey } }));
  };

  const setKey = (id: string, value: string) => {
    setProviders(prev => ({ ...prev, [id]: { ...prev[id], apiKey: value } }));
  };

  return (
    <Dialog open={open} onOpenChange={v => !v && onClose()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>AI Provider Settings</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 mt-2 max-h-[60vh] overflow-y-auto pr-1">
          {PROVIDERS.map(provider => {
            const state = providers[provider.id];
            return (
              <div key={provider.id} className="rounded-lg border border-border p-4 space-y-3">
                {/* Header row */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <StatusIcon status={state.status} />
                    <span className="font-medium text-sm">{provider.label}</span>
                  </div>
                  <button
                    onClick={() => checkProvider(provider.id)}
                    className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                    title="Recheck connection"
                  >
                    <Wifi className="h-3 w-3" />
                    Test
                  </button>
                </div>

                {/* Status message */}
                {state.status === 'ok' && (
                  <p className="text-xs text-green-500">
                    {provider.id === 'ollama' ? 'Connected to local Ollama' : 'API key configured and working'}
                  </p>
                )}
                {state.status === 'error' && state.error && (
                  <p className="text-xs text-destructive">{state.error}</p>
                )}

                {/* Ollama URL config */}
                {provider.id === 'ollama' && (
                  <div className="space-y-1.5">
                    <p className="text-xs text-muted-foreground">Server URL (default: http://localhost:11434)</p>
                    <div className="flex gap-2">
                      <Input
                        placeholder="http://localhost:11434"
                        value={ollamaUrl}
                        onChange={e => setOllamaUrl(e.target.value)}
                        onKeyDown={e => e.key === 'Enter' && saveOllamaUrl()}
                        className="h-8 text-xs font-mono"
                      />
                      <Button size="sm" className="h-8 px-3 text-xs" onClick={saveOllamaUrl}>
                        {ollamaUrlSaved ? 'Saved!' : 'Save'}
                      </Button>
                    </div>
                    {state.status !== 'ok' && (
                      <p className="text-xs text-muted-foreground">
                        Make sure Ollama is running. On Windows, try <code className="bg-muted px-1 rounded">http://127.0.0.1:11434</code> if localhost doesn't work.
                      </p>
                    )}
                  </div>
                )}

                {/* API key input for providers that need it */}
                {provider.requiresKey && (
                  <div className="flex gap-2">
                    <div className="relative flex-1">
                      <Input
                        type={state.showKey ? 'text' : 'password'}
                        placeholder={state.status === 'ok' ? '••••••••• (already set)' : provider.keyPlaceholder}
                        value={state.apiKey}
                        onChange={e => setKey(provider.id, e.target.value)}
                        onKeyDown={e => e.key === 'Enter' && saveKey(provider.id)}
                        className="pr-9 text-sm h-8"
                      />
                      <button
                        type="button"
                        onClick={() => toggleShowKey(provider.id)}
                        className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                      >
                        {state.showKey ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
                      </button>
                    </div>
                    <Button
                      size="sm"
                      className="h-8 px-3 text-xs"
                      onClick={() => saveKey(provider.id)}
                      disabled={!state.apiKey.trim() || state.saving}
                    >
                      {state.saving ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : state.saved ? (
                        'Saved!'
                      ) : (
                        'Save'
                      )}
                    </Button>
                  </div>
                )}
              </div>
            );
          })}
        </div>

        <p className="text-xs text-muted-foreground mt-2">
          API keys are stored securely in your OS keychain and never leave your device.
        </p>
      </DialogContent>
    </Dialog>
  );
}

function StatusIcon({ status }: { status: Status }) {
  if (status === 'checking') return <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />;
  if (status === 'ok') return <CheckCircle2 className="h-4 w-4 text-green-500" />;
  if (status === 'error') return <XCircle className="h-4 w-4 text-destructive" />;
  return <div className="h-4 w-4 rounded-full border-2 border-muted-foreground/30" />;
}
