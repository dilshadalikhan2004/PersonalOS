import { ArrowUp, Bot, Quote, Sparkles } from 'lucide-react';
import type { FormEvent } from 'react';
import { useEffect, useMemo, useState } from 'react';
import { invokeDesktop, isDesktopRuntime, listenDesktop } from '@/ui/lib/desktop';

type ChatRole = 'user' | 'assistant';

interface ChatCitation {
  uploadId: string;
  chunkIndex: number;
  excerpt: string;
}

interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  citations: ChatCitation[];
  createdAtUnixMs: number;
}

interface ChatTokenEvent {
  requestId: string;
  token: string;
}

export function ChatPage(): JSX.Element {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [question, setQuestion] = useState('');
  const [streamingId, setStreamingId] = useState<string | null>(null);
  const [draftAnswer, setDraftAnswer] = useState('');

  useEffect(() => {
    void invokeDesktop<ChatMessage[]>('get_chat_history')
      .then(setMessages)
      .catch(() => setMessages([]));
  }, []);

  useEffect(() => {
    let unsubscribe: (() => void) | undefined;
    if (!isDesktopRuntime()) return;
    void listenDesktop<ChatTokenEvent>('chat-token', (event) => {
      if (event.payload.requestId === streamingId) {
        setDraftAnswer((value) => value + event.payload.token);
      }
    }).then((cleanup) => {
      unsubscribe = cleanup;
    });
    return () => unsubscribe?.();
  }, [streamingId]);

  const hasMessages = messages.length > 0 || draftAnswer.length > 0;
  const suggestions = useMemo(
    () => ['Find my passport', 'Show electricity bill', 'Medical report'],
    [],
  );

  async function submit(event?: FormEvent): Promise<void> {
    event?.preventDefault();
    const trimmed = question.trim();
    if (!trimmed || streamingId) return;
    const requestId = crypto.randomUUID();
    const userMessage: ChatMessage = {
      id: `local-${requestId}`,
      role: 'user',
      content: trimmed,
      citations: [],
      createdAtUnixMs: Date.now(),
    };
    setMessages((value) => [...value, userMessage]);
    setQuestion('');
    setDraftAnswer('');
    setStreamingId(requestId);
    if (!isDesktopRuntime()) {
      setMessages((value) => [
        ...value,
        {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: 'Open the LifeOS desktop app to chat with local documents.',
          citations: [],
          createdAtUnixMs: Date.now(),
        },
      ]);
      setStreamingId(null);
      return;
    }
    try {
      const response = await invokeDesktop<ChatMessage>('ask_chat', {
        request: { requestId, question: trimmed },
      });
      setMessages((value) => [...value.filter((item) => item.id !== response.id), response]);
    } catch {
      setMessages((value) => [
        ...value,
        {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: "I don't know.",
          citations: [],
          createdAtUnixMs: Date.now(),
        },
      ]);
    } finally {
      setStreamingId(null);
      setDraftAnswer('');
    }
  }

  return (
    <div className="page-enter mx-auto flex min-h-[calc(100vh-105px)] max-w-4xl flex-col px-5 py-8 md:px-10">
      <div className="flex-1 space-y-4 overflow-hidden pb-6">
        {!hasMessages ? (
          <div className="mx-auto flex min-h-[52vh] max-w-2xl flex-col items-center justify-center text-center">
            <div className="mx-auto mb-6 grid size-12 place-items-center rounded-2xl bg-white/[0.05] text-zinc-200 ring-1 ring-white/10">
              <Bot size={22} />
            </div>
            <h2 className="text-2xl font-semibold text-zinc-100">Ask your documents</h2>
            <p className="mx-auto mt-3 max-w-md text-sm leading-6 text-zinc-500">
              Answers are grounded in local encrypted files. If LifeOS cannot find evidence, it will
              say it does not know.
            </p>
            <div className="mt-8 flex flex-wrap justify-center gap-2">
              {suggestions.map((prompt) => (
                <button
                  key={prompt}
                  onClick={() => setQuestion(prompt)}
                  className="rounded-full border border-white/[0.07] bg-white/[0.025] px-3 py-1.5 text-[11px] text-zinc-500 hover:bg-white/[0.06] hover:text-zinc-300"
                >
                  {prompt}
                </button>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {draftAnswer ? (
              <MessageBubble
                message={{
                  id: 'draft',
                  role: 'assistant',
                  content: draftAnswer,
                  citations: [],
                  createdAtUnixMs: Date.now(),
                }}
              />
            ) : null}
          </div>
        )}
      </div>

      <form
        onSubmit={(event) => void submit(event)}
        className="sticky bottom-4 rounded-2xl border border-white/[0.09] bg-zinc-950/85 p-2 shadow-xl shadow-black/30 backdrop-blur-xl"
      >
        <textarea
          value={question}
          onChange={(event) => setQuestion(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && !event.shiftKey) void submit();
          }}
          placeholder="Ask anything about your local documents..."
          className="min-h-[72px] w-full resize-none bg-transparent px-3 pt-3 text-sm text-zinc-200 outline-none placeholder:text-zinc-600"
        />
        <div className="flex items-center justify-between">
          <p className="flex items-center gap-1.5 px-2 text-[10px] text-zinc-600">
            <Sparkles size={11} /> Local RAG only
          </p>
          <button
            aria-label="Send"
            disabled={!question.trim() || Boolean(streamingId)}
            className="grid size-8 place-items-center rounded-lg bg-zinc-100 text-zinc-950 disabled:cursor-not-allowed disabled:opacity-40"
          >
            <ArrowUp size={16} />
          </button>
        </div>
      </form>
    </div>
  );
}

function MessageBubble({ message }: { message: ChatMessage }): JSX.Element {
  const isUser = message.role === 'user';
  return (
    <article className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[82%] rounded-2xl border px-4 py-3 text-sm leading-6 ${
          isUser
            ? 'border-white/[0.08] bg-white/[0.09] text-zinc-100'
            : 'border-white/[0.07] bg-white/[0.035] text-zinc-300'
        }`}
      >
        <p className="whitespace-pre-wrap">{message.content}</p>
        {message.citations.length ? (
          <div className="mt-3 space-y-2 border-t border-white/[0.06] pt-3">
            {message.citations.slice(0, 3).map((citation) => (
              <div key={`${citation.uploadId}-${citation.chunkIndex}`} className="flex gap-2">
                <Quote size={13} className="mt-1 shrink-0 text-zinc-600" />
                <p className="line-clamp-2 text-[11px] leading-5 text-zinc-500">
                  {citation.excerpt}
                </p>
              </div>
            ))}
          </div>
        ) : null}
      </div>
    </article>
  );
}
