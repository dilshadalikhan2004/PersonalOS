import { ArrowRight, CalendarDays, Check, CirclePlay, Sparkles } from 'lucide-react';
import { AccentButton } from '@/ui/app/App';
import { Panel, SectionHeading } from '@/ui/shared/primitives';

const priorities = [
  { title: 'Review product narrative', area: 'Work · 10:00 AM', color: 'bg-violet-400' },
  { title: '30 minute focus walk', area: 'Personal · 1:30 PM', color: 'bg-emerald-400' },
  { title: 'Dinner with Maya', area: 'Personal · 7:00 PM', color: 'bg-amber-300' },
];

export function HomePage(): JSX.Element {
  return (
    <div className="page-enter mx-auto max-w-[1320px] p-5 md:p-8 lg:p-10">
      <div className="mb-10 flex flex-col justify-between gap-6 sm:flex-row sm:items-end">
        <div>
          <span className="inline-flex items-center gap-1.5 rounded-full border border-violet-400/20 bg-violet-400/10 px-2.5 py-1 text-[11px] font-medium text-violet-200">
            <Sparkles size={12} /> Tuesday, 15 July
          </span>
          <h2 className="mt-4 max-w-xl text-3xl font-semibold tracking-[-0.055em] text-white md:text-[38px]">
            Make room for what matters.
          </h2>
          <p className="mt-3 max-w-lg text-sm leading-6 text-zinc-500">
            A gentle view of your day, brought together in one private place.
          </p>
        </div>
        <AccentButton>Plan your day</AccentButton>
      </div>
      <div className="grid gap-5 xl:grid-cols-[1.35fr_0.9fr]">
        <Panel className="relative overflow-hidden p-5 md:p-6">
          <div className="absolute -right-12 -top-16 size-52 rounded-full bg-violet-500/[0.11] blur-3xl" />
          <SectionHeading
            eyebrow="Today"
            title="Your three priorities"
            action={<span className="text-xs text-zinc-500">3 of 3 open</span>}
          />
          <div className="relative space-y-2">
            {priorities.map((priority) => (
              <div
                key={priority.title}
                className="flex items-center gap-3 rounded-xl border border-white/[0.05] bg-black/10 p-3.5"
              >
                <button
                  aria-label={`Complete ${priority.title}`}
                  className="grid size-5 place-items-center rounded-full border border-zinc-600 text-transparent transition-colors hover:border-violet-300 hover:text-violet-200"
                >
                  <Check size={12} />
                </button>
                <span className={`size-1.5 rounded-full ${priority.color}`} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-[13px] font-medium text-zinc-200">{priority.title}</p>
                  <p className="mt-0.5 text-[11px] text-zinc-600">{priority.area}</p>
                </div>
                <ArrowRight size={15} className="text-zinc-700" />
              </div>
            ))}
          </div>
        </Panel>
        <Panel className="p-5 md:p-6">
          <SectionHeading eyebrow="Up next" title="Your schedule" />
          <div className="space-y-5 border-l border-white/[0.08] pl-4">
            <div className="relative">
              <span className="absolute -left-[19px] top-1.5 size-2 rounded-full bg-violet-400 ring-4 ring-violet-400/10" />
              <p className="text-[11px] text-zinc-600">10:00 AM</p>
              <p className="mt-1 text-[13px] font-medium text-zinc-200">Narrative review</p>
              <p className="mt-1 text-[11px] text-zinc-600">45 min · Studio</p>
            </div>
            <div className="relative">
              <span className="absolute -left-[19px] top-1.5 size-2 rounded-full bg-emerald-400" />
              <p className="text-[11px] text-zinc-600">1:30 PM</p>
              <p className="mt-1 text-[13px] font-medium text-zinc-200">Focus walk</p>
            </div>
            <div className="relative">
              <span className="absolute -left-[19px] top-1.5 size-2 rounded-full bg-amber-300" />
              <p className="text-[11px] text-zinc-600">7:00 PM</p>
              <p className="mt-1 text-[13px] font-medium text-zinc-200">Dinner with Maya</p>
            </div>
          </div>
        </Panel>
      </div>
      <div className="mt-9 grid gap-5 md:grid-cols-2">
        <div>
          <SectionHeading eyebrow="Continue" title="Pick up where you left off" />
          <Panel className="group flex items-center gap-4 p-4 transition-colors hover:bg-white/[0.06]">
            <div className="grid size-11 place-items-center rounded-xl bg-gradient-to-br from-fuchsia-400/30 to-violet-500/20 text-fuchsia-200">
              <CirclePlay size={19} />
            </div>
            <div className="flex-1">
              <p className="text-[13px] font-medium text-zinc-200">Reframing the next chapter</p>
              <p className="mt-1 text-[11px] text-zinc-600">Personal notes · edited yesterday</p>
            </div>
            <ArrowRight size={16} className="text-zinc-700" />
          </Panel>
        </div>
        <div>
          <SectionHeading eyebrow="A small note" title="A thought to keep" />
          <Panel className="p-4">
            <CalendarDays size={17} className="mb-4 text-zinc-600" />
            <p className="max-w-sm text-[13px] leading-6 text-zinc-400">
              “The future depends on what you do today.”
            </p>
            <p className="mt-2 text-[11px] text-zinc-600">Mahatma Gandhi</p>
          </Panel>
        </div>
      </div>
    </div>
  );
}
