import { useState, useRef, useEffect } from 'react';
import { Button } from './ui/button';
import { Textarea } from './ui/textarea';
import { Send, Trash2, Loader2, Bot, User } from 'lucide-react';
import { cn } from '../lib/utils';
import { aiApi, type Message } from '../lib/tauri';
import { MarkdownRenderer } from './MarkdownRenderer';

interface ConversationPanelProps {
  systemPrompt: string;
  provider: string;
  model: string;
}

export function ConversationPanel({ systemPrompt, provider, model }: ConversationPanelProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Scroll to bottom whenever messages change
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isLoading]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || isLoading) return;

    const userMessage: Message = { role: 'user', content: text };
    const nextMessages: Message[] = [...messages, userMessage];
    setMessages(nextMessages);
    setInput('');
    setIsLoading(true);
    setError(null);

    try {
      const fullMessages: Message[] = [
        { role: 'system', content: systemPrompt },
        ...nextMessages,
      ];
      const response = await aiApi.runConversation({
        provider,
        model,
        messages: fullMessages,
      });
      setMessages(prev => [...prev, { role: 'assistant', content: response.content }]);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
      textareaRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleClear = () => {
    setMessages([]);
    setError(null);
    textareaRef.current?.focus();
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* System prompt context bar */}
      <div className="px-4 py-2 bg-muted/40 border-b flex items-start gap-2 flex-shrink-0">
        <span className="text-xs font-medium text-muted-foreground mt-0.5 shrink-0">System:</span>
        <p className="text-xs text-muted-foreground truncate">{systemPrompt || '(no prompt body)'}</p>
        {messages.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto h-6 px-2 text-xs shrink-0"
            onClick={handleClear}
            title="Clear conversation"
          >
            <Trash2 className="h-3 w-3 mr-1" />
            Clear
          </Button>
        )}
      </div>

      {/* Message list */}
      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-3 min-h-0">
        {messages.length === 0 && (
          <div className="h-full flex items-center justify-center text-muted-foreground text-sm">
            Start a conversation using the prompt above as context.
          </div>
        )}
        {messages.map((msg, idx) => (
          <div
            key={idx}
            className={cn(
              'flex gap-2',
              msg.role === 'user' ? 'justify-end' : 'justify-start'
            )}
          >
            {msg.role === 'assistant' && (
              <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center shrink-0 mt-0.5">
                <Bot className="h-3.5 w-3.5 text-primary" />
              </div>
            )}
            <div
              className={cn(
                'max-w-[80%] rounded-lg px-3 py-2 text-sm',
                msg.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted border border-border'
              )}
            >
              {msg.role === 'assistant' ? (
                <MarkdownRenderer content={msg.content} />
              ) : (
                <p className="whitespace-pre-wrap">{msg.content}</p>
              )}
            </div>
            {msg.role === 'user' && (
              <div className="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center shrink-0 mt-0.5">
                <User className="h-3.5 w-3.5 text-primary" />
              </div>
            )}
          </div>
        ))}
        {isLoading && (
          <div className="flex gap-2 justify-start">
            <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center shrink-0 mt-0.5">
              <Bot className="h-3.5 w-3.5 text-primary" />
            </div>
            <div className="bg-muted border border-border rounded-lg px-3 py-2">
              <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
            </div>
          </div>
        )}
        {error && (
          <div className="text-xs text-destructive bg-destructive/10 rounded-md px-3 py-2 border border-destructive/20">
            {error}
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input area */}
      <div className="px-4 py-2 border-t flex gap-2 items-end flex-shrink-0">
        <Textarea
          ref={textareaRef}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a message… (Enter to send, Shift+Enter for newline)"
          className="flex-1 resize-none text-sm min-h-[38px] max-h-[120px]"
          rows={1}
          disabled={isLoading}
        />
        <Button
          size="sm"
          onClick={handleSend}
          disabled={isLoading || !input.trim()}
          className="h-9 px-3 shrink-0"
        >
          {isLoading
            ? <Loader2 className="h-4 w-4 animate-spin" />
            : <Send className="h-4 w-4" />
          }
        </Button>
      </div>
    </div>
  );
}
