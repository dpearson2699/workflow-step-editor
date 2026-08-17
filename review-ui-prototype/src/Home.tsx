// PROTOTYPE — landing page: the workflow list. Entry point of the app.
// No workflow deletion here (decided scope: folders are user-deletable in
// Finder); each row offers Reveal in Finder instead.
import type { Permissions, WorkflowMeta } from './data';
import { fmtDate, fmtDur, workflows } from './data';

export default function Home(props: {
  openWorkflow: (id: string) => void;
  recording: boolean;
  toggleRecording: () => void;
  perms: Permissions;
  requestPerm: (k: keyof Permissions) => void;
}) {
  const p = props;
  const allOk = p.perms.input && p.perms.accessibility && p.perms.screen;
  return (
    <div className="home-root">
      <header className="home-bar">
        <div className="home-brand">Workflow Step Editor</div>
        <div className="va-perms">
          {(['input', 'accessibility', 'screen'] as const).map(k => (
            <button key={k} className={p.perms[k] ? 'perm ok' : 'perm bad'} onClick={() => p.requestPerm(k)}>
              {p.perms[k] ? '✓' : '✗'} {k === 'input' ? 'Input Monitoring' : k === 'accessibility' ? 'Accessibility' : 'Screen Recording'}
            </button>
          ))}
        </div>
      </header>

      <main className="home-main">
        <div className="home-hero">
          <button className={p.recording ? 'home-rec on' : 'home-rec'} disabled={!allOk} onClick={p.toggleRecording}>
            {p.recording ? '■ Stop Recording' : '● Record New Workflow'}
          </button>
          {p.recording && <span className="home-live">Recording… steps are being captured. Stop to review.</span>}
          {!allOk && <span className="home-hint">Grant the missing permission above to enable recording.</span>}
        </div>

        <h2 className="home-h2">Workflows <span className="va-count">{workflows.length}</span></h2>
        <ul className="home-list">
          {workflows.map((w: WorkflowMeta) => (
            <li className="home-row" key={w.id} onClick={() => p.openWorkflow(w.id)}>
              <img className="home-thumb" src={w.thumb} alt="" />
              <div className="home-info">
                <div className="home-name">{w.name}</div>
                <div className="home-meta">{fmtDate(w.createdAt)} · {w.stepCount} steps · {fmtDur(w.durationSec)}</div>
              </div>
              <button
                className="home-reveal"
                title="Reveal in Finder"
                onClick={e => { e.stopPropagation(); }}
              >⌘ Reveal</button>
              <span className="home-chev">›</span>
            </li>
          ))}
        </ul>
      </main>
    </div>
  );
}
