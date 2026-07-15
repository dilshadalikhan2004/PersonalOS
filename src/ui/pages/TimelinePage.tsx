import {
  Calendar,
  ChevronDown,
  CircleDot,
  FileText,
  HeartPulse,
  RefreshCw,
  ShieldCheck,
  ShoppingBag,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { invokeDesktop } from '@/ui/lib/desktop';
import { Panel } from '@/ui/shared/primitives';

interface LifeTimelineEvent {
  id: string;
  title: string;
  category: string;
  date: string;
  source: string;
  confidence: number;
}

const categoryIcons: Record<string, typeof Calendar> = {
  Asset: ShoppingBag,
  Health: HeartPulse,
  Identity: FileText,
  Protection: ShieldCheck,
  Upload: FileText,
};

export function TimelinePage(): JSX.Element {
  const [events, setEvents] = useState<LifeTimelineEvent[]>([]);
  const [activeCategory, setActiveCategory] = useState('All');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const loadTimeline = async (): Promise<void> => {
    setLoading(true);
    await invokeDesktop<LifeTimelineEvent[]>('get_life_timeline')
      .then(setEvents)
      .catch(() => setEvents([]))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    void loadTimeline();
  }, []);

  const categories = useMemo(
    () => ['All', ...Array.from(new Set(events.map((event) => event.category))).sort()],
    [events],
  );
  const visibleEvents = events.filter(
    (event) => activeCategory === 'All' || event.category === activeCategory,
  );

  return (
    <div className="page-enter mx-auto max-w-5xl p-5 md:p-8 lg:p-10">
      <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <p className="text-sm text-zinc-500">
            A chronological life timeline generated from local documents.
          </p>
        </div>
        <button
          onClick={() => void loadTimeline()}
          className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/[0.08] px-3 py-2 text-xs text-zinc-400 hover:bg-white/[0.04]"
        >
          <RefreshCw size={14} /> Regenerate
        </button>
      </div>

      <div className="mt-6 flex flex-wrap gap-2">
        {categories.map((category) => (
          <button
            key={category}
            onClick={() => setActiveCategory(category)}
            className={`rounded-full border px-3 py-1.5 text-[11px] ${
              activeCategory === category
                ? 'border-zinc-200 bg-zinc-100 text-zinc-950'
                : 'border-white/[0.08] bg-white/[0.025] text-zinc-500 hover:bg-white/[0.06] hover:text-zinc-300'
            }`}
          >
            {category}
          </button>
        ))}
      </div>

      {visibleEvents.length ? (
        <div className="relative mt-10 pl-8 md:pl-28">
          <div className="absolute bottom-5 left-[13px] top-5 w-px bg-gradient-to-b from-zinc-300/50 via-white/[0.08] to-transparent md:left-[77px]" />
          {visibleEvents.map((event) => {
            const Icon = categoryIcons[event.category] ?? CircleDot;
            const expanded = expandedId === event.id;
            return (
              <div
                key={`${event.id}-${event.title}`}
                className="relative mb-5 grid grid-cols-[auto_1fr] gap-4"
              >
                <time className="absolute -left-28 top-5 hidden w-24 text-right text-[11px] text-zinc-600 md:block">
                  {event.date}
                </time>
                <span className="relative z-10 mt-5 size-3 rounded-full border-[3px] border-[#17181b] bg-zinc-200" />
                <Panel className="overflow-hidden">
                  <button
                    onClick={() => setExpandedId(expanded ? null : event.id)}
                    className="flex w-full gap-4 p-5 text-left"
                  >
                    <div className="grid size-9 shrink-0 place-items-center rounded-xl bg-white/[0.05] text-zinc-400">
                      <Icon size={17} />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="text-[13px] font-medium text-zinc-200">{event.title}</p>
                      <p className="mt-1 text-xs text-zinc-600 md:hidden">{event.date}</p>
                      <p className="mt-1 text-xs text-zinc-600">{event.source}</p>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="rounded-full bg-white/[0.04] px-2 py-1 text-[10px] text-zinc-500">
                        {event.category}
                      </span>
                      <ChevronDown
                        size={15}
                        className={`text-zinc-600 transition-transform ${expanded ? 'rotate-180' : ''}`}
                      />
                    </div>
                  </button>
                  {expanded ? (
                    <div className="border-t border-white/[0.06] px-5 py-4 text-xs leading-6 text-zinc-500">
                      Generated locally from encrypted document metadata. Confidence{' '}
                      {Math.round(event.confidence * 100)}%.
                    </div>
                  ) : null}
                </Panel>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="mt-10 rounded-2xl border border-dashed border-white/[0.08] bg-white/[0.02] px-5 py-14 text-center">
          <Calendar size={24} className="mx-auto mb-3 text-zinc-600" />
          <p className="text-sm text-zinc-500">
            {loading
              ? 'Generating timeline from local data...'
              : 'Upload documents with dates to generate your life timeline.'}
          </p>
        </div>
      )}
    </div>
  );
}
