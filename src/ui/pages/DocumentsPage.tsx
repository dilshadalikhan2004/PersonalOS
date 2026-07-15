import {
  Archive,
  FileImage,
  FileText,
  Folder,
  Grid2X2,
  List,
  RefreshCw,
  Search,
  SlidersHorizontal,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { UploadDropzone } from '@/ui/features/uploads/UploadDropzone';
import { invokeDesktop } from '@/ui/lib/desktop';
import { Panel, SectionHeading } from '@/ui/shared/primitives';

interface LibraryDocument {
  id: string;
  fileName: string;
  mediaType: string;
  sizeBytes: number;
  createdAtUnixMs: number;
  title?: string | null;
  documentType?: string | null;
  expiryDate?: string | null;
  documentDate?: string | null;
}

export function DocumentsPage(): JSX.Element {
  const [documents, setDocuments] = useState<LibraryDocument[]>([]);
  const [query, setQuery] = useState('');
  const [filter, setFilter] = useState('all');
  const [loading, setLoading] = useState(true);
  const [view, setView] = useState<'grid' | 'list'>('grid');

  const loadDocuments = async (): Promise<void> => {
    setLoading(true);
    await invokeDesktop<LibraryDocument[]>('list_library_documents')
      .then(setDocuments)
      .catch(() => setDocuments([]))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    void loadDocuments();
  }, []);

  const categories = useMemo(() => {
    const counts = new Map<string, number>();
    documents.forEach((document) => {
      const category = document.documentType || kindFromMedia(document.mediaType);
      counts.set(category, (counts.get(category) ?? 0) + 1);
    });
    return [...counts.entries()].sort((left, right) => right[1] - left[1]);
  }, [documents]);

  const filtered = documents.filter((document) => {
    const haystack = `${document.fileName} ${document.title ?? ''} ${document.documentType ?? ''}`
      .trim()
      .toLowerCase();
    const matchesQuery = !query.trim() || haystack.includes(query.trim().toLowerCase());
    const category = document.documentType || kindFromMedia(document.mediaType);
    const matchesFilter = filter === 'all' || category === filter;
    return matchesQuery && matchesFilter;
  });

  return (
    <div className="page-enter mx-auto max-w-[1320px] p-5 md:p-8 lg:p-10">
      <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <p className="text-sm text-zinc-500">Everything you have chosen to keep locally.</p>
        </div>
        <button
          onClick={() => void loadDocuments()}
          className="inline-flex items-center justify-center gap-2 rounded-xl bg-zinc-100 px-3.5 py-2 text-xs font-semibold text-zinc-950"
        >
          <RefreshCw size={15} /> Refresh
        </button>
      </div>

      <div className="my-8 flex flex-col gap-3 sm:flex-row">
        <label className="flex h-10 flex-1 items-center gap-2 rounded-xl border border-white/[0.08] bg-black/10 px-3 text-zinc-600">
          <Search size={15} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search your encrypted library"
            className="w-full bg-transparent text-xs text-zinc-300 outline-none placeholder:text-zinc-600"
          />
        </label>
        <div className="flex gap-2">
          <label className="toolbar-button">
            <SlidersHorizontal size={15} />
            <select
              value={filter}
              onChange={(event) => setFilter(event.target.value)}
              className="bg-transparent text-xs outline-none"
            >
              <option value="all">All</option>
              {categories.map(([category]) => (
                <option key={category} value={category}>
                  {category}
                </option>
              ))}
            </select>
          </label>
          <button
            aria-label="Grid view"
            onClick={() => setView('grid')}
            className={`toolbar-icon ${view === 'grid' ? 'text-zinc-200' : 'text-zinc-600'}`}
          >
            <Grid2X2 size={15} />
          </button>
          <button
            aria-label="List view"
            onClick={() => setView('list')}
            className={`toolbar-icon ${view === 'list' ? 'text-zinc-200' : 'text-zinc-600'}`}
          >
            <List size={15} />
          </button>
        </div>
      </div>

      <UploadDropzone />

      <section className="mb-10">
        <SectionHeading eyebrow="Collections" title="Browse by type" />
        {categories.length ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {categories.slice(0, 6).map(([title, count]) => (
              <button
                key={title}
                onClick={() => setFilter(title)}
                className="flex items-center gap-3 rounded-2xl border border-white/[0.07] bg-white/[0.03] p-4 text-left hover:bg-white/[0.06]"
              >
                <span className="grid size-9 place-items-center rounded-xl bg-white/[0.05] text-zinc-300">
                  <Folder size={18} />
                </span>
                <span>
                  <span className="block text-[13px] font-medium text-zinc-200">{title}</span>
                  <span className="mt-0.5 block text-[11px] text-zinc-600">
                    {count} document{count === 1 ? '' : 's'}
                  </span>
                </span>
              </button>
            ))}
          </div>
        ) : (
          <Empty
            text={loading ? 'Loading local library...' : 'Upload documents to create collections.'}
          />
        )}
      </section>

      <SectionHeading
        eyebrow="Library"
        title="Your documents"
        action={<span className="text-xs text-zinc-600">{filtered.length} shown</span>}
      />
      {filtered.length ? (
        view === 'grid' ? (
          <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
            {filtered.map((document) => (
              <DocumentCard key={document.id} document={document} />
            ))}
          </div>
        ) : (
          <Panel className="divide-y divide-white/[0.06]">
            {filtered.map((document) => (
              <DocumentRow key={document.id} document={document} />
            ))}
          </Panel>
        )
      ) : (
        <Empty text={loading ? 'Loading local library...' : 'No documents match this view.'} />
      )}
    </div>
  );
}

