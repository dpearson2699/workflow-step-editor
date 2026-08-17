// PROTOTYPE — floating variant switcher. Hidden in production builds.
import { useCallback, useEffect } from 'react';

const NAMES: Record<string, string> = {
  A: 'Timeline feed',
  B: 'Master–detail',
  C: 'Filmstrip',
  D: 'Master–detail + triple',
};

export default function PrototypeSwitcher(props: {
  variants: string[];
  current: string;
  onChange: (v: string) => void;
}) {
  const { variants, current, onChange } = props;
  const cycle = useCallback((d: number) => {
    const i = variants.indexOf(current);
    onChange(variants[(i + d + variants.length) % variants.length]);
  }, [variants, current, onChange]);

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement | null;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) return;
      if (e.key === 'ArrowLeft') cycle(-1);
      if (e.key === 'ArrowRight') cycle(1);
    };
    window.addEventListener('keydown', h);
    return () => window.removeEventListener('keydown', h);
  }, [cycle]);

  if (import.meta.env.PROD) return null;
  return (
    <div className="proto-switcher">
      <button onClick={() => cycle(-1)}>←</button>
      <span>{current} — {NAMES[current] ?? ''}</span>
      <button onClick={() => cycle(1)}>→</button>
    </div>
  );
}
