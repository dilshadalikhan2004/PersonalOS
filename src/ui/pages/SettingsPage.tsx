import {
  ChevronRight,
  LockKeyhole,
  Moon,
  Palette,
  ShieldCheck,
  Sparkles,
  UserRound,
} from 'lucide-react';
import { Panel, SectionHeading } from '@/ui/shared/primitives';

const settings = [
  [UserRound, 'Profile', 'Your name, preferences and profile details'],
  [Palette, 'Appearance', 'Dark mode, colors and interface density'],
  [LockKeyhole, 'Privacy', 'Your local data and permissions'],
  [Sparkles, 'Assistant', 'Local model preferences and behavior'],
  [ShieldCheck, 'Security', 'Encryption and device access'],
];
export function SettingsPage(): JSX.Element {
  return (
    <div className="page-enter mx-auto max-w-3xl p-5 md:p-8 lg:p-10">
      <p className="mb-9 text-sm text-zinc-500">Make LifeOS feel like your own.</p>
      <Panel className="mb-8 p-5">
        <div className="flex items-center gap-4">
          <div className="grid size-12 place-items-center rounded-2xl bg-gradient-to-br from-violet-300 to-indigo-700 text-sm font-bold text-white">
            AS
          </div>
          <div className="flex-1">
            <p className="text-sm font-medium text-zinc-200">Alex Smith</p>
            <p className="mt-1 text-xs text-zinc-600">This is your private, local workspace.</p>
          </div>
          <button className="rounded-lg border border-white/[0.08] px-3 py-1.5 text-xs text-zinc-400">
            Edit
          </button>
        </div>
      </Panel>
      <SectionHeading eyebrow="Preferences" title="Personalize your space" />
      <Panel className="divide-y divide-white/[0.06]">
        {settings.map(([Icon, title, description]) => {
          const SettingIcon = Icon as typeof UserRound;
          return (
            <button
              key={title as string}
              className="flex w-full items-center gap-4 px-5 py-4 text-left transition-colors hover:bg-white/[0.035]"
            >
              <span className="grid size-9 place-items-center rounded-xl bg-white/[0.05] text-zinc-400">
                <SettingIcon size={17} />
              </span>
              <span className="flex-1">
                <span className="block text-[13px] font-medium text-zinc-300">
                  {title as string}
                </span>
                <span className="mt-0.5 block text-[11px] text-zinc-600">
                  {description as string}
                </span>
              </span>
              <ChevronRight size={16} className="text-zinc-700" />
            </button>
          );
        })}
      </Panel>
      <SectionHeading eyebrow="Appearance" title="Theme" />
      <Panel className="flex items-center justify-between p-5">
        <div className="flex items-center gap-3">
          <span className="grid size-9 place-items-center rounded-xl bg-white/[0.05] text-zinc-400">
            <Moon size={17} />
          </span>
          <div>
            <p className="text-[13px] font-medium text-zinc-300">Dark</p>
            <p className="mt-0.5 text-[11px] text-zinc-600">Always use dark appearance</p>
          </div>
        </div>
        <div className="relative h-6 w-11 rounded-full bg-violet-500 p-1">
          <span className="ml-auto block size-4 rounded-full bg-white shadow-sm" />
        </div>
      </Panel>
    </div>
  );
}
