import { ArrowUpRight, Bell, Command, Search } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { DashboardPage } from '@/ui/pages/DashboardPage';
import { DocumentsPage } from '@/ui/pages/DocumentsPage';
import { HomePage } from '@/ui/pages/HomePage';
import { SettingsPage } from '@/ui/pages/SettingsPage';
import { TimelinePage } from '@/ui/pages/TimelinePage';
import { ChatPage } from '@/ui/pages/ChatPage';
import { invokeDesktop } from '@/ui/lib/desktop';
import { Sidebar } from '@/ui/shared/Sidebar';
import type { PageKey } from '@/ui/shared/navigation';
import { pageMetadata } from '@/ui/shared/navigation';

function readPage(): PageKey {
  const requestedPage = window.location.hash.slice(1) as PageKey;
  return requestedPage in pageMetadata ? requestedPage : 'home';
}

const pages: Record<PageKey, () => JSX.Element> = {
  home: HomePage,
  dashboard: DashboardPage,
  chat: ChatPage,
  documents: DocumentsPage,
  timeline: TimelinePage,
  settings: SettingsPage,
};

interface LocalReminder {
  id: string;
  title: string;
  reminderType: string;
  dueDate: string;
  daysUntil: number;
  severity: 'overdue' | 'critical' | 'soon' | 'later';
  source: string;
}

