import {
  Archive,
  CalendarClock,
  FileClock,
  FileText,
  HardDrive,
  ReceiptText,
  ShieldCheck,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { invokeDesktop } from '@/ui/lib/desktop';
import { Panel, SectionHeading } from '@/ui/shared/primitives';

interface UploadedFile {
  id: string;
  fileName: string;
  mediaType: string;
  sizeBytes: number;
  createdAtUnixMs: number;
}

interface DashboardItem {
  uploadId: string;
  title: string;
  subtitle: string;
  date?: string | null;
}

interface CategoryCount {
  category: string;
  count: number;
}

interface DashboardSummary {
  upcomingExpiry: DashboardItem[];
  bills: DashboardItem[];
  insurance: DashboardItem[];
  timeline: DashboardItem[];
  recentUploads: UploadedFile[];
  categories: CategoryCount[];
  storageUsageBytes: number;
}

const emptySummary: DashboardSummary = {
  upcomingExpiry: [],
  bills: [],
  insurance: [],
  timeline: [],
  recentUploads: [],
  categories: [],
  storageUsageBytes: 0,
};

export function DashboardPage(): JSX.Element {
  const [summary, setSummary] = useState<DashboardSummary>(emptySummary);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void invokeDesktop<DashboardSummary>('get_dashboard_summary')
      .then(setSummary)
      .catch(() => setSummary(emptySummary))
      .finally(() => setLoading(false));
  }, []);

  const totalDocuments = summary.categories.reduce((sum, item) => sum + item.count, 0);
  const primaryStats = useMemo(
    () => [
      [CalendarClock, 'Upcoming expiry', `${summary.upcomingExpiry.length}`, 'tracked'],
      [ReceiptText, 'Bills', `${summary.bills.length}`, 'local'],
      [ShieldCheck, 'Insurance', `${summary.insurance.length}`, 'policies'],
      [HardDrive, 'Storage', formatBytes(summary.storageUsageBytes), 'encrypted'],
    ],
    [summary],
  );

  return (
    <div className="page-enter mx-auto max-w-[1320px] p-5 md:p-8 lg:p-10">
      <div className="mb-9 flex items-end justify-between">
        <div>
          <p className="text-sm text-zinc-500">
            {loading ? 'Reading local encrypted data...' : 'A local snapshot from your documents.'}
          </p>
        </div>
        <span className="rounded-xl border border-white/[0.08] px-3 py-2 text-xs text-zinc-400">
          Offline
        </span>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {primaryStats.map(([Icon, label, value, sub]) => {
          const MetricIcon = Icon as typeof CalendarClock;
          return (
            <Panel key={label as string} className="p-5">
              <MetricIcon size={17} className="mb-7 text-zinc-300" />
              <p className="text-xs text-zinc-500">{label as string}</p>
              <div className="mt-1 flex items-end justify-between">
                <p className="text-xl font-semibold text-zinc-100">{value as string}</p>
                <span className="text-[10px] text-zinc-500">{sub as string}</span>
              </div>
            </Panel>
          );
        })}
      </div>

      <div className="mt-8 grid gap-5 xl:grid-cols-[1.2fr_0.9fr]">
        <Panel className="p-5 md:p-6">
          <SectionHeading eyebrow="Timeline" title="Document timeline" />
          <ItemList items={summary.timeline} empty="Upload documents to build a local timeline." />
        </Panel>
        <Panel className="p-5 md:p-6">
          <SectionHeading eyebrow="Expiry" title="Upcoming expiry" />
          <ItemList items={summary.upcomingExpiry} empty="No expiry dates found yet." />
        </Panel>
      </div>

      <div className="mt-5 grid gap-5 xl:grid-cols-3">
        <Panel className="p-5">
          <SectionHeading eyebrow="Bills" title="Bills" />
          <ItemList items={summary.bills} empty="No bills detected yet." compact />
        </Panel>
        <Panel className="p-5">
          <SectionHeading eyebrow="Insurance" title="Insurance" />
          <ItemList
            items={summary.insurance}
            empty="No insurance documents detected yet."
            compact
          />
        </Panel>
        <Panel className="p-5">
          <SectionHeading eyebrow="Categories" title="Document categories" />
          {summary.categories.length ? (
            <div className="space-y-3">
              {summary.categories.slice(0, 7).map((category) => (
                <div key={category.category} className="flex items-center gap-3">
                  <Archive size={14} className="text-zinc-600" />
                  <span className="flex-1 text-[13px] text-zinc-400">{category.category}</span>
                  <span className="text-xs text-zinc-500">{category.count}</span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState text="Categories appear after local AI metadata extraction." />
          )}
          <p className="mt-5 text-[11px] text-zinc-600">{totalDocuments} categorized documents</p>
        </Panel>
      </div>

      <div className="mt-5">
        <Panel className="p-5">
          <SectionHeading eyebrow="Recent" title="Recent uploads" />
          {summary.recentUploads.length ? (
            <div className="divide-y divide-white/[0.06]">
              {summary.recentUploads.map((upload) => (
                <div key={upload.id} className="flex items-center gap-3 py-3">
                  <FileText size={15} className="text-zinc-600" />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-[13px] text-zinc-300">{upload.fileName}</p>
                    <p className="text-[11px] text-zinc-600">{upload.mediaType}</p>
                  </div>
                  <span className="text-[11px] text-zinc-500">{formatBytes(upload.sizeBytes)}</span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState text="No encrypted uploads yet." />
          )}
        </Panel>
      </div>
    </div>
  );
}

function ItemList({
  items,
  empty,
  compact = false,
}: {
  items: DashboardItem[];
  empty: string;
  compact?: boolean;
}): JSX.Element {
  if (!items.length) return <EmptyState text={empty} />;
  return (
    <div className={compact ? 'space-y-3' : 'space-y-4'}>
      {items.map((item) => (
        <div key={`${item.uploadId}-${item.title}`} className="flex items-center gap-3">
          <FileClock size={15} className="shrink-0 text-zinc-600" />
          <div className="min-w-0 flex-1">
            <p className="truncate text-[13px] text-zinc-300">{item.title}</p>
            <p className="truncate text-[11px] text-zinc-600">{item.subtitle}</p>
          </div>
          <span className="text-[11px] text-zinc-500">{item.date ?? 'No date'}</span>
        </div>
      ))}
    </div>
  );
}

function EmptyState({ text }: { text: string }): JSX.Element {
  return (
    <div className="rounded-xl border border-dashed border-white/[0.08] px-4 py-6 text-center text-xs text-zinc-600">
      {text}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}
