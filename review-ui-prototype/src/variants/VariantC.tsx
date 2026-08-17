// PROTOTYPE — Variant C: "Filmstrip". Horizontal storyboard of full-screen
// frames on top; a large stage with a side edit panel below.
import { useState } from 'react';
import type { VariantProps } from '../data';
import { CLASSIFICATIONS, fmtTime } from '../data';

export default function VariantC(p: VariantProps) {
  const [selId, setSelId] = useState(p.steps[0]?.id ?? '');
  const [shot, setShot] = useState<'full' | 'window' | 'element'>('window');
  const [permsOpen, setPermsOpen] = useState(false);
  const sel = p.steps.find(s => s.id === selId) ?? p.steps[0];
  const allOk = p.perms.input && p.perms.accessibility && p.perms.screen;

  return (
    <div className="vc-root">
      <header className="vc-top">
        <div className="vc-brand">Approve invoice <span className="va-count">{p.steps.length} steps</span></div>
        <button className="vc-perm-toggle" onClick={() => setPermsOpen(o => !o)}>
          {allOk ? '✓ permissions' : '⚠ permissions'}
        </button>
        <button
          className={p.recording ? 'rec-fab on' : 'rec-fab'}
          disabled={!allOk}
          onClick={p.toggleRecording}
        >
          {p.recording ? '■' : '●'}
        </button>
      </header>
      {permsOpen && (
        <div className="vc-perm-drawer">
          {(['input', 'accessibility', 'screen'] as const).map(k => (
            <button key={k} className={p.perms[k] ? 'perm ok' : 'perm bad'} onClick={() => p.requestPerm(k)}>
              {p.perms[k] ? '✓' : '✗'} {k === 'input' ? 'Input Monitoring' : k === 'accessibility' ? 'Accessibility' : 'Screen Recording'}
            </button>
          ))}
        </div>
      )}

      <div className="vc-strip">
        {p.steps.map((s, i) => (
          <figure
            key={s.id}
            className={sel && s.id === sel.id ? 'vc-frame sel' : 'vc-frame'}
            onClick={() => setSelId(s.id)}
          >
            <img src={s.shots.full} alt={s.title} />
            <figcaption>
              <span className={`dot dot-${s.classification}`} /> {i + 1}
            </figcaption>
          </figure>
        ))}
      </div>

      {sel && (
        <div className="vc-stage">
          <div className="vc-view">
            <div className="vb-shotbar">
              {(['full', 'window', 'element'] as const).map(k => (
                <button key={k} className={shot === k ? 'seg sel' : 'seg'} onClick={() => setShot(k)}>{k}</button>
              ))}
              <span className="vc-time">{fmtTime(sel.ts)}</span>
            </div>
            <img className="vc-big" src={sel.shots[shot]} alt={shot} />
          </div>
          <aside className="vc-panel">
            <select
              className={`cls cls-${sel.classification}`}
              value={sel.classification}
              onChange={e => p.updateStep(sel.id, { classification: e.target.value as never })}
            >
              {CLASSIFICATIONS.map(c => <option key={c} value={c}>{c}</option>)}
            </select>
            <input className="vc-title" value={sel.title} onChange={e => p.updateStep(sel.id, { title: e.target.value })} />
            <textarea
              className="vc-desc"
              placeholder="Add a description…"
              value={sel.description}
              onChange={e => p.updateStep(sel.id, { description: e.target.value })}
            />
            <div className="va-meta vc-meta">
              <span>{sel.app} — {sel.windowTitle}</span>
              {sel.pos && <span>({sel.pos.x}, {sel.pos.y})</span>}
              {sel.key && <span>key: {sel.key.modifiers.concat(sel.key.chars).join('+')}</span>}
              {sel.element && <span>{sel.element.role}{sel.element.title ? ` "${sel.element.title}"` : ''} · {sel.element.source}</span>}
            </div>
            <button className="del wide" onClick={() => p.deleteStep(sel.id)}>Delete step</button>
          </aside>
        </div>
      )}
    </div>
  );
}
