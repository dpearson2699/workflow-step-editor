# PR-02 Plan: Detail review view with editing

## Outcome

Opening a workflow from the landing list shows the full variant D review
view: the compact text-only step list beside a detail pane where all three
screenshots stay visible (one large, two labeled click-to-swap thumbnails),
with title and description editing, the four-value classification dropdown,
step deletion, the metadata grid, and workflow rename — every edit persisted
through production commands without an explicit save.

## Scope and Ownership

- Behavior: Add commands `update_step(workflow_id, step_id, patch)` (title,
  description, classification), `delete_step(workflow_id, step_id)` (removes
  the step entry; its events stay in the log), and
  `rename_workflow(id, name)` (manifest name only; the folder never
  renames), implemented through the existing load -> mutate ->
  `save_manifest` path on the coordinator. Build the detail pane per the
  pinned prototype: step list (index, classification dot, auto-title, time,
  hover-delete), screenshot-triple viewer with click-to-swap, editable
  title/description with auto-save, classification dropdown, metadata grid
  (time, app/window, coordinates, key, element with `ax`/`fallback`
  source) from the loaded events, and rename in the header.
- Owned paths: `src`, `src-tauri/src`

## Slice Cohesion

- Primary outcome: A saved workflow is reviewable and editable in the detail
  view with edits persisted.
- Primary execution flow: Open workflow -> `get_workflow` -> step list and
  triple render -> edit title/description/classification, delete a step, or
  rename -> mutation command -> manifest saved -> UI reflects stored state.
- Owning observable seam: The review UI over the production `get_workflow`,
  `update_step`, `delete_step`, and `rename_workflow` commands and the
  stored manifest they mutate.
- Primary acceptance criterion: AC-003
- Regression guards: AC-002
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: The editing surface and its mutation commands are one
  behavior observed at one seam — the review UI persisting through the
  manifest; neither is shippable or provable without the other.
- Path-count warning: none

## Non-Goals

- The record flow, draft states, save/discard, and any deletion of whole
  workflows (PR-03).
- Landing-page changes beyond wiring the existing navigation into the real
  pane.
- Re-parse, grouping, or synthetic steps.

## Dependencies

- Slice dependencies: PR-01
- Wave: 2
- Execution mode: serial

## Acceptance Coverage

- AC-003: This slice implements and proves the entire detail-view
  invariant — triple always visible with click-to-swap, persisted edits,
  classification dropdown, step deletion with events retained, metadata
  grid, and rename.

## Verification

- Rust: `cargo test` in `src-tauri` covering the three mutation commands,
  including step deletion retaining events and rename leaving the folder
  name unchanged.
- Frontend: `npx vitest run` component tests for edit persistence flows,
  click-to-swap, and the metadata grid rendering both element sources.
- Build: `npm run build` passes.
- Independent command: `npm run build && (cd src-tauri && cargo test)`
- UI gate: snapshot_required_human_deferred
- Automated UI acceptance: AC-003
- UI proof target: step-detail-pane
- Final UI slice: PR-03
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: stateful editing flows over persistence; multi-file
  backend/frontend integration
- Binding: codex_task_request
