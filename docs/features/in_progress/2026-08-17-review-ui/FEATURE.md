# Review UI

## Source / Issue

https://github.com/dpearson2699/workflow-step-editor/issues/13

## Goal

Build the Workflow Step Editor review UI on the proven capture pipeline so
the app is demoable end to end: a workflow-list landing page, the
master–detail review view with the always-visible screenshot triple, the
record → draft review → save/discard flow, saved-workflow hard deletion, and
workflow rename.

## Scope

- Landing page: one row per saved workflow (thumbnail, name,
  `date · step count · duration`), permission status strip in the header, a
  prominent Record button disabled until all three permissions pass with an
  explanatory hint, hover Reveal-in-Finder per row, navigation into the
  detail view and `‹ Workflows` back.
- Detail view (pinned variant D direction): compact text-only step list
  (index, classification dot, auto-title, time, hover-delete); detail pane
  with one large screenshot plus the other two as labeled click-to-swap
  thumbnails; title and description editing; classification dropdown
  (`click`/`type`/`wait`/`assert`); step deletion; metadata grid (time,
  app/window, coordinates, key, element with source).
- Record flow: Record → live capture view where the only visible action is a
  prominent Stop Recording banner while step rows stream in over the capture
  channel; Stop → draft review (`draft` badge, full editing, Discard with
  confirmation, Save…); Save → naming dialog with a pre-selected timestamp
  default. Naming is the save ceremony; capture writes are already
  crash-safe on disk, and edits auto-save.
- Saved-workflow hard deletion per `docs/adr/0003`: a non-primary Delete…
  control, a destructive confirmation naming the keystroke data with Cancel
  as default, backend-performed deletion by workflow id validated inside the
  workflow root, UI updates only after backend success, an already-missing
  directory treated as deleted, no tombstone/trash/audit copy. Draft Discard
  shares the same removal primitive.
- Workflow rename.
- Backend surface the foundation did not ship: `update_step`, `delete_step`,
  `rename_workflow`, `reveal_workflow`, `delete_workflow`; workflow-summary
  data the landing rows need (step count, duration, thumbnail); a path for
  the webview to display screenshot files from app storage.

## Non-Goals

- Text-input grouping (issue #14) and keyboard shortcuts (issue #15).
- Re-parse, automatic wait detection, synthetic wait/assert steps, and
  manual step insertion (issue #10 boundary).
- Any trash, restore, audit, or purge lifecycle for deletion; forensic
  erasure claims (`docs/adr/0003`).
- Schema v2 or SQLite; schema v1 persistence stays as decided in issue #7.

## Doc Authority

| Subject | Current authority | Conflict or obligation | Owning update |
| --- | --- | --- | --- |
| Review-UI direction, record flow | issue #8 resolution; `prototype/map-1-8` @ `e5e2652` | none | none |
| Data model and command surface | issue #7 resolution | deletion clause superseded, noted on #7 | none |
| Hard-delete semantics | issue #8 adopted decision record | ADR obligation | `docs/adr/0003-hard-delete-for-saved-workflows.md` (landed with this Discuss) |
| Step parsing and auto-titles | issue #10 resolution | none | none |
| Capability boundary and acceptance | issues #11 and #13 | none | none |
| Draft terminology | `CONTEXT.md` | new term | `CONTEXT.md` (landed with this Discuss) |

## Open decision IDs

- none

## Codex Task Roster

- Status: registered
- Entry: PR-01 | implementation | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-01 | review | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-02 | implementation | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-02 | review | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-03 | implementation | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-03 | review | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision

## UI Acceptance Policy

- Policy: final_pr_design_gate
- Final UI slice: PR-03

## Durable Sources

- `docs/PROJECT_GOAL.md`
- `CONTEXT.md`
- `docs/adr/0001-pre-buffered-screen-capture.md`
- `docs/adr/0002-key-event-semantic-classifier.md`
- `docs/adr/0003-hard-delete-for-saved-workflows.md`
- `src-tauri/src/commands/mod.rs`, `src-tauri/src/recording/` (shipped
  command surface, coordinator, store)
- `src/App.tsx` (dev-only capture trigger this capability replaces)
- `prototype/map-1-8` @ `e5e265262809ca2961967a65800d45efa5795b1a`
  (`review-ui-prototype/`, run `npm install && npm run dev`, open
  `/?variant=D&view=home`)
