// The variant D review detail view (DEC-001, AC-003, AC-005): a compact
// text-only step list beside a detail pane where all three screenshots
// stay visible (one large, two labeled click-to-swap thumbnails), with
// auto-saved title/description/classification editing, step deletion,
// the metadata grid, workflow rename in the header, and the single
// saved-workflow Delete… control behind its destructive confirmation.
//
// Steps resolve their raw events by id (`event_ids[0]`), never by array
// index. Edits persist through per-entity serialized autosave queues;
// the workflow's autosave generation is invalidated before deletion so
// stale completions are ignored (see ../lib/autosave.ts).

import { useEffect, useMemo, useRef, useState } from "react";

import type {
  ApiClient,
  Classification,
  ShotVariant,
  Step,
  WorkflowEvent,
} from "../api/client";
import { createAutosave, type Autosave, type SaveStatus } from "../lib/autosave";
import { formatElement, formatEventTime, formatKey } from "../lib/format";

const CLASSIFICATIONS: Classification[] = ["click", "type", "wait", "assert"];
const SHOT_VARIANTS: ShotVariant[] = ["full", "window", "element"];

/** The autosave queue key for one step's edits. */
function stepKey(stepId: string): string {
  return `step:${stepId}`;
}

/** The autosave queue key for the workflow rename. */
const WORKFLOW_KEY = "workflow";

function SaveIndicator(props: {
  status: SaveStatus | undefined;
  onRetry: () => void;
}) {
  const status = props.status;
  if (status === undefined || status.state === "idle") {
    return null;
  }
  if (status.state === "saving") {
    return (
      <span className="save-status" role="status">
        Saving…
      </span>
    );
  }
  return (
    <span className="save-status save-error" role="alert">
      Save failed: {status.message}{" "}
      <button type="button" className="retry-button" onClick={props.onRetry}>
        Retry
      </button>
    </span>
  );
}

