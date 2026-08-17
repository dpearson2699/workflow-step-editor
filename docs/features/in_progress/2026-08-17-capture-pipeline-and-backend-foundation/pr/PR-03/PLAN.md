# PR-03 Plan: macOS capture pipeline, recording lifecycle, and dev trigger

## Outcome

The live macOS capture pipeline behind the `CapturePipeline` trait: the
ListenOnly event tap, pre-buffered per-display screen capture, AX element
resolution, per-event screenshot triples, live 1:1 step emission over the
capture channel, schema v1 persistence through the PR-02 store, the
recording-lifecycle commands, and a bare dev-only trigger. After this slice
the proven gate (AC-001) can run.

## Scope and Ownership

- Behavior: `CapturePipeline` trait as the platform boundary with the macOS
  implementation. ListenOnly CGEventTap on a dedicated CFRunLoop thread with
  runtime `CGEventTapIsEnabled` verification. Continuous SCStream per active
  display holding the latest frame; streams restart on display-configuration
  changes; the event's display selects the frame. Per-event triple derived
  from the buffered pre-event frame: full frame, window bounds crop
  (`CGWindowListCopyWindowInfo` hit test), element crop. Clicks resolve the
  element at the click point; key-downs resolve the frontmost application's
  focused window and `AXFocusedUIElement` (DEC-008); fallback is a
  fixed-size crop (about 300x200 pt, clamped, scaled by the display scale)
  with `source: "fallback"`. PNG encoding through a bounded async queue.
  Fail-stop on tap disable, stream failure, or lost permission (DEC-007).
  Commands: `start_recording(name, channel) -> workflow_id` (refuses unless
  all three permissions pass; one active recording), `stop_recording() ->
  workflow_id`, `list_workflows()`, `get_workflow(id)`. Live parsed steps
  stream over the Tauri channel. A bare dev-only trigger in the shell page
  starts and stops recording.
- Owned paths: `src-tauri/src/capture/`, `src-tauri/src/commands/`,
  `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`,
  `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src/`

## Slice Cohesion

- Primary outcome: A recording session captures real global input into
  persisted schema v1 workflow data with per-event screenshot triples.
- Primary execution flow: `start_recording` -> tap event -> buffered frame +
  window/element resolution -> triple PNGs -> parsed step over the channel ->
  store append -> `stop_recording` manifest save.
- Owning observable seam: The recording-lifecycle Tauri commands over the
  `CapturePipeline` trait.
- Primary acceptance criterion: AC-005
- Regression guards: AC-002, AC-003, AC-004 (consumed, not modified)
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Tap, buffer, resolution, queue, and lifecycle commands are
  one live capture flow with no independently observable half: a tap without
  triples violates the per-event invariant, and triples without the
  lifecycle commands have no production consumer. Success is observable only
  at the command seam driving the whole flow.
- Path-count warning: none

## Non-Goals

- The product review UI, permission status strip, and step-edit commands
  (review-UI capability, issue #13).
- Burst grouping, keyboard shortcuts, synthetic `wait`/`assert` steps.
- Automatic recovery from mid-recording failure beyond fail-stop (DEC-007).
- Windows or Linux `CapturePipeline` implementations.

## Dependencies

- Slice dependencies: PR-02
- Wave: 3
- Execution mode: serial

## Acceptance Coverage

- AC-005: permission gating and single-active-recording tests against a
  faked permission source at the lifecycle seam; the real TCC prompts and
  ordered requests are exercised in the feature-owned proven-gate manual
  run.
- After this slice merges, the feature-owned proven gate can run on the
  signed build.

## Verification

- `cargo test` for lifecycle gating, single-active-recording, crop
  arithmetic (element/window rect clamping and scaling), and channel
  delivery of parsed steps through a collecting fake seam.
- Real-capture smoke check during implementation: a short recording writes
  `events.jsonl`, `workflow.json`, and three PNGs per event on the
  implementer's machine (dev-signed build); evidence recorded in the
  receipt. The user-facing proven gate itself is AC-001 after merge.
- `cargo clippy` clean for touched code; `npm run tauri build` succeeds.
- Verify live crate versions and API shapes before locking: core-graphics
  0.25.0, screencapturekit 8.0.1, accessibility-sys 0.2.0 or axuielement
  0.9.1 (hit-test API shape is a named research gap — verify locally).
- Independent command: `cargo test --manifest-path src-tauri/Cargo.toml`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 xhigh
- Selection predicates: interacting concurrency and state invariants (tap
  thread, stream buffer, bounded async queue, single-active-recording);
  cross-module coordination (tap -> buffer -> AX -> parser -> store ->
  channel)
- Binding: codex_task_request

## Parallelization Assessment

- No same-wave pair: every wave in this bundle holds exactly one slice and
  runs serially, so no pair record applies.
