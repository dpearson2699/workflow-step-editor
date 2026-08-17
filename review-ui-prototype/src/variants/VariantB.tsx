// PROTOTYPE — Variant B: "Master–detail". Compact step list on the left,
// one selected step inspected deeply on the right.
import { useState } from 'react';
import type { VariantProps } from '../data';
import { CLASSIFICATIONS, fmtTime } from '../data';

export default function VariantB(p: VariantProps) {
  const [selId, setSelId] = useState(p.steps[0]?.id ?? '');
  const [shot, setShot] = useState<'full' | 'window' | 'element'>('full');
  const sel = p.steps.find(s => s.id === selId) ?? p.steps[0];

  return (
    <div className="vb-root">
      <aside className="vb-side">
        <div className="vb-side-head">
          <div className="vb-brand">Approve invoice</div>
          <button
            className={p.recording ? 'rec-btn on' : 'rec-btn'}
            disabled={!p.perms.input || !p.perms.accessibility || !p.perms.screen}
            onClick={p.toggleRecording}
          >
            {p.recording ? '■ Stop' : '● Record'}
          </button>
        </div>
        <ul className="vb-list">
          {p.steps.map((s, i) => (
            <li
              key={s.id}
              className={sel && s.id === sel.id ? 'vb-row sel' : 'vb-row'}
              onClick={() => setSelId(s.id)}
            >
              <span className="vb-i">{i + 1}</span>
              <span className={`dot dot-${s.classification}`} />
              <span className="vb-row-title">{s.title}</span>
              <span className="vb-row-time">{fmtTime(s.ts).slice(0, 8)}</span>
              <button className="del subtle" title="Delete step" onClick={e => { e.stopPropagation(); p.deleteStep(s.id); }}>✕</button>
            </li>
          ))}
        </ul>
        <div className="vb-perm-footer">
          {(['input', 'accessibility', 'screen'] as const).map(k => (
            <button key={k} className={p.perms[k] ? 'perm ok' : 'perm bad'} onClick={() => p.requestPerm(k)}>
              {p.perms[k] ? '✓' : '✗'} {k === 'input' ? 'Input' : k === 'accessibility' ? 'AX' : 'Screen'}
            </button>
          ))}
        </div>
      </aside>

      {sel ? (
        <section className="vb-detail">
          <div className="vb-shotbar">
            {(['full', 'window', 'element'] as const).map(k => (
              <button key={k} className={shot === k ? 'seg sel' : 'seg'} onClick={() => setShot(k)}>{k}</button>
            ))}
          </div>
          <img className="vb-big" src={sel.shots[shot]} alt={shot} />
          <div className="vb-edit">
            <div className="vb-edit-row">
              <select
                className={`cls cls-${sel.classification}`}
                value={sel.classification}
                onChange={e => p.updateStep(sel.id, { classification: e.target.value as never })}
              >
                {CLASSIFICATIONS.map(c => <option key={c} value={c}>{c}</option>)}
              </select>
              <input className="vb-title" value={sel.title} onChange={e => p.updateStep(sel.id, { title: e.target.value })} />
            </div>
            <textarea
              className="vb-desc"
              placeholder="Add a description…"
              value={sel.description}
              onChange={e => p.updateStep(sel.id, { description: e.target.value })}
            />
            <dl className="vb-meta">
              <dt>Time</dt><dd>{fmtTime(sel.ts)}</dd>
              <dt>App</dt><dd>{sel.app} — {sel.windowTitle}</dd>
              <dt>Coords</dt><dd>{sel.pos ? `(${sel.pos.x}, ${sel.pos.y})` : '—'}</dd>
              <dt>Key</dt><dd>{sel.key ? sel.key.modifiers.concat(sel.key.chars).join('+') : '—'}</dd>
              <dt>Element</dt><dd>{sel.element ? `${sel.element.role}${sel.element.title ? ` "${sel.element.title}"` : ''} · ${sel.element.source}` : '—'}</dd>
            </dl>
          </div>
        </section>
      ) : <section className="vb-detail empty">No steps</section>}
    </div>
  );
}
