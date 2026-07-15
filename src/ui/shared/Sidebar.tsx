import { Command, MoreHorizontal, Plus, Settings } from 'lucide-react';
import type { PageKey } from '@/ui/shared/navigation';
import { navigationItems } from '@/ui/shared/navigation';

export function Sidebar({ activePage }: { activePage: PageKey }): JSX.Element {
  return (
    <aside className="relative z-10 mr-3 hidden w-[232px] shrink-0 flex-col rounded-[25px] border border-white/[0.07] bg-[#191a1d]/90 p-3 shadow-2xl shadow-black/20 backdrop-blur-xl md:flex">
      <a href="#home" className="mb-8 flex items-center gap-2.5 px-2 py-2">
        <span className="grid size-8 place-items-center rounded-[10px] bg-gradient-to-br from-violet-300 via-violet-500 to-indigo-700 shadow-lg shadow-violet-900/40">
          <Command size={17} className="text-white" strokeWidth={2.5} />
        </span>
        <span className="text-[15px] font-semibold tracking-[-0.03em]">LifeOS</span>
      </a>
      <nav className="space-y-1">
        {navigationItems.map(({ key, label, icon: Icon }) => (
          <a
            key={key}
            href={`#${key}`}
            className={`group flex items-center gap-3 rounded-xl px-3 py-2.5 text-[13px] transition-colors ${
              activePage === key
                ? 'bg-white/[0.09] font-medium text-white shadow-sm'
                : 'text-zinc-500 hover:bg-white/[0.04] hover:text-zinc-300'
            }`}
          >
            <Icon size={17} strokeWidth={activePage === key ? 2 : 1.8} />
            {label}
          </a>
        ))}
      </nav>
      <div className="mt-8 border-t border-white/[0.06] pt-5">
        <p className="px-3 text-[10px] font-semibold uppercase tracking-[0.16em] text-zinc-600">
          Collections
        </p>
        <a
          href="#documents"
          className="mt-2 flex items-center gap-3 rounded-xl px-3 py-2 text-[13px] text-zinc-500 hover:bg-white/[0.04]"
        >
          <span className="size-2 rounded-full bg-amber-300" /> Personal
        </a>
        <a
          href="#documents"
          className="flex items-center gap-3 rounded-xl px-3 py-2 text-[13px] text-zinc-500 hover:bg-white/[0.04]"
        >
          <span className="size-2 rounded-full bg-sky-400" /> Work
        </a>
        <button className="mt-2 flex items-center gap-2 px-3 text-xs text-zinc-600 hover:text-zinc-400">
          <Plus size={14} /> New collection
        </button>
      </div>
      <div className="mt-auto space-y-2">
        <a
          href="#settings"
          className="flex items-center gap-3 rounded-xl px-3 py-2.5 text-[13px] text-zinc-500 hover:bg-white/[0.04] hover:text-zinc-300"
        >
          <Settings size={17} /> Settings
        </a>
        <div className="flex items-center gap-2 rounded-xl bg-black/15 p-2">
          <div className="grid size-7 place-items-center rounded-lg bg-zinc-700 text-[10px] font-bold text-zinc-200">
            AS
          </div>
          <span className="min-w-0 flex-1 truncate text-xs font-medium text-zinc-300">
            Alex Smith
          </span>
          <button aria-label="Account options" className="text-zinc-600">
            <MoreHorizontal size={16} />
          </button>
        </div>
      </div>
    </aside>
  );
}
