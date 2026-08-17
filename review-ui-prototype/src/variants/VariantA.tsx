// PROTOTYPE — Variant A: "Timeline feed". Chronological vertical feed; every
// step is a full card with an inline thumbnail strip and inline editing.
import { useState } from 'react';
import type { VariantProps } from '../data';
import { CLASSIFICATIONS, fmtTime } from '../data';

export default function VariantA(p: VariantProps) {
  const [zoom, setZoom] = useState<Record<string, 'full' | 'window' | 'element' | null>>({});
  return (
    <div className="va-root">
      <header className="va-header">
        <div className="va-brand">Workflow Step Editor</div>
        <div className="va-perms">
          {(['input', 'accessibility', 'screen'] as const).map(k => (
            <button key={k} className={p.perms[k] ? 'perm ok' : 'perm bad'} onClick={() => p.requestPerm(k)}>
              {p.perms[k] ? '✓' : '✗'} {k === 'input' ? 'Input Monitoring' : k === 'accessibility' ? 'Accessibility' : 'Screen Recording'}
            </button>
          ))}
        </div>
        <button
          className={p.recording ? 'rec-btn on' : 'rec-btn'}
          disabled={!p.perms.input || !p.perms.accessibility || !p.perms.screen}
          onClick={p.toggleRecording}
        >
          {p.recording ? '■ Stop Recording' : '● Record'}
        </button>
      </header>

      <main className="va-feed">
        <h2 className="va-wf-title">Approve invoice <span className="va-count">{p.steps.length} steps</span></h2>
        {p.steps.map((s, i) => (
          <article className="va-card" key={s.id}>
            <div className="va-rail">
              <div className="va-index">{i + 1}</div>
              <div className="va-time">{fmtTime(s.ts)}</div>
            </div>
            <div className="va-body">
              <div className="va-row1">
                <select
                  className={`cls cls-${s.classification}`}
                  value={s.classification}
                  onChange={e => p.updateStep(s.id, { classification: e.target.value as never })}
                >
                  {CLASSIFICATIONS.map(c => <option key={c} value={c}>{c}</option>)}
                </select>
                <input
                  className="va-title"
                  value={s.title}
                  onChange={e => p.updateStep(s.id, { title: e.target.value })}
                />
                <button className="del" title="Delete step" onClick={() => p.deleteStep(s.id)}>🗑</button>
              </div>
              <textarea
                className="va-desc"
                placeholder="Add a description…"
                value={s.description}
                onChange={e => p.updateStep(s.id, { description: e.target.value })}
              />
              <div className="va-meta">
                <span>{s.app} — {s.windowTitle}</span>
                {s.pos && <span>({s.pos.x}, {s.pos.y})</span>}
                {s.element && <span>{s.element.role}{s.element.title ? ` "${s.element.title}"` : ''} · {s.element.source}</span>}
                {s.key && <span>key: {s.key.modifiers.concat(s.key.chars).join('+')}</span>}
              </div>
              <div className="va-shots">
                {(['full', 'window', 'element'] as const).map(kind => (
                  <img
                    key={kind}
                    src={s.shots[kind]}
                    className={zoom[s.id] === kind ? 'va-thumb sel' : 'va-thumb'}
                    alt={kind}
                    onClick={() => setZoom(z => ({ ...z, [s.id]: z[s.id] === kind ? null : kind }))}
                  />
                ))}
              </div>
              {zoom[s.id] && <img className="va-zoom" src={s.shots[zoom[s.id]!]} alt="zoom" />}
            </div>
          </article>
        ))}
      </main>
    </div>
  );
}
