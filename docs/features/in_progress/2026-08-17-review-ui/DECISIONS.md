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
  leaves draft mode; Discard deletes the folder. A crash or quit before
  Save leaves the workflow listable under its default name with every
  captured event and screenshot retained. The crash guarantee covers raw
  capture data: parsed steps persist to the manifest at stop/fail
  finalization, so a crash during active recording can leave a folder
  whose manifest holds fewer steps than its event log. Re-parse recovery
  stays out of scope.
- Rationale: Issue #8 records that naming is the save ceremony because
  capture writes (events and screenshots) are crash-safe; the worker
  writes manifest steps only at finalization
  (`src-tauri/src/recording/coordinator.rs`, `run_worker`). Guaranteeing
  reviewable steps through a mid-recording crash would require incremental
  manifest persistence or re-parse — both outside the accepted scope.
- Rejected alternatives: A persisted `draft` manifest field; incremental
  manifest persistence; startup re-parse recovery.
- Canonical docs: issue #8 resolution; issue #7 decision 5; `CONTEXT.md`
  (Draft).

## DEC-006: Landing summary presentation semantics

- Status: accepted
- Decision: Duration is the span from the first to the last event
  timestamp, zero when fewer than two events exist, and omitted when the
  event log is unreadable. The thumbnail is the first step's window crop,
  with a labeled placeholder fallback. The landing list sorts newest
  first. A workflow with a readable manifest but a damaged or missing
  event log stays listed with placeholder duration and thumbnail. Step
  count comes from manifest steps, not event count.
- Rationale: Matches the pinned prototype rows (`date · step count ·
  duration`, newest first) and keeps a damaged event log from hiding an
  otherwise readable workflow. Step-count-from-manifest preserves the
  step/event distinction for future grouping.
- Rejected alternatives: Duration from `created_at` to stop time;
  frontend-computed summaries via per-row `get_workflow`.
- Canonical docs: none.

## DEC-007: Screenshots reach the webview through a scoped backend read

- Status: accepted
- Decision: The frontend requests screenshots by workflow id, event id,
  and an allowlisted variant (full, window, element); the backend derives
  the canonical shot path, validates confinement, and returns PNG bytes.
  No asset-protocol scope over the workflow root is granted, and no
  filesystem path crosses IPC in either direction.
- Rationale: A broad asset scope would make `events.jsonl` — recorded
  keystrokes — addressable from the webview, and the app currently runs
  with a null CSP. The backend read reuses the store's existing
  confinement validation.
- Rejected alternatives: Tauri asset protocol scoped to the workflow
  root; frontend-supplied file paths.
- Canonical docs: none.

## DEC-008: Manifest mutations serialize behind one coordinator lock

- Status: accepted
- Decision: All manifest load–mutate–save operations (`update_step`,
  `delete_step`, `rename_workflow`) and worker finalization serialize
  behind one coordinator-owned mutation lock, without holding the
  recording-phase lock during filesystem I/O. Mutations and deletion
  targeting the active or stopping workflow are rejected; mutating a
  different saved workflow while recording stays allowed.
- Rationale: Atomic file replacement prevents torn manifests but not lost
  updates — two concurrent load–mutate–save calls can silently drop each
  other's changes, and stop finalization writes the whole manifest.
- Rejected alternatives: Fire-and-forget saves with atomic rename only;
  a per-workflow lock registry (unneeded for one local user).
- Canonical docs: none.

## DEC-009: Live recording rows and failed-recording destination

- Status: accepted
- Decision: During recording the step list streams compact rows — index,
  classification dot, auto-title, and event time — from the live channel;
  the channel's step payload gains the event timestamp as an additive
  transient field. The full detail pane (screenshot triple, metadata,
  editing) activates in draft review after Stop. A recording that ends in
  failure lands in draft review with a visible error banner whenever its
  workflow still loads; the banner states the recording failed and may be
  incomplete, and the user reviews or discards what was committed. A
  workflow that no longer loads surfaces the error on the landing page
  instead.
- Rationale: Matches the pinned recording view (rows stream in; Stop is
  the only visible action) without per-event screenshot loads during
  capture. The current live payload carries no timestamp, so the row
  format needs the additive field. Failed recordings keep the
  committed-data guarantee visible instead of discarding review access.
- Rejected alternatives: Rendering the full triple live per event;
  returning to the landing page on failure.
- Canonical docs: none.
