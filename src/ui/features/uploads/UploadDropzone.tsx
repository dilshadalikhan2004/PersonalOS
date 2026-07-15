import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import { FileCheck2, FileUp, LoaderCircle, ShieldCheck, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { invokeDesktop, isDesktopRuntime, listenDesktop } from '@/ui/lib/desktop';

type UploadState = { id: string; name: string; percent: number; stage: string; error?: string };
type UploadProgress = { uploadId: string; percent: number; stage: string };

const filters = [
  { name: 'Documents and images', extensions: ['pdf', 'png', 'jpg', 'jpeg', 'docx'] },
];

export function UploadDropzone(): JSX.Element {
  const [uploads, setUploads] = useState<UploadState[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [ocrLanguages, setOcrLanguages] = useState('eng');
  const ocrLanguagesRef = useRef('eng');

  useEffect(() => {
    if (!isDesktopRuntime()) return;
    let removeProgress: (() => void) | undefined;
    let removeDrop: (() => void) | undefined;
    void listenDesktop<UploadProgress>('document-upload-progress', ({ payload }) => {
      setUploads((current) =>
        current.map((item) =>
          item.id === payload.uploadId
            ? { ...item, percent: payload.percent, stage: payload.stage }
            : item,
        ),
      );
    }).then((remove) => {
      removeProgress = remove;
    });
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'over') setIsDragging(true);
        if (event.payload.type === 'leave') setIsDragging(false);
        if (event.payload.type === 'drop') {
          setIsDragging(false);
          enqueue(event.payload.paths);
        }
      })
      .then((remove) => {
        removeDrop = remove;
      });
    return () => {
      removeProgress?.();
      removeDrop?.();
    };
  }, []);

  const enqueue = (paths: string[]): void => {
    if (!isDesktopRuntime()) {
      setUploads((current) => [
        ...current,
        {
          id: crypto.randomUUID(),
          name: 'Desktop app required',
          percent: 0,
          stage: 'Open LifeOS in Tauri to securely upload local files.',
          error: 'Desktop app required for local file access.',
        },
      ]);
      return;
    }
    paths.forEach((path) => {
      const id = crypto.randomUUID();
      const name = path.split(/[/\\]/).pop() ?? 'Document';
      setUploads((current) => [
        ...current,
        { id, name, percent: 0, stage: 'Preparing secure upload' },
      ]);
      void invokeDesktop('upload_document', {
        request: { sourcePath: path, uploadId: id, ocrLanguages: ocrLanguagesRef.current },
      }).catch(() => {
        setUploads((current) =>
          current.map((item) =>
            item.id === id ? { ...item, error: 'Could not securely store this file.' } : item,
          ),
        );
      });
    });
  };

  const chooseFiles = async (): Promise<void> => {
    if (!isDesktopRuntime()) {
      enqueue([]);
      return;
    }
    const selected = await open({ title: 'Add to LifeOS', multiple: true, filters });
    if (selected) enqueue(Array.isArray(selected) ? selected : [selected]);
  };

  return (
    <section className="mb-9">
      <div className="mb-3 flex items-center justify-between gap-4">
        <p className="text-[11px] text-zinc-600">OCR runs entirely on this device.</p>
        <label className="flex items-center gap-2 text-[11px] text-zinc-500">
          Text language
          <select
            value={ocrLanguages}
            onChange={(event) => {
              ocrLanguagesRef.current = event.target.value;
              setOcrLanguages(event.target.value);
            }}
            className="rounded-lg border border-white/[0.08] bg-white/[0.04] px-2 py-1 text-[11px] text-zinc-300 outline-none"
          >
            <option value="eng">English</option>
            <option value="eng+hin">English + Hindi</option>
            <option value="eng+spa">English + Spanish</option>
          </select>
        </label>
      </div>
      <button
        onClick={() => void chooseFiles()}
        className={`group relative flex min-h-40 w-full flex-col items-center justify-center overflow-hidden rounded-2xl border border-dashed p-6 text-center transition-all ${isDragging ? 'scale-[1.01] border-violet-300 bg-violet-400/[0.12]' : 'border-white/[0.12] bg-white/[0.025] hover:border-violet-400/50 hover:bg-violet-400/[0.045]'}`}
      >
        <div className="absolute inset-x-12 bottom-0 h-px bg-gradient-to-r from-transparent via-violet-400/60 to-transparent" />
        <span className="grid size-11 place-items-center rounded-2xl bg-violet-400/10 text-violet-200 ring-1 ring-violet-300/20">
          <FileUp size={20} />
        </span>
        <p className="mt-3 text-sm font-medium text-zinc-200">
          Drop files here, or <span className="text-violet-300">browse your device</span>
        </p>
        <p className="mt-1.5 text-[11px] text-zinc-600">PDF, PNG, JPEG, or DOCX · up to 50 MB</p>
        <span className="mt-3 inline-flex items-center gap-1.5 text-[10px] text-zinc-600">
          <ShieldCheck size={12} /> Encrypted before storage
        </span>
      </button>
      {uploads.length ? (
        <div className="mt-3 space-y-2">
          {uploads.map((upload) => (
            <div
              key={upload.id}
              className="rounded-xl border border-white/[0.07] bg-white/[0.025] px-3.5 py-3"
            >
              <div className="flex items-center gap-3">
                <span className="grid size-8 place-items-center rounded-lg bg-white/[0.05] text-zinc-400">
                  {upload.error ? (
                    <X size={15} />
                  ) : upload.percent === 100 ? (
                    <FileCheck2 size={15} className="text-emerald-400" />
                  ) : (
                    <LoaderCircle size={15} className="animate-spin" />
                  )}
                </span>
                <div className="min-w-0 flex-1 text-left">
                  <p className="truncate text-xs font-medium text-zinc-300">{upload.name}</p>
                  <p
                    className={`mt-0.5 text-[10px] ${upload.error ? 'text-rose-300' : 'text-zinc-600'}`}
                  >
                    {upload.error ?? upload.stage}
                  </p>
                </div>
                <span className="text-[11px] text-zinc-500">
                  {upload.error ? 'Failed' : `${upload.percent}%`}
                </span>
              </div>
              {!upload.error && upload.percent < 100 ? (
                <div className="mt-2 h-1 overflow-hidden rounded-full bg-white/[0.06]">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-violet-500 to-fuchsia-300 transition-all duration-300"
                    style={{ width: `${upload.percent}%` }}
                  />
                </div>
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </section>
  );
}
