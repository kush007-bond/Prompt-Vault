import React from 'react';
import { cn } from '../lib/utils';

type Block =
  | { type: 'heading'; level: 1 | 2 | 3; text: string }
  | { type: 'code'; lang: string; content: string }
  | { type: 'ul'; items: string[] }
  | { type: 'ol'; items: string[] }
  | { type: 'paragraph'; text: string }
  | { type: 'hr' };

function parseBlocks(markdown: string): Block[] {
  const lines = markdown.split('\n');
  const blocks: Block[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Fenced code block
    if (line.startsWith('```')) {
      const lang = line.slice(3).trim();
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith('```')) {
        codeLines.push(lines[i]);
        i++;
      }
      blocks.push({ type: 'code', lang, content: codeLines.join('\n') });
      i++; // skip closing ```
      continue;
    }

    // Headings
    const headingMatch = line.match(/^(#{1,3})\s+(.+)/);
    if (headingMatch) {
      blocks.push({
        type: 'heading',
        level: Math.min(headingMatch[1].length, 3) as 1 | 2 | 3,
        text: headingMatch[2],
      });
      i++;
      continue;
    }

    // Horizontal rule
    if (/^[-*_]{3,}\s*$/.test(line.trim()) && line.trim().length > 0) {
      blocks.push({ type: 'hr' });
      i++;
      continue;
    }

    // Unordered list
    if (/^[-*+]\s/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^[-*+]\s/.test(lines[i])) {
        items.push(lines[i].slice(2));
        i++;
      }
      blocks.push({ type: 'ul', items });
      continue;
    }

    // Ordered list
    if (/^\d+\.\s/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\d+\.\s/.test(lines[i])) {
        items.push(lines[i].replace(/^\d+\.\s/, ''));
        i++;
      }
      blocks.push({ type: 'ol', items });
      continue;
    }

    // Skip blank lines
    if (line.trim() === '') {
      i++;
      continue;
    }

    // Paragraph — collect consecutive non-special lines
    const paraLines: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== '' &&
      !lines[i].startsWith('#') &&
      !lines[i].startsWith('```') &&
      !/^[-*+]\s/.test(lines[i]) &&
      !/^\d+\.\s/.test(lines[i]) &&
      !/^[-*_]{3,}\s*$/.test(lines[i].trim())
    ) {
      paraLines.push(lines[i]);
      i++;
    }
    if (paraLines.length > 0) {
      blocks.push({ type: 'paragraph', text: paraLines.join('\n') });
    }
  }

  return blocks;
}

function parseInline(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  // Inline code first (prevents nesting), then bold (**/__), then italic (*/_)
  const regex = /(`[^`\n]+`)|(\*\*(?:[^*]|\*(?!\*))+\*\*)|(__(?:[^_]|_(?!_))+__)|(\*(?:[^*\n])+\*)|(_(?:[^_\n])+_)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(<span key={key++}>{text.slice(lastIndex, match.index)}</span>);
    }
    const full = match[0];
    if (full.startsWith('`')) {
      parts.push(
        <code
          key={key++}
          className="px-1 py-0.5 rounded bg-muted font-mono text-xs border border-border"
        >
          {full.slice(1, -1)}
        </code>
      );
    } else if (full.startsWith('**') || full.startsWith('__')) {
      parts.push(
        <strong key={key++} className="font-semibold">
          {full.slice(2, -2)}
        </strong>
      );
    } else if (full.startsWith('*') || full.startsWith('_')) {
      parts.push(
        <em key={key++} className="italic">
          {full.slice(1, -1)}
        </em>
      );
    }
    lastIndex = match.index + full.length;
  }

  if (lastIndex < text.length) {
    parts.push(<span key={key++}>{text.slice(lastIndex)}</span>);
  }

  return <>{parts}</>;
}

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export function MarkdownRenderer({ content, className }: MarkdownRendererProps) {
  const blocks = parseBlocks(content);

  return (
    <div className={cn('space-y-2 text-sm', className)}>
      {blocks.map((block, idx) => {
        switch (block.type) {
          case 'heading': {
            const classes: Record<1 | 2 | 3, string> = {
              1: 'text-base font-bold mt-1',
              2: 'text-sm font-bold mt-1',
              3: 'text-sm font-semibold mt-0.5',
            };
            if (block.level === 1) return <h1 key={idx} className={classes[1]}>{parseInline(block.text)}</h1>;
            if (block.level === 2) return <h2 key={idx} className={classes[2]}>{parseInline(block.text)}</h2>;
            return <h3 key={idx} className={classes[3]}>{parseInline(block.text)}</h3>;
          }
          case 'code':
            return (
              <pre
                key={idx}
                className="rounded-md bg-muted border border-border p-3 overflow-x-auto"
              >
                <code className="font-mono text-xs">{block.content}</code>
              </pre>
            );
          case 'ul':
            return (
              <ul key={idx} className="list-disc list-inside space-y-0.5 pl-2">
                {block.items.map((item, i) => (
                  <li key={i}>{parseInline(item)}</li>
                ))}
              </ul>
            );
          case 'ol':
            return (
              <ol key={idx} className="list-decimal list-inside space-y-0.5 pl-2">
                {block.items.map((item, i) => (
                  <li key={i}>{parseInline(item)}</li>
                ))}
              </ol>
            );
          case 'paragraph':
            return (
              <p key={idx} className="leading-relaxed whitespace-pre-wrap">
                {parseInline(block.text)}
              </p>
            );
          case 'hr':
            return <hr key={idx} className="border-border my-1" />;
          default:
            return null;
        }
      })}
    </div>
  );
}
