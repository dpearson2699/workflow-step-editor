// The live capture view (DEC-002, DEC-009, AC-004): compact step rows
// stream in (index, classification dot, auto-title, event time) while
// the prominent Stop Recording banner is the only visible action. The
// full detail pane (screenshots, metadata, editing) activates in draft
// review after Stop. Presentational: rows and callbacks come from the
// record session, never from IPC.

import type { LiveRow } from "../record";
import { formatEventTime } from "../lib/format";

export interface RecordingViewProps {
  rows: readonly LiveRow[];
  /** Disables the banner button once a Stop click latched or issued. */
  stopPending: boolean;
  /** A failed stop command; Stop is re-armed. */
  stopError: string | null;
  onStop: () => void;
}

export function RecordingView(props: RecordingViewProps) {
  return (
    <div className="detail-root">
      <header className="app-header">
        <div className="app-brand">
          <span className="rec-dot" aria-hidden="true">
            ●
          </span>{" "}
          Recording
        </div>
      </header>

      <div className="rec-overlay">
        <span className="rec-dot" aria-hidden="true">
          ●
        </span>
        <span role="status">
          Recording — {props.rows.length}{" "}
          {props.rows.length === 1 ? "step" : "steps"} captured
        </span>
        <button
          type="button"
          className="stop-big"
          disabled={props.stopPending}
          onClick={props.onStop}
        >
          ■ Stop Recording
        </button>
      </div>

      <div className="detail-body">
        <aside className="step-list-pane">
          <ul className="step-list" aria-label="Steps">
            {props.rows.map((row, index) => (
              <li key={row.id} className="step-row">
                {/* Static rows: the Stop banner stays the sole visible
                    action while recording. */}
                <span className="step-row-static">
                  <span className="step-index">{index + 1}</span>
                  <span
                    className={`dot dot-${row.classification}`}
                    aria-hidden="true"
                  />
                  <span className="step-row-title">{row.title}</span>
                  <span className="step-row-time">
                    {formatEventTime(row.ts)}
                  </span>
                </span>
              </li>
            ))}
          </ul>
        </aside>

        <section className="step-detail" aria-label="Recording status">
          {props.stopError !== null && (
            <p className="landing-error" role="alert">
              Could not stop the recording: {props.stopError}
            </p>
          )}
          <p className="step-detail-empty">
            {props.rows.length === 0
              ? "Click and type in any app — captured steps appear here."
              : "Recording. Stop to review, edit, and save the captured steps."}
          </p>
        </section>
      </div>
    </div>
  );
}
