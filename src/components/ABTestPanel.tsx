import { useState, useEffect } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Play, Copy, Loader2, Clock } from 'lucide-react';
import { cn } from '../lib/utils';
import { aiApi, type RunPromptRequest, type ModelInfo } from '../lib/tauri';
import { MarkdownRenderer } from './MarkdownRenderer';

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

interface SlotState {
  provider: string;
  model: string;
  models: ModelInfo[];
  modelsLoading: boolean;
  response: string;
  loading: boolean;
  error: string | null;
  durationMs: number | null;
  tokensOut: number | null;
}

function makeSlot(provider = 'openai', model = 'gpt-4o'): SlotState {
  return { provider, model, models: [], modelsLoading: false, response: '', loading: false, error: null, durationMs: null, tokensOut: null };
}

interface ABTestPanelProps {
  promptText: string;
  onOpenSettings: () => void;
}

export function ABTestPanel({ promptText, onOpenSettings }: ABTestPanelProps) {
  const [slotA, setSlotA] = useState<SlotState>(() => makeSlot('openai', 'gpt-4o'));
  const [slotB, setSlotB] = useState<SlotState>(() => makeSlot('anthropic', 'claude-sonnet-4-6'));

  const patchA = (patch: Partial<SlotState>) => setSlotA(s => ({ ...s, ...patch }));
  const patchB = (patch: Partial<SlotState>) => setSlotB(s => ({ ...s, ...patch }));

  // Load models for each slot when provider changes
  const loadModels = async (
    provider: string,
    patch: (p: Partial<SlotState>) => void,
    currentModel: string,
  ) => {
    patch({ modelsLoading: true, models: [] });
    try {
      const models = await aiApi.listModels(provider);
      const bestModel = models.find(m => m.id === currentModel) ? currentModel : (models[0]?.id ?? DEFAULT_MODELS[provider] ?? '');
      patch({ models, model: bestModel, modelsLoading: false });
    } catch {
      patch({ modelsLoading: false });
    }
  };

  useEffect(() => { loadModels(slotA.provider, patchA, slotA.model); }, [slotA.provider]);
  useEffect(() => { loadModels(slotB.provider, patchB, slotB.model); }, [slotB.provider]);

  const runSlot = async (
    slot: SlotState,
    patch: (p: Partial<SlotState>) => void,
  ) => {
    patch({ loading: true, response: '', error: null, durationMs: null, tokensOut: null });
    const req: RunPromptRequest = { provider: slot.provider, model: slot.model, prompt: promptText };
    try {
      const start = Date.now();
      const res = await aiApi.runPrompt(req);
      patch({
        response: res.content,
        loading: false,
        durationMs: res.duration_ms ?? Date.now() - start,
        tokensOut: res.tokens_output ?? null,
      });
    } catch (e) {
      patch({ error: String(e), loading: false });
    }
  };

  const handleRunAB = () => {
    runSlot(slotA, patchA);
    runSlot(slotB, patchB);
  };

  const isRunning = slotA.loading || slotB.loading;

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Controls bar */}
      <div className="px-4 py-2 border-b flex items-center gap-3 flex-shrink-0 flex-wrap">
        <span className="text-xs font-medium text-muted-foreground">A/B Test</span>
        <Button size="sm" className="h-8" onClick={handleRunAB} disabled={isRunning || !promptText}>
          {isRunning
            ? <><Loader2 className="h-3.5 w-3.5 mr-2 animate-spin" />Running…</>
            : <><Play className="h-3.5 w-3.5 mr-2" />Run A/B</>
          }
        </Button>
        <button className="text-xs text-muted-foreground hover:text-foreground ml-auto" onClick={onOpenSettings}>
          Configure keys →
        </button>
      </div>

      {/* Two columns */}
      <div className="flex flex-1 overflow-hidden divide-x">
        <SlotColumn label="A" slot={slotA} patch={patchA} />
        <SlotColumn label="B" slot={slotB} patch={patchB} />
      </div>
    </div>
  );
}

function SlotColumn({
  label,
  slot,
  patch,
}: {
  label: string;
  slot: SlotState;
  patch: (p: Partial<SlotState>) => void;
}) {
  const handleCopy = () => navigator.clipboard.writeText(slot.response);

  return (
    <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
      {/* Slot header */}
      <div className="px-3 py-1.5 border-b bg-muted/30 flex items-center gap-2 flex-shrink-0 flex-wrap">
        <span className="text-xs font-bold text-primary">{label}</span>

        <Select
          value={slot.provider}
          onValueChange={v => patch({ provider: v, model: DEFAULT_MODELS[v] ?? '' })}
        >
          <SelectTrigger className="h-7 w-28 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {PROVIDERS.map(p => <SelectItem key={p.id} value={p.id}>{p.label}</SelectItem>)}
          </SelectContent>
        </Select>

        {slot.modelsLoading ? (
          <div className="h-7 w-32 flex items-center px-2 rounded-md border bg-background">
            <Loader2 className="h-3 w-3 animate-spin text-muted-foreground mr-1" />
            <span className="text-xs text-muted-foreground">Loading…</span>
          </div>
        ) : slot.models.length > 0 ? (
          <Select value={slot.model} onValueChange={v => patch({ model: v })}>
            <SelectTrigger className="h-7 w-40 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {slot.models.map(m => <SelectItem key={m.id} value={m.id}>{m.name}</SelectItem>)}
            </SelectContent>
          </Select>
        ) : (
          <Input
            value={slot.model}
            onChange={e => patch({ model: e.target.value })}
            className="h-7 w-32 text-xs"
            placeholder="Model"
          />
        )}

        {slot.durationMs !== null && (
          <span className="text-xs text-muted-foreground flex items-center gap-1 ml-auto">
            <Clock className="h-3 w-3" />{(slot.durationMs / 1000).toFixed(1)}s
            {slot.tokensOut != null && <span className="ml-1">· {slot.tokensOut} tok</span>}
          </span>
        )}
      </div>

      {/* Response body */}
      <div className="flex-1 overflow-y-auto p-3 relative min-h-0">
        {slot.loading && (
          <div className="flex items-center gap-2 text-muted-foreground text-sm">
            <Loader2 className="h-4 w-4 animate-spin" /> Generating…
          </div>
        )}
        {slot.error && (
          <div className="text-xs text-destructive bg-destructive/10 rounded-md p-2 border border-destructive/20">
            {slot.error}
          </div>
        )}
        {!slot.loading && slot.response && (
          <>
            <button
              onClick={handleCopy}
              className={cn(
                "absolute top-2 right-2 text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
              )}
            >
              <Copy className="h-3 w-3" /> Copy
            </button>
            <MarkdownRenderer content={slot.response} />
          </>
        )}
        {!slot.loading && !slot.response && !slot.error && (
          <div className="text-muted-foreground text-sm text-center mt-4">
            Click "Run A/B" to compare
          </div>
        )}
      </div>
    </div>
  );
}