export function App(): JSX.Element {
  const [page, setPage] = useState<PageKey>(readPage);
  const [reminders, setReminders] = useState<LocalReminder[]>([]);
  const [showReminders, setShowReminders] = useState(false);
  const [toast, setToast] = useState<LocalReminder | null>(null);

  useEffect(() => {
    const syncPage = (): void => setPage(readPage());
    window.addEventListener('hashchange', syncPage);
    return () => window.removeEventListener('hashchange', syncPage);
  }, []);

  useEffect(() => {
    void invokeDesktop<LocalReminder[]>('get_local_reminders')
      .then((items) => {
        setReminders(items);
        const urgent = items.find(
          (item) => item.severity === 'overdue' || item.severity === 'critical',
        );
        if (urgent) {
          setToast(urgent);
          showLocalNotification(urgent);
        }
      })
      .catch(() => setReminders([]));
  }, []);

  const CurrentPage = pages[page];
  const metadata = pageMetadata[page];
  const urgentCount = reminders.filter(
    (item) => item.severity === 'overdue' || item.severity === 'critical',
  ).length;

  return (
    <main className="app-shell min-h-screen overflow-hidden bg-[#101113] text-[#f4f4f5]">
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />
      <div className="relative mx-auto flex min-h-screen max-w-[1728px] p-3 md:p-4">
        <Sidebar activePage={page} />
        <section className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-[25px] border border-white/[0.07] bg-[#17181b]/85 shadow-2xl shadow-black/20 backdrop-blur-xl">
          <header className="flex h-[72px] shrink-0 items-center justify-between border-b border-white/[0.06] px-5 md:px-8">
            <div>
              <p className="text-[11px] font-medium uppercase tracking-[0.18em] text-zinc-500">
                LifeOS / {metadata.eyebrow}
              </p>
              <h1 className="mt-0.5 text-lg font-semibold tracking-[-0.03em] text-zinc-100">
                {metadata.title}
              </h1>
            </div>
            <div className="flex items-center gap-2">
              <button aria-label="Search" className="header-button hidden sm:flex">
                <Search size={16} />
                <span className="ml-2 text-xs text-zinc-400">Search</span>
                <kbd className="ml-4 rounded border border-white/10 bg-white/[0.04] px-1.5 py-0.5 text-[10px] text-zinc-500">
                  ⌘ K
                </kbd>
              </button>
              <button
                aria-label="Notifications"
                onClick={() => setShowReminders((value) => !value)}
                className="header-icon-button relative"
              >
                <Bell size={17} />
                {urgentCount ? (
                  <span className="absolute -right-1 -top-1 grid size-4 place-items-center rounded-full bg-rose-400 text-[9px] font-semibold text-zinc-950">
                    {urgentCount}
                  </span>
                ) : null}
              </button>
              {showReminders ? <ReminderPopover reminders={reminders} /> : null}
              <div className="ml-1 grid size-8 place-items-center rounded-full bg-gradient-to-br from-[#c7a6ff] to-[#7657d7] text-[11px] font-bold text-white shadow-lg shadow-violet-900/30">
                AS
              </div>
            </div>
          </header>
          <div className="min-h-0 flex-1 overflow-y-auto">
            <CurrentPage />
          </div>
        </section>
      </div>
      {toast ? (
        <button
          onClick={() => setToast(null)}
          className="fixed bottom-5 right-5 z-50 max-w-sm rounded-2xl border border-white/[0.08] bg-zinc-950/90 p-4 text-left shadow-2xl shadow-black/40 backdrop-blur-xl"
        >
          <p className="text-xs font-medium text-zinc-200">{toast.title}</p>
          <p className="mt-1 text-[11px] text-zinc-500">{reminderDueText(toast)}</p>
        </button>
      ) : null}
    </main>
  );
}

function ReminderPopover({ reminders }: { reminders: LocalReminder[] }): JSX.Element {
  return (
    <div className="absolute right-20 top-16 z-40 w-[340px] rounded-2xl border border-white/[0.08] bg-zinc-950/95 p-3 shadow-2xl shadow-black/40 backdrop-blur-xl">
      <div className="flex items-center justify-between px-2 pb-2">
        <p className="text-xs font-semibold text-zinc-200">Local reminders</p>
        <span className="text-[10px] text-zinc-600">No cloud</span>
      </div>
      <div className="max-h-[420px] space-y-2 overflow-y-auto">
        {reminders.length ? (
          reminders.slice(0, 12).map((reminder) => (
            <article
              key={`${reminder.id}-${reminder.reminderType}`}
              className="rounded-xl border border-white/[0.06] bg-white/[0.03] p-3"
            >
              <div className="flex items-start gap-3">
                <span className={`mt-1 size-2 rounded-full ${severityColor(reminder.severity)}`} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-xs font-medium text-zinc-200">{reminder.title}</p>
                  <p className="mt-1 text-[11px] text-zinc-600">{reminderDueText(reminder)}</p>
                  <p className="mt-1 truncate text-[10px] text-zinc-700">{reminder.source}</p>
                </div>
              </div>
            </article>
          ))
        ) : (
          <div className="rounded-xl border border-dashed border-white/[0.08] px-4 py-8 text-center text-xs text-zinc-600">
            No local reminders detected yet.
          </div>
        )}
      </div>
    </div>
  );
}

function reminderDueText(reminder: LocalReminder): string {
  if (reminder.daysUntil < 0) return `${Math.abs(reminder.daysUntil)} days overdue`;
  if (reminder.daysUntil === 0) return 'Due today';
  return `Due in ${reminder.daysUntil} days · ${reminder.dueDate}`;
}

function severityColor(severity: LocalReminder['severity']): string {
  if (severity === 'overdue') return 'bg-rose-400';
  if (severity === 'critical') return 'bg-amber-300';
  if (severity === 'soon') return 'bg-sky-300';
  return 'bg-zinc-500';
}

function showLocalNotification(reminder: LocalReminder): void {
  if (!('Notification' in window)) return;
  if (Notification.permission === 'granted') {
    new Notification('LifeOS reminder', {
      body: `${reminder.title} · ${reminderDueText(reminder)}`,
    });
    return;
  }
  if (Notification.permission === 'default') {
    void Notification.requestPermission().then((permission) => {
      if (permission === 'granted') {
        new Notification('LifeOS reminder', {
          body: `${reminder.title} · ${reminderDueText(reminder)}`,
        });
      }
    });
  }
}

export function AccentButton({ children }: { children: ReactNode }): JSX.Element {
  return (
    <button className="inline-flex items-center gap-2 rounded-xl bg-zinc-100 px-3.5 py-2 text-xs font-semibold text-zinc-950 transition-colors hover:bg-white">
      {children}
      <ArrowUpRight size={14} strokeWidth={2.3} />
    </button>
  );
}

export function CommandKey(): JSX.Element {
  return <Command size={13} className="text-zinc-500" />;
}
