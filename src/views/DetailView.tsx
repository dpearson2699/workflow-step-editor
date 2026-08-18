// The variant D review detail view (DEC-001, AC-003, AC-005): a compact
// text-only step list beside a detail pane where all three screenshots
// stay visible (one large, two labeled click-to-swap thumbnails), with
// auto-saved title/description/classification editing, step deletion,
// the metadata grid, workflow rename in the header, and the single
// saved-workflow Delete… control behind its destructive confirmation.
//
// Draft review (DEC-002, DEC-005, AC-004): mounted with the `draft`
// prop after a recording stops, the same view opens in draft mode — a
// `draft` badge beside the manifest name, full editing, Discard behind
// a confirmation, and Save…, which opens the naming dialog pre-selected
// with the manifest's default timestamp name. Draft is UI-session
// state: it exits only when the rename command succeeds, and Discard
// removes the folder through the shared hard-delete primitive. A failed
// recording carries its error banner over the same draft view.
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

/**
 * The destructive confirmation shared by saved-workflow deletion and
 * draft Discard. Cancel is the default action (DEC-003), and the dialog
 * stays open while the removal runs so a failure surfaces inside it.
 */
function ConfirmRemovalDialog(props: {
  ariaLabel: string;
  title: string;
  body: string;
  confirmLabel: string;
  error: string | null;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="modal-scrim">
      <div
        className="modal"
        role="alertdialog"
        aria-modal="true"
        aria-label={props.ariaLabel}
        onKeyDown={(event) => {
          // While the removal is in flight the dialog must stay open
          // (like the disabled Cancel button), so a failure can still
          // surface inside it (AC-004, AC-005).
          if (event.key === "Escape" && !props.busy) {
            props.onCancel();
          }
        }}
      >
        <h3>{props.title}</h3>
        <p>{props.body}</p>
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
            disabled={props.busy}
            onClick={props.onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="modal-delete"
            disabled={props.busy}
            onClick={props.onConfirm}
          >
            {props.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * The save ceremony (DEC-002): naming the draft. The input pre-selects
 * the manifest's existing default timestamp name; a failed rename keeps
 * the dialog and its error visible, and the draft state stands.
 */
function SaveRecordingDialog(props: {
  defaultName: string;
  stepCount: number;
  error: string | null;
  saving: boolean;
  onCancel: () => void;
  onSave: (name: string) => void;
}) {
  const [name, setName] = useState(props.defaultName);
  const invalid = name.trim() === "";
  return (
    <div className="modal-scrim">
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label="Save recording"
        onKeyDown={(event) => {
          if (event.key === "Escape" && !props.saving) {
            props.onCancel();
          }
        }}
      >
        <h3>Save recording</h3>
        <p>
          {props.stepCount} {props.stepCount === 1 ? "step" : "steps"}{" "}
          captured. Events are already on disk — naming finishes the
          save, and edits save automatically.
        </p>
        <input
          className="modal-name-input"
          aria-label="Recording name"
          autoFocus
          value={name}
          onChange={(event) => setName(event.target.value)}
          onFocus={(event) => event.target.select()}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !invalid && !props.saving) {
              props.onSave(name);
            }
          }}
        />
        {invalid && (
          <p className="landing-error" role="alert">
            Name cannot be empty
          </p>
        )}
        {props.error !== null && (
          <p className="landing-error" role="alert">
            {props.error}
          </p>
        )}
        <div className="modal-actions">
          <button
            type="button"
            className="modal-cancel"
            disabled={props.saving}
            onClick={props.onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="modal-save"
            disabled={invalid || props.saving}
            onClick={() => props.onSave(name)}
          >
            Save
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
   *  list refreshes on mount, so the row disappears with it. Draft
   *  Discard shares this exit. */
  onDeleted: () => void;
  /**
   * Mounts the view in draft review (DEC-002, DEC-005): draft badge,
   * Discard, and Save… replace the rename input and Delete…. `failure`
   * carries a failed recording's error into the banner. Draft is
   * UI-session state; it exits only on rename-command success.
   */
  draft?: { failure: string | null };
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
  // Draft review is UI-session state (DEC-005): it starts from the
  // mount-time prop and exits only on rename-command success.
  const [draftActive, setDraftActive] = useState(props.draft !== undefined);
  const [confirmingDiscard, setConfirmingDiscard] = useState(false);
  const [naming, setNaming] = useState(false);
  const [savingDraft, setSavingDraft] = useState(false);
  const [saveDraftError, setSaveDraftError] = useState<string | null>(null);
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
      // Drop any failed rename retry: it holds a previous value that no
      // longer matches the (invalid) input.
      autosave.discard(WORKFLOW_KEY);
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

  /**
   * The naming save ceremony (DEC-002): renames through
   * `rename_workflow` and exits draft mode only when the command
   * succeeds. A failure keeps the dialog, its error, and draft state.
   */
  async function saveDraft(rawName: string): Promise<void> {
    const trimmed = rawName.trim();
    if (trimmed === "") {
      return;
    }
    setSavingDraft(true);
    setSaveDraftError(null);
    try {
      await api.renameWorkflow(workflowId, trimmed);
    } catch (caught) {
      setSaveDraftError(String(caught));
      setSavingDraft(false);
      return;
    }
    setSavingDraft(false);
    nameRef.current = trimmed;
    nameEditedRef.current = true;
    setName(trimmed);
    setNaming(false);
    setDraftActive(false);
  }

  return (
    <div className="detail-root">
      <header className="app-header">
        <button type="button" className="back-button" onClick={props.onBack}>
          ‹ Workflows
        </button>
        {draftActive ? (
          <>
            {/* Naming is the save ceremony (DEC-002): in draft mode the
                manifest's default name is static and the dialog names
                the workflow, so the rename autosave stays out of play. */}
            <span className="draft-name">
              {name !== "" ? name : "New Recording"}
            </span>
            <span className="draft-badge">draft</span>
            <span className="header-spacer" />
            <div className="draft-actions">
              <button
                type="button"
                className="discard-button"
                onClick={() => {
                  setDeleteError(null);
                  setConfirmingDiscard(true);
                }}
              >
                Discard
              </button>
              <button
                type="button"
                className="save-draft-button"
                disabled={steps === null}
                onClick={() => {
                  setSaveDraftError(null);
                  setNaming(true);
                }}
              >
                Save…
              </button>
            </div>
          </>
        ) : (
          <>
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
            {/* While the input is invalid the validation error replaces
                the save indicator, so a stale Retry can never resend a
                previous name over a blank input. */}
            {nameError === null && (
              <SaveIndicator
                status={statuses.get(WORKFLOW_KEY)}
                onRetry={() => autosave.retry(WORKFLOW_KEY)}
              />
            )}
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
          </>
        )}
      </header>

      {/* Not gated on draftActive: a genuine failed terminal can settle
          after the user already saved the draft (the supersedable
          fallback race), and the incomplete recording must not present
          as an ordinary saved workflow. */}
      {props.draft?.failure != null && (
        <p className="record-failed-banner" role="alert">
          Recording failed and may be incomplete: {props.draft.failure}.
          The steps below were captured before the failure — review them
          before relying on this workflow.
        </p>
      )}

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
        <ConfirmRemovalDialog
          ariaLabel={`Delete workflow ${name}`}
          title={`Delete “${name}”?`}
          body={
            "This permanently deletes the saved workflow — its recorded " +
            "keystroke data, screenshots, and event log. This cannot be " +
            "undone."
          }
          confirmLabel="Delete Workflow"
          error={deleteError}
          busy={deleting}
          onCancel={() => setConfirmingDelete(false)}
          onConfirm={confirmDeleteWorkflow}
        />
      )}

      {confirmingDiscard && (
        <ConfirmRemovalDialog
          ariaLabel="Discard recording"
          title="Discard this recording?"
          body={
            "This permanently deletes the draft — its recorded keystroke " +
            "data, screenshots, and event log. This cannot be undone."
          }
          confirmLabel="Discard Recording"
          error={deleteError}
          busy={deleting}
          onCancel={() => setConfirmingDiscard(false)}
          onConfirm={confirmDeleteWorkflow}
        />
      )}

      {naming && steps !== null && (
        <SaveRecordingDialog
          defaultName={name}
          stepCount={steps.length}
          error={saveDraftError}
          saving={savingDraft}
          onCancel={() => setNaming(false)}
          onSave={(newName) => void saveDraft(newName)}
        />
      )}
    </div>
  );
}
