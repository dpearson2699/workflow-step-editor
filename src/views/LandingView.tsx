// The landing page (DEC-001, AC-002): permission strip, gated Record,
// and one row per saved workflow with thumbnail, name, and
// `date · step count · duration`. Presentational: data and callbacks
// come from the shell, never from IPC.

import type {
  PermissionKind,
  PermissionReport,
  WorkflowSummary,
} from "../api/client";
import { formatSummaryMeta } from "../lib/format";

const PERMISSION_LABELS: Record<PermissionKind, string> = {
  input_monitoring: "Input Monitoring",
  accessibility: "Accessibility",
  screen_recording: "Screen Recording",
};

const PERMISSION_ORDER: PermissionKind[] = [
  "input_monitoring",
  "accessibility",
  "screen_recording",
];

export function allPermissionsGranted(report: PermissionReport | null): boolean {
  return (
    report !== null &&
    PERMISSION_ORDER.every((kind) => report[kind] === "granted")
  );
}

function PermissionStrip(props: {
  permissions: PermissionReport | null;
  onRequestPermission: (kind: PermissionKind) => void;
}) {
  if (props.permissions === null) {
    return null;
  }
  const report = props.permissions;
  return (
    <div className="perm-strip" aria-label="Permissions">
      {PERMISSION_ORDER.map((kind) => {
        const granted = report[kind] === "granted";
        return (
          <button
            key={kind}
            type="button"
            className={granted ? "perm-pill granted" : "perm-pill missing"}
            title={`${PERMISSION_LABELS[kind]}: ${report[kind].split("_").join(" ")}`}
            onClick={() => props.onRequestPermission(kind)}
          >
            {granted ? "✓" : "✗"} {PERMISSION_LABELS[kind]}
          </button>
        );
      })}
    </div>
  );
}

function RowThumbnail(props: { thumbnailUrl: string | null; name: string }) {
  if (props.thumbnailUrl === null) {
    // The labeled placeholder for a damaged event log or a workflow
    // without steps (DEC-006), and for a thumbnail still loading.
    return <div className="row-thumb placeholder">No preview</div>;
  }
  return (
    <img
      className="row-thumb"
      src={props.thumbnailUrl}
      alt={`First step of ${props.name}`}
    />
  );
}

export interface LandingViewProps {
  workflows: WorkflowSummary[] | null;
  error: string | null;
  permissions: PermissionReport | null;
  /** Blob URL per workflow id; missing entries render the placeholder. */
  thumbnails: ReadonlyMap<string, string>;
  onRequestPermission: (kind: PermissionKind) => void;
  onOpenWorkflow: (workflow: WorkflowSummary) => void;
  onRevealWorkflow: (id: string) => void;
  onRecord: () => void;
}

export function LandingView(props: LandingViewProps) {
  const ready = allPermissionsGranted(props.permissions);
  return (
    <div className="landing-root">
      <header className="app-header">
        <div className="app-brand">Workflow Step Editor</div>
        <PermissionStrip
          permissions={props.permissions}
          onRequestPermission={props.onRequestPermission}
        />
      </header>

      <main className="landing-main">
        <div className="record-hero">
          <button
            type="button"
            className="record-button"
            disabled={!ready}
            onClick={props.onRecord}
          >
            ● Record New Workflow
          </button>
          {!ready && (
            <span className="record-hint">
              Grant the missing permission above to enable recording.
            </span>
          )}
        </div>

        <h2 className="landing-heading">
          Workflows
          {props.workflows !== null && (
            <span className="workflow-count">{props.workflows.length}</span>
          )}
        </h2>
        {props.error !== null && (
          <p className="landing-error" role="alert">
            {props.error}
          </p>
        )}
        {props.workflows !== null && props.workflows.length === 0 && (
          <p className="landing-empty">
            No workflows yet. Record one to get started.
          </p>
        )}
        {props.workflows !== null && props.workflows.length > 0 && (
          <ul className="workflow-list" aria-label="Workflows">
            {/* Pinned prototype Home row order: open target, then the
                hover-revealed Reveal control, then the chevron — three
                flow siblings, so Reveal and the chevron never overlap. */}
            {props.workflows.map((workflow) => (
              <li className="workflow-row" key={workflow.id}>
                <button
                  type="button"
                  className="row-open"
                  onClick={() => props.onOpenWorkflow(workflow)}
                >
                  <RowThumbnail
                    thumbnailUrl={props.thumbnails.get(workflow.id) ?? null}
                    name={workflow.name}
                  />
                  <span className="row-info">
                    <span className="row-name">{workflow.name}</span>
                    <span className="row-meta">
                      {formatSummaryMeta(workflow)}
                    </span>
                  </span>
                </button>
                <button
                  type="button"
                  className="row-reveal"
                  title="Reveal in Finder"
                  onClick={() => props.onRevealWorkflow(workflow.id)}
                >
                  ⌘ Reveal
                </button>
                <span className="row-chevron" aria-hidden="true">
                  ›
                </span>
              </li>
            ))}
          </ul>
        )}
      </main>
    </div>
  );
}
