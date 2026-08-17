# PR-02 Plan: Detail review, editing, and saved-workflow deletion

## Outcome

Opening a workflow from the landing list shows the full variant D review
view: the compact text-only step list beside a detail pane where all three
screenshots stay visible (one large, two labeled click-to-swap thumbnails),
with title and description editing, the four-value classification dropdown,
step deletion, the metadata grid, and workflow rename — every edit persisted
through production commands without an explicit save. A saved workflow can
also be hard-deleted through the decided destructive confirmation, performed
by the backend inside the workflow root.

## Scope and Ownership

- Behavior: Add commands `update_step(workflow_id, step_id, patch)` (a
  transient patch whose only optional fields are title, description, and
  classification), `delete_step(workflow_id, step_id)` (removes the step
  entry; its events and screenshots stay byte-identical), and
  `rename_workflow(id, name)` (manifest name only, trimmed and non-empty;
  the folder and id never change), implemented through the coordinator's
  load -> mutate -> `save_manifest` path serialized behind the DEC-008
  mutation lock. The lock also wraps worker finalization, and mutations
  targeting the active or stopping workflow are rejected while edits to
  other workflows stay allowed; the recording-phase lock is never held
  during filesystem I/O. Build the detail pane per the pinned prototype:
  step list (index, classification dot, auto-title, time, hover-delete),
  screenshot-triple viewer with click-to-swap and labeled placeholders for
  missing images, editable title/description with auto-save, classification
  dropdown, metadata grid (time, app/window, coordinates, key, element
  with `ax`/`fallback` source), and rename in the header. Steps resolve
  their events by id (`event_ids[0]`), never by array index. Frontend
  autosave uses per-entity serialized queues with visible error recovery;
  a completed older request never overwrites a newer local edit, and a
  deleted step blocks stale queued updates. Add the store's folder-removal
  primitive and the `delete_workflow(id)` command over it: validate the id
  and resolve the directory inside the workflow root with the store's
  existing confinement checks (symlinked or non-directory targets are
  refused), then remove the whole directory with one
  `std::fs::remove_dir_all` call, whose post-CVE-2022-21658
  implementation deletes descriptor-relative and does not follow or
  traverse symbolic links — no per-child path traversal reopens the
  substitution window. Success requires directory absence: the primitive
  caches the manifest bytes before removal, and on a removal error
  reports success only when the directory is gone; otherwise it restores
  `workflow.json` from the cache when missing and reports failure, so the
  workflow stays listed per `docs/adr/0003` and no sensitive remnant is
  ever hidden behind a success result. An already-missing directory or
  root counts as success; no tombstone, trash, or audit artifact is
  written; deleting the active or stopping workflow is rejected
  (DEC-008). Add one saved-workflow Delete…
  control (non-primary, in the detail header) with the destructive
  confirmation naming the keystroke data and Cancel as the default
  action; the row disappears only after backend success, failures
  surface, and a missing directory counts as deleted with a list refresh.
  Before invoking deletion the frontend invalidates the workflow's
  autosave generation so stale queued completions from the removed
  workflow are ignored.
- Owned paths: `src`, `src-tauri/src`

## Slice Cohesion

- Primary outcome: A saved workflow is reviewable and editable in the detail
  view with edits persisted.
- Primary execution flow: Open workflow -> `get_workflow` -> step list and
  triple render -> edit title/description/classification, delete a step, or
  rename -> mutation command -> manifest saved -> UI reflects stored state.
- Owning observable seam: The saved-workflow management surface over the
  production `get_workflow`, `update_step`, `delete_step`,
  `rename_workflow`, and `delete_workflow` commands and the stored
  workflow folder they mutate or remove.
- Primary acceptance criterion: AC-003
- Regression guards: AC-002
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Editing, rename, step deletion, and whole-workflow
  deletion are the management actions of one saved workflow, observed at
  one seam — the review UI mutating or removing the same stored folder
  authority, serialized behind the same DEC-008 lock and guard. Deletion
  is the terminal management action of the surface this slice builds;
  splitting it would ship a management view whose decided Delete… control
  is absent.
- Path-count warning: none

## Non-Goals

- The record flow, draft states, save/discard, and the live capture view
  (PR-03; draft Discard consumes this slice's `delete_workflow`).
- Landing-page changes beyond wiring the existing navigation into the real
  pane and the Delete… affordance.
- Re-parse, grouping, or synthetic steps.
- Trash, restore, audit, or purge lifecycle; forensic erasure claims.

## Dependencies

- Slice dependencies: PR-01
- Wave: 2
- Execution mode: serial

## Acceptance Coverage

- AC-003: This slice implements and proves the entire detail-view
  invariant — triple always visible with click-to-swap, persisted edits,
  classification dropdown, step deletion with events retained, metadata
  grid, and rename.
- AC-005: This slice implements and proves the validated backend hard
  delete and its confirmation flow.

## Verification

- Rust: `cargo test` in `src-tauri` covering the three mutation commands:
  patches change only supplied fields; classification rejects values
  outside the four-value enum; unknown workflow/step writes nothing;
  concurrent title and description patches lose neither change; rename
  changes the manifest name but not id or folder; step deletion leaves
  `events.jsonl` and shot files byte-identical; mutation of the active or
  stopping workflow is rejected while another saved workflow stays
  editable; stop finalization cannot overwrite a completed edit. Removal
  primitive: missing directory or root is success; no tombstone is
  written; symlinked and non-directory targets are refused; success is
  reported only when the directory is absent; an injected failure that
  leaves any remnant restores the manifest when missing, reports
  failure, and keeps the workflow listed; a retried delete then
  completes; deletion of the active or stopping workflow is rejected;
  deleting one workflow while another records is safe.
- Frontend: `npx vitest run` component tests for edit persistence flows,
  out-of-order autosave completions, failed-autosave recovery, deletion
  of the selected step, stale-update suppression after step and workflow
  deletion, click-to-swap, the metadata grid rendering both element
  sources, and the Delete… confirmation (Cancel default, keystroke-data
  language, row removal only after backend success, failure keeps the row
  and surfaces the error).
- Build: `npm run build` passes.
- Independent command: `npm run build && (cd src-tauri && cargo test)`
- UI gate: snapshot_required_human_deferred
- Automated UI acceptance: AC-003
- UI proof target: step-detail-pane
- Final UI slice: PR-03
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: stateful editing flows over persistence; one
  critical predicate (destructive deletion of user data with
  root-confinement validation); multi-file backend/frontend integration
- Binding: Claude task adapter request
