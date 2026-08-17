# Decisions

## DEC-001: Review UI is variant D plus a workflow-list landing page

- Status: accepted
- Decision: The review UI implements the pinned variant D direction — a
  compact text-only step list (index, classification dot, auto-title, time,
  hover-delete) beside a detail pane where all three screenshots stay
  visible, one large and two as labeled click-to-swap thumbnails — plus the
  landing page (workflow rows, permission strip, gated Record with hint,
  hover Reveal-in-Finder, chevron navigation).
- Rationale: User-selected through the issue #8 prototype iteration
  (A/B/C → D); row thumbnails explicitly rejected for width cost.
- Rejected alternatives: Variant A (timeline feed), B (plain
  master–detail), C (filmstrip), row thumbnails.
- Canonical docs: issue #8 resolution; `prototype/map-1-8` @ `e5e2652`.

## DEC-002: Record flow — live capture view, draft review, save ceremony

- Status: accepted
- Decision: Record opens the detail view in recording mode where the sole
  visible action is a prominent Stop Recording banner while step rows
  stream in live. Stop enters draft review (`draft` badge, full editing,
  Discard with confirmation, Save…). Save opens the naming dialog with a
  pre-selected timestamp default; saving names the workflow. Edits
  auto-save.
- Rationale: User-directed refinement in issue #8; capture writes are
  crash-safe so naming is the only save ceremony needed.
- Rejected alternatives: A dialog interrupting before review; explicit
  save-per-edit.
- Canonical docs: issue #8 resolution.

## DEC-003: Saved-workflow deletion is a confirmed hard delete

- Status: accepted
- Decision: Saved workflows get a non-primary Delete… control behind a
  destructive confirmation that names the keystroke data with Cancel as
  default. The backend deletes by workflow id, validated inside the
  workflow root; the UI updates only after backend success; an
  already-missing directory counts as deleted; no tombstone, trash, audit
  copy, or restore lifecycle. Draft Discard shares the removal primitive.
- Rationale: Retained keystroke data has no audit consumer in a local
  single-user product; privacy semantics of Delete win. Supersedes the
  issue #7 "no deletion in the MVP UI" clause.
- Rejected alternatives: Front-end soft delete with audit retention; no
  deletion.
- Canonical docs: `docs/adr/0003-hard-delete-for-saved-workflows.md`;
  issue #8 adopted record.

## DEC-004: Backend delta — five commands plus summary extension

- Status: accepted
- Decision: This capability adds `update_step(workflow_id, step_id, patch)`,
  `delete_step(workflow_id, step_id)`, `rename_workflow(id, name)`,
  `reveal_workflow(id)`, and `delete_workflow(id)`; extends the
  workflow-list summary with step count, duration, and thumbnail; and adds
  a scoped path for the webview to display stored screenshots. Schema v1
  and the `WorkflowStore` seam stay unchanged in shape; store additions are
  additive.
- Rationale: Issue #7 decision 4 names the command surface; issue #8 adds
  deletion; issue #13 authorizes the backend delta the UI needs.
- Rejected alternatives: Frontend-computed summaries via `get_workflow`
  per row (re-reads every event log on the landing page for no gain).
- Canonical docs: issues #7, #8, #13.

## DEC-005: Draft is a UI-session state, not a storage state

- Status: accepted
- Decision: No draft flag enters schema v1. The workflow folder exists from
  recording start under the timestamp default name; Save renames it and
  leaves draft mode; Discard deletes the folder; a crash or quit before
  Save leaves the workflow listable under its default name with nothing
  lost.
- Rationale: Issue #8 records that naming is the save ceremony because
  capture writes are already crash-safe; a storage flag would add a schema
  change with no user-visible benefit.
- Rejected alternatives: A persisted `draft` manifest field with a
  reconciliation pass at startup.
- Canonical docs: issue #8 resolution; issue #7 decision 5; `CONTEXT.md`
  (Draft).