function DeleteWorkflowDialog(props: {
  name: string;
  error: string | null;
  deleting: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="modal-scrim">
      <div
        className="modal"
        role="alertdialog"
        aria-modal="true"
        aria-label={`Delete workflow ${props.name}`}
        onKeyDown={(event) => {
          // While the deletion is in flight the dialog must stay open
          // (like the disabled Cancel button), so a failure can still
          // surface inside it (AC-005).
          if (event.key === "Escape" && !props.deleting) {
            props.onCancel();
          }
        }}
      >
        <h3>Delete “{props.name}”?</h3>
        <p>
          This permanently deletes the saved workflow — its recorded
          keystroke data, screenshots, and event log. This cannot be
          undone.
        </p>
        {props.error !== null && (
          <p className="landing-error" role="alert">
            {props.error}
          </p>
        )}
        <div className="modal-actions">
          {/* Cancel is the default action: it takes initial focus, so
              Enter cancels (DEC-003). */}
          <button
            type="button"
            className="modal-cancel"
            autoFocus
            disabled={props.deleting}
            onClick={props.onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="modal-delete"
            disabled={props.deleting}
            onClick={props.onConfirm}
          >
            Delete Workflow
          </button>
        </div>
      </div>
    </div>
  );
}

export interface DetailViewProps {
  api: ApiClient;
  workflowId: string;
  /** Name shown until the manifest loads. */
  initialName: string;
  onBack: () => void;
  /** Called only after a successful backend hard delete; the landing
   *  list refreshes on mount, so the row disappears with it. */
  onDeleted: () => void;
}

export function DetailView(props: DetailViewProps) {
  const { api, workflowId } = props;

  const [name, setName] = useState(props.initialName);
  const [nameError, setNameError] = useState<string | null>(null);
  const [steps, setSteps] = useState<Step[] | null>(null);
  const [events, setEvents] = useState<WorkflowEvent[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [shot, setShot] = useState<ShotVariant>("full");
  const [statuses, setStatuses] = useState<ReadonlyMap<string, SaveStatus>>(
    new Map(),
  );
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [shotUrls, setShotUrls] = useState<ReadonlyMap<string, string>>(
    new Map(),
  );

  // Handlers read the latest steps through this ref so an edit computed
  // between renders never works from a stale list.
  const stepsRef = useRef<Step[]>([]);
  function commitSteps(next: Step[]): void {
    stepsRef.current = next;
    setSteps(next);
  }

  // The selection mirror defeats the stale-closure hazard across the
  // deleteStep await, exactly like stepsRef does for the list.
  const selectedIdRef = useRef<string | null>(null);
  function commitSelected(next: string | null): void {
    selectedIdRef.current = next;
    setSelectedId(next);
  }

  // The latest name and whether the user edited it; the load callback
  // must not clobber an edit typed while the load was in flight.
  const nameRef = useRef(props.initialName);
  const nameEditedRef = useRef(false);

  // Statuses mirror for handlers that must read the current pending
  // set (the deletion-failure recovery below).
  const statusesRef = useRef<ReadonlyMap<string, SaveStatus>>(new Map());
  const autosaveRef = useRef<Autosave | null>(null);
  if (autosaveRef.current === null) {
    autosaveRef.current = createAutosave((key, status) => {
      const next = new Map(statusesRef.current);
      next.set(key, status);
      statusesRef.current = next;
      setStatuses(next);
    });
  }
  const autosave = autosaveRef.current;

  useEffect(() => {
    let stale = false;
    api
      .getWorkflow(workflowId)
      .then((loaded) => {
        if (stale) {
          return;
        }
        // A rename typed while this load was in flight wins over the
        // older manifest name.
        if (!nameEditedRef.current) {
          nameRef.current = loaded.manifest.name;
          setName(loaded.manifest.name);
        }
        stepsRef.current = loaded.manifest.steps;
        setSteps(loaded.manifest.steps);
        setEvents(loaded.events);
        commitSelected(loaded.manifest.steps[0]?.id ?? null);
      })
      .catch((caught) => {
        if (!stale) {
          setLoadError(`Could not load workflow: ${String(caught)}`);
        }
      });
    return () => {
      stale = true;
    };
  }, [api, workflowId]);

  const eventById = useMemo(
    () => new Map(events.map((event) => [event.id, event])),
    [events],
  );

  const selected =
    selectedId === null
      ? null
      : (steps ?? []).find((step) => step.id === selectedId) ?? null;
  // Steps resolve events by id, never by array index.
  const selectedEvent =
    selected === null
      ? null
      : eventById.get(selected.event_ids[0] ?? "") ?? null;
  const selectedEventId = selectedEvent?.id ?? null;

  // The screenshot triple for the selected step, through the scoped
  // backend read (DEC-007). Blob URLs are cached per event/variant and
  // revoked on unmount; a failed read keeps the labeled placeholder.
  const urlCache = useRef(new Map<string, string>());
  const requestedShots = useRef(new Set<string>());
  const disposed = useRef(false);
  useEffect(() => {
    disposed.current = false;
    const cache = urlCache.current;
    const requested = requestedShots.current;
    return () => {
      disposed.current = true;
      for (const url of cache.values()) {
        URL.revokeObjectURL(url);
      }
      cache.clear();
      requested.clear();
    };
  }, []);
  useEffect(() => {
    if (selectedEventId === null) {
      return;
    }
    for (const variant of SHOT_VARIANTS) {
      const key = `${selectedEventId}/${variant}`;
      if (requestedShots.current.has(key)) {
        continue;
      }
      requestedShots.current.add(key);
      api
        .readScreenshot(workflowId, selectedEventId, variant)
        .then((bytes) => {
          const blob = new Blob([bytes as BlobPart], { type: "image/png" });
          const url = URL.createObjectURL(blob);
          if (disposed.current) {
            URL.revokeObjectURL(url);
            return;
          }
          urlCache.current.set(key, url);
          setShotUrls(new Map(urlCache.current));
        })
        .catch(() => {
          // The labeled placeholder stays for this variant. Un-cache
          // the key so re-selecting the step retries a transiently
          // failed read instead of pinning the placeholder for the
          // whole session.
          requestedShots.current.delete(key);
        });
    }
  }, [api, workflowId, selectedEventId]);

  function editStep(
    stepId: string,
    change: Partial<Pick<Step, "title" | "description" | "classification">>,
  ): void {
    const next = stepsRef.current.map((step) =>
      step.id === stepId ? { ...step, ...change } : step,
    );
    commitSteps(next);
    const edited = next.find((step) => step.id === stepId);
    if (edited === undefined) {
      return;
    }
    // The scheduled save always carries the step's latest full editable
    // trio, so coalesced queue entries never drop a field edit.
    const patch = {
      title: edited.title,
      description: edited.description,
      classification: edited.classification,
    };
    autosave.schedule(stepKey(stepId), () =>
      api.updateStep(workflowId, stepId, patch),
    );
  }

  function editName(value: string): void {
    nameRef.current = value;
    nameEditedRef.current = true;
    setName(value);
    const trimmed = value.trim();
    if (trimmed === "") {
      setNameError("Name cannot be empty");
      return;
    }
    setNameError(null);
    autosave.schedule(WORKFLOW_KEY, () =>
      api.renameWorkflow(workflowId, trimmed),
    );
  }

  // In-flight step deletions; a re-entrant click (double-click on ✕)
  // must not send a second request that fails with StepNotFound.
  const deletingSteps = useRef(new Set<string>());
  async function deleteStep(stepId: string): Promise<void> {
    if (deletingSteps.current.has(stepId)) {
      return;
    }
    deletingSteps.current.add(stepId);
    setActionError(null);
    try {
      await api.deleteStep(workflowId, stepId);
    } catch (caught) {
      setActionError(`Could not delete step: ${String(caught)}`);
      return;
    } finally {
      deletingSteps.current.delete(stepId);
    }
    // The deleted step blocks its stale queued updates.
    autosave.block(stepKey(stepId));
    const previous = stepsRef.current;
    const index = previous.findIndex((step) => step.id === stepId);
    const next = previous.filter((step) => step.id !== stepId);
    commitSteps(next);
    // Reconcile against the LATEST selection, not the one captured when
    // the deletion started: the user may have selected another row (or
    // this row) during the request.
    if (selectedIdRef.current === stepId) {
      const fallback = next[Math.min(Math.max(index, 0), next.length - 1)];
      commitSelected(fallback?.id ?? null);
    }
  }

  function confirmDeleteWorkflow(): void {
    setDeleteError(null);
    setDeleting(true);
    // Invalidate the autosave generation before deletion: stale queued
    // completions from the removed workflow are ignored. Keep the
    // pending set so a FAILED deletion can re-arm it below.
    const pending = statusesRef.current;
    autosave.invalidate();
    api.deleteWorkflow(workflowId).then(
      () => {
        props.onDeleted();
      },
      (caught: unknown) => {
        setDeleting(false);
        setDeleteError(String(caught));
        // The workflow still exists, but the invalidation dropped its
        // queued and in-flight saves. Re-send the current local value
        // of every key that had unsettled work so no edit is silently
        // lost and no indicator sticks on "Saving…".
        const cleared = new Map<string, SaveStatus>();
        statusesRef.current = cleared;
        setStatuses(cleared);
        for (const [key, status] of pending) {
          if (status.state === "idle") {
            continue;
          }
          if (key === WORKFLOW_KEY) {
            const trimmed = nameRef.current.trim();
            if (trimmed !== "") {
              autosave.schedule(WORKFLOW_KEY, () =>
                api.renameWorkflow(workflowId, trimmed),
              );
            }
            continue;
          }
          const stepId = key.startsWith("step:")
            ? key.slice("step:".length)
            : null;
          const step =
            stepId === null
              ? undefined
              : stepsRef.current.find((entry) => entry.id === stepId);
          if (stepId !== null && step !== undefined) {
            const patch = {
              title: step.title,
              description: step.description,
              classification: step.classification,
            };
            autosave.schedule(stepKey(stepId), () =>
              api.updateStep(workflowId, stepId, patch),
            );
          }
        }
      },
    );
  }

  return (
    <div className="detail-root">
      <header className="app-header">
        <button type="button" className="back-button" onClick={props.onBack}>
          ‹ Workflows
        </button>
        <input
          className="name-input"
          aria-label="Workflow name"
          value={name}
          onChange={(event) => editName(event.target.value)}
        />
        {nameError !== null && (
          <span className="save-status save-error" role="alert">
            {nameError}
          </span>
        )}
        <SaveIndicator
          status={statuses.get(WORKFLOW_KEY)}
          onRetry={() => autosave.retry(WORKFLOW_KEY)}
        />
        <button
          type="button"
          className="delete-workflow-button"
          onClick={() => {
            setDeleteError(null);
            setConfirmingDelete(true);
          }}
        >
          Delete…
        </button>
      </header>

      {loadError !== null ? (
        <main className="detail-pane">
          <p className="landing-error" role="alert">
            {loadError}
          </p>
        </main>
      ) : (
        <div className="detail-body">
          <aside className="step-list-pane">
            <ul className="step-list" aria-label="Steps">
              {(steps ?? []).map((step, index) => {
                const event = eventById.get(step.event_ids[0] ?? "");
                return (
                  <li
                    key={step.id}
                    className={
                      step.id === selectedId ? "step-row sel" : "step-row"
                    }
                  >
                    <button
                      type="button"
                      className="step-row-open"
                      onClick={() => commitSelected(step.id)}
                    >
                      <span className="step-index">{index + 1}</span>
                      <span
                        className={`dot dot-${step.classification}`}
                        aria-hidden="true"
                      />
                      <span className="step-row-title">{step.title}</span>
                      <span className="step-row-time">
                        {event !== undefined ? formatEventTime(event.ts) : "—"}
                      </span>
                    </button>
                    <button
                      type="button"
                      className="step-delete"
                      aria-label={`Delete step ${index + 1}`}
                      title={`Delete step ${index + 1}`}
                      onClick={() => void deleteStep(step.id)}
                    >
                      ✕
                    </button>
                  </li>
                );
              })}
            </ul>
          </aside>

          <section className="step-detail" aria-label="Step detail">
            {actionError !== null && (
              <p className="landing-error" role="alert">
                {actionError}
              </p>
            )}
            {selected === null ? (
              <p className="step-detail-empty">
                {steps === null ? "Loading…" : "No steps"}
              </p>
            ) : (
              <>
                <div className="shot-wrap">
                  {(() => {
                    const bigUrl =
                      selectedEventId === null
                        ? undefined
                        : shotUrls.get(`${selectedEventId}/${shot}`);
                    return bigUrl !== undefined ? (
                      <img
                        className="shot-big"
                        src={bigUrl}
                        alt={`${shot} screenshot`}
                      />
                    ) : (
                      <div className="shot-big shot-placeholder">
                        No {shot} screenshot
                      </div>
                    );
                  })()}
                  <div className="shot-thumbs">
                    {SHOT_VARIANTS.map((variant) => {
                      const url =
                        selectedEventId === null
                          ? undefined
                          : shotUrls.get(`${selectedEventId}/${variant}`);
                      return (
                        <figure
                          key={variant}
                          className={
                            variant === shot ? "shot-thumb sel" : "shot-thumb"
                          }
                        >
                          <button
                            type="button"
                            className="shot-thumb-open"
                            aria-label={`Show ${variant} screenshot`}
                            onClick={() => setShot(variant)}
                          >
                            {url !== undefined ? (
                              <img src={url} alt={`${variant} thumbnail`} />
                            ) : (
                              <span className="shot-placeholder">
                                No {variant} image
                              </span>
                            )}
                          </button>
                          <figcaption>{variant}</figcaption>
                        </figure>
                      );
                    })}
                  </div>
                </div>

                <div className="step-edit">
                  <div className="edit-row">
                    <select
                      className={`cls cls-${selected.classification}`}
                      aria-label="Classification"
                      value={selected.classification}
                      onChange={(event) =>
                        editStep(selected.id, {
                          classification: event.target.value as Classification,
                        })
                      }
                    >
                      {CLASSIFICATIONS.map((classification) => (
                        <option key={classification} value={classification}>
                          {classification}
                        </option>
                      ))}
                    </select>
                    <input
                      className="title-input"
                      aria-label="Step title"
                      value={selected.title}
                      onChange={(event) =>
                        editStep(selected.id, { title: event.target.value })
                      }
                    />
                  </div>
                  <textarea
                    className="description-input"
                    aria-label="Step description"
                    placeholder="Add a description…"
                    value={selected.description}
                    onChange={(event) =>
                      editStep(selected.id, {
                        description: event.target.value,
                      })
                    }
                  />
                  <SaveIndicator
                    status={statuses.get(stepKey(selected.id))}
                    onRetry={() => autosave.retry(stepKey(selected.id))}
                  />

                  <dl className="meta-grid">
                    <dt>Time</dt>
                    <dd>
                      {selectedEvent !== null
                        ? formatEventTime(selectedEvent.ts)
                        : "—"}
                    </dd>
                    <dt>App</dt>
                    <dd>
                      {selectedEvent?.window != null
                        ? `${selectedEvent.window.app} — ${selectedEvent.window.title}`
                        : "—"}
                    </dd>
                    <dt>Coords</dt>
                    <dd>
                      {selectedEvent !== null
                        ? `(${selectedEvent.pos.x}, ${selectedEvent.pos.y})`
                        : "—"}
                    </dd>
                    <dt>Key</dt>
                    <dd>
                      {selectedEvent?.key != null
                        ? formatKey(selectedEvent.key)
                        : "—"}
                    </dd>
                    <dt>Element</dt>
                    <dd>
                      {selectedEvent !== null
                        ? formatElement(selectedEvent.element)
                        : "—"}
                    </dd>
                  </dl>
                </div>
              </>
            )}
          </section>
        </div>
      )}

      {confirmingDelete && (
        <DeleteWorkflowDialog
          name={name}
          error={deleteError}
          deleting={deleting}
          onCancel={() => setConfirmingDelete(false)}
          onConfirm={confirmDeleteWorkflow}
        />
      )}
    </div>
  );
}
