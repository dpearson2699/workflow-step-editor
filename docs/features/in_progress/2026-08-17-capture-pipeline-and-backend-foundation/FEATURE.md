# macOS capture pipeline and backend foundation

## Source / Issue

https://github.com/dpearson2699/workflow-step-editor/issues/12

## Goal

Record real desktop work into durable workflow data. While recording is
active, the Rust backend observes every global click and key-down, attaches a
pre-event screenshot triple and element metadata to each event, parses each
event live into one titled step, and persists schema v1 on the local
filesystem. The review UI capability
(https://github.com/dpearson2699/workflow-step-editor/issues/13) builds on
this bundle's command surface.

## Scope

- Tauri v2 scaffold: Vite + React + TypeScript shell, fixed bundle
  identifier, dev builds signed with the local Apple Development identity so
  TCC grants persist across rebuilds.
- ListenOnly CGEventTap input monitor on a dedicated CFRunLoop thread
  (core-graphics 0.25.0), with runtime `CGEventTapIsEnabled` verification.
- Continuous SCStream per active display. The buffered pre-event frame yields
  all three screenshot-triple artifacts: full screen (the frame), window crop
  (bounds of the hit window via `CGWindowListCopyWindowInfo`), element crop
  (AX frame or fixed-size fallback). Streams restart on display-configuration
  changes. The event's display selects the frame.
- AX element resolution: role, title, frame, source `ax`/`fallback`. Clicks
  hit-test the click point (`AXUIElementCopyElementAtPosition` on the
  system-wide element). Key-downs resolve the focused window of the
  frontmost application and the system focused element
  (`AXFocusedUIElement`); their fallback is a fixed-size crop centered
  inside the focused window's bounds (DEC-008).
- `KeySemantics`: one pure, stateless, unpersisted key-event classifier in
  the recording/parser core. Chord detection uses the semantic non-Shift
  modifier mask only; no timing rules. Auto-titles route through it.
- Live 1:1 parsing: each captured event becomes one step immediately and
  streams to the UI over the capture channel. Auto-titles per the decided
  formats (`Click "OK" — TextEdit`, `Press Cmd+S — TextEdit`).
  Classification defaults: click -> `click`, key-down -> `type`.
- Schema v1 persistence: append-only `events.jsonl` during capture,
  `workflow.json` manifest, `shots/` PNGs; one folder per workflow under
  app-data with readable names; `WorkflowStore` trait (`create`,
  `append_event`, `load`, `save_manifest`, `list`) with the JSON
  implementation.
- Permissions: ordered checks and requests — Input Monitoring is requested
  before any Accessibility API call, plus Screen Recording. Permission
  commands (`check_permissions`, `request_permission(kind)`) for the UI.
- Capture-lifecycle commands and the live capture channel:
  `start_recording(name, channel) -> workflow_id`,
  `stop_recording() -> workflow_id`, `list_workflows()`, `get_workflow(id)`.
- A bare dev-only trigger to start and stop recording.
- Every key-down gets a full screenshot triple through a bounded async queue
  (literal must-have compliance; degradation under typing load is a recorded
  tradeoff only if measured).

## Non-Goals

- The product review UI (issue #13): landing page, permission status strip,
  master–detail review, record flow, save/discard, hard delete, rename,
  reveal-in-Finder, and the step-edit commands (`update_step`,
  `delete_step`, `rename_workflow`, `reveal_workflow`).
- Text-input grouping (issue #14) and keyboard shortcuts (issue #15).
- Synthetic `wait`/`assert` steps, re-parse, manual step insertion.
- SQLite or any non-JSON store.
- Non-macOS capture backends. The platform boundary is the single
  `CapturePipeline` trait.
- Workflow deletion in any form (owned by issue #13's hard-delete record).

## Doc Authority

| Subject | Current authority | Conflict or obligation | Owning update |
| --- | --- | --- | --- |
| Pre-buffered capture and artifact derivation | `docs/adr/0001-pre-buffered-screen-capture.md` | none | none |
| Key-event chord semantics and classifier ownership | `docs/adr/0002-key-event-semantic-classifier.md` | none | none |
| Ubiquitous language (event, step, screenshot triple, workflow, classification, shortcut, hotkey) | `CONTEXT.md` | new terms crystallising during build | `CONTEXT.md` |
| Window crop is a bounds crop, not an isolated window image | ADR 0001 consequences | README must state the caveat | `README.md` (created in this bundle) |
| macOS-only support | capability split, issue #11 | README must state the limitation | `README.md` (created in this bundle) |
| Schema v1 field shapes | issue #7 decision record | none — fields fixed by the record | none |

## Open decision IDs

- none

## Codex Task Roster

- Status: registered
- Entry: PR-01 | implementation | claude-fable-5 medium | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-01 | review | gitnexus-pr-review native, claude-fable-5 high | same predicates
- Entry: PR-02 | implementation | claude-fable-5 high | same predicates
- Entry: PR-02 | review | gitnexus-pr-review native, claude-fable-5 high | same predicates
- Entry: PR-03 | implementation | claude-fable-5 xhigh | same predicates
- Entry: PR-03 | review | gitnexus-pr-review native, claude-fable-5 high | same predicates
- Operational authority is the structured task-authorization projection in
  `state.json`; this list is the readable mirror.

## Durable Sources

- https://github.com/dpearson2699/workflow-step-editor/issues/12 (owning issue)
- https://github.com/dpearson2699/workflow-step-editor/issues/6 (capture architecture, permission UX, proven gate)
- https://github.com/dpearson2699/workflow-step-editor/issues/7 (schema v1, storage, command surface)
- https://github.com/dpearson2699/workflow-step-editor/issues/10 (live 1:1 parsing, auto-titles)
- https://github.com/dpearson2699/workflow-step-editor/issues/9 (shortcut semantics, classifier ownership)
- https://github.com/dpearson2699/workflow-step-editor/issues/11 (capability split, acceptance line)
- https://github.com/dpearson2699/workflow-step-editor/issues/2, /issues/3, /issues/4, /issues/5 (research facts, observed 2026-08-16)
- `docs/adr/0001-pre-buffered-screen-capture.md`
- `docs/adr/0002-key-event-semantic-classifier.md`
- `CONTEXT.md`
- `docs/PROJECT_GOAL.md`
