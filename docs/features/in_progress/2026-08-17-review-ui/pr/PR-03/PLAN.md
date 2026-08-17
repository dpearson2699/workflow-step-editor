# PR-03 Plan: Record flow, draft save/discard, and hard deletion

## Outcome

Record on the landing page enters a live capture view whose only visible
action is a prominent Stop Recording banner while step rows stream in; Stop
enters draft review (`draft` badge, full editing, Discard with confirmation,
Save…); Save opens the naming dialog with a pre-selected timestamp default
and names the workflow; saved workflows gain a Delete… control behind the
decided destructive confirmation, performed by the backend inside the
workflow root. The app is demoable end to end.

## Scope and Ownership

- Behavior: Add the store's folder-removal primitive (resolve the id inside
  the workflow root, refuse escapes and substituted links, remove the
  directory, treat an already-missing directory as success) and the
  `delete_workflow(id)` command over it. Build the record flow per the
  pinned prototype and DEC-002/DEC-005: recording mode driven by the live
  channel (rows stream in; Stop Recording banner is the sole visible
  action), draft review as UI-session state (badge, Discard confirmation,
  Save…), the naming dialog (timestamp default pre-selected; saving renames
  via `rename_workflow` and exits draft mode), Discard deleting the draft
  folder through the shared primitive and returning to the landing page.
  Add the saved-workflow Delete… control with the destructive confirmation
  naming the keystroke data and Cancel as default; the row disappears only
  after backend success, failures surface, and a missing directory counts
  as deleted with a list refresh (`docs/adr/0003`).
- Owned paths: `src`, `src-tauri/src`

## Slice Cohesion

- Primary outcome: A recording can be captured, reviewed as a draft, saved
  with a name or discarded, and a saved workflow can be hard-deleted.
- Primary execution flow: Record -> live rows stream over the channel ->
  Stop -> draft review -> Save names the workflow (or Discard/Delete…
  removes its folder through the one shared primitive) -> landing list
  reflects stored state.
- Owning observable seam: The review UI over the production
  `start_recording` channel, `stop_recording`, `rename_workflow`, and the
  store's single folder-removal primitive.
- Primary acceptance criterion: AC-004
- Regression guards: AC-002, AC-003
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Draft Discard and saved-workflow Delete… are one removal
  primitive by decided contract (DEC-003); the record flow's Discard is
  that primitive's first consumer, so the flow and the deletion surface are
  inseparable at the folder-removal seam.
- Path-count warning: none

## Non-Goals

- Any trash, restore, audit, or purge lifecycle; forensic erasure claims.
- Text-input grouping, keyboard shortcuts, re-parse, auto wait detection.
- Changes to capture, parsing, or schema v1.

## Dependencies

- Slice dependencies: PR-02
- Wave: 3
- Execution mode: serial

## Acceptance Coverage

- AC-004: This slice implements and proves the record flow — live
  streaming, Stop banner as sole action, draft badge, Discard confirmation,
  Save naming dialog with timestamp default, folder removal on Discard.
- AC-005: This slice implements and proves the validated backend hard
  delete and its confirmation flow.

## Verification

- Rust: `cargo test` in `src-tauri` covering the removal primitive's
  root-confinement, link refusal, missing-directory success, and the
  `delete_workflow` command.
- Frontend: `npx vitest run` component tests for the record-flow state
  transitions, the naming dialog default, and both confirmation flows.
- Build: `npm run build` passes.
- Independent command: `npm run build && (cd src-tauri && cargo test)`
- UI gate: final_human_required
- Automated UI acceptance: AC-004
- UI proof target: stop-recording-banner
- Final UI slice: PR-03
- Final design acceptance: AC-001

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: asynchronous live-channel state; one critical
  predicate (destructive deletion of user data with root-confinement
  validation)
- Binding: codex_task_request