function DocumentCard({ document }: { document: LibraryDocument }): JSX.Element {
  const Icon = document.mediaType.startsWith('image/') ? FileImage : FileText;
  return (
    <article className="group rounded-2xl border border-white/[0.07] bg-white/[0.025] p-3 transition-colors hover:bg-white/[0.06]">
      <div className="relative h-28 rounded-xl bg-gradient-to-br from-zinc-700/40 via-zinc-900/20 to-black/20 p-3">
        <Icon size={18} className="text-white/70" />
        <span className="absolute bottom-3 left-3 rounded-full bg-black/20 px-2 py-1 text-[10px] text-white/60">
          {document.documentType || kindFromMedia(document.mediaType)}
        </span>
      </div>
      <div className="px-1 pb-1 pt-3">
        <p className="truncate text-[13px] font-medium text-zinc-200">
          {document.title || document.fileName}
        </p>
        <p className="mt-1 text-[11px] text-zinc-600">
          {formatBytes(document.sizeBytes)} · {formatDate(document.createdAtUnixMs)}
        </p>
      </div>
    </article>
  );
}

function DocumentRow({ document }: { document: LibraryDocument }): JSX.Element {
  return (
    <div className="flex items-center gap-3 px-5 py-4">
      <Archive size={15} className="text-zinc-600" />
      <div className="min-w-0 flex-1">
        <p className="truncate text-[13px] font-medium text-zinc-200">
          {document.title || document.fileName}
        </p>
        <p className="text-[11px] text-zinc-600">
          {document.documentType || kindFromMedia(document.mediaType)} ·{' '}
          {formatBytes(document.sizeBytes)}
        </p>
      </div>
      <span className="text-[11px] text-zinc-500">
        {document.documentDate || formatDate(document.createdAtUnixMs)}
      </span>
    </div>
  );
}

function Empty({ text }: { text: string }): JSX.Element {
  return (
    <div className="rounded-2xl border border-dashed border-white/[0.08] bg-white/[0.02] px-5 py-10 text-center text-sm text-zinc-600">
      {text}
    </div>
  );
}

function kindFromMedia(mediaType: string): string {
  if (mediaType.includes('pdf')) return 'PDF';
  if (mediaType.includes('image')) return 'Image';
  if (mediaType.includes('word')) return 'DOCX';
  return 'Document';
}

function formatDate(value: number): string {
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(value));
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}
