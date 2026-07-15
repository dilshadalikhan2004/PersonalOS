import type { ReactNode } from 'react';

export function SectionHeading({
  eyebrow,
  title,
  action,
}: {
  eyebrow?: string;
  title: string;
  action?: ReactNode;
}): JSX.Element {
  return (
    <div className="mb-4 flex items-end justify-between gap-4">
      <div>
        {eyebrow ? (
          <p className="mb-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-zinc-600">
            {eyebrow}
          </p>
        ) : null}
        <h2 className="text-[17px] font-semibold tracking-[-0.035em] text-zinc-100">{title}</h2>
      </div>
      {action}
    </div>
  );
}

export function Panel({
  children,
  className = '',
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return (
    <section className={`rounded-2xl border border-white/[0.07] bg-white/[0.035] ${className}`}>
      {children}
    </section>
  );
}

export function Dots(): JSX.Element {
  return <span className="tracking-[0.16em] text-zinc-600">•••</span>;
}
