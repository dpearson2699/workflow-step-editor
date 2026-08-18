# PR-01 Plan: Per-kind frame selection with a post-event key-down frame

## Outcome

A key-down step's screenshot triple is cut from the first buffered frame
whose display timestamp is after the event (bounded wait on the capture
worker, pre-event fallback per DEC-002), a click step's triple is unchanged,
and ADR-0001, the README, and the `frame_age_ms` doc comments record the
per-kind rule.

## Scope and Ownership

- Behavior: Implement DEC-001 and DEC-002 in the capture pipeline without
  changing its architecture:
  - `FrameBroker` gains one pure post-event query: the oldest retained
    frame for a display whose `ts_ns` is strictly after the event
    timestamp (`previous` before `newest`), or `None` when no retained
    frame is later. `eligible` and `snapshot` keep their pre-event
    semantics; the tap callback keeps pinning the pre-event snapshot for
    every event (`pipeline.rs::start_tap` unchanged in behavior).
  - The capture worker (`worker.rs::run_capture_worker`) receives the
    shared broker handle and a clock. For a `KeyDown` job it resolves
    metadata, selects the display with the existing DEC-008 rule
    (extract that selection from `packets.rs::build_packet` so the
    worker and `build_packet` share one function), then waits for the
    broker's post-event frame on that display until
    `job.ts_ns + POST_EVENT_FRAME_DEADLINE_NS` (250 ms). The deadline is
    anchored to the event timestamp: a job that reaches the worker after
    its deadline does not wait. If a post-event frame exists it replaces
    the pinned frame for that display; otherwise the pinned pre-event
    frame is used. Clicks never wait. The wait polls the broker at a
    short interval (about 5 ms) or uses a condition variable; either is
    acceptable, and it never blocks the tap or reorders jobs.
  - `packets.rs::build_packet` consumes the selected frame (through a
    snapshot override or a frame parameter, implementer's choice) and
    keeps every crop, fallback, and null-window rule unchanged.
    `frame_age_ms` keeps the existing saturating computation, so a
    post-event frame reports `0`.
  - The clock used for the deadline is injectable at the worker seam
    (default: `hostclock::host_now_ns`) so worker tests are
    deterministic; production wiring in `pipeline.rs` passes the host
    clock and the broker.
  - Doc comments on `CaptureMeta.frame_age_ms` (`domain/schema.rs`) and
    `CapturePacket.frame_age_ms` (`recording/pipeline.rs`) state: age of
    the pre-event frame for a click; `0` for a post-event key-down frame.
  - `docs/adr/0001-pre-buffered-screen-capture.md`: append an amendment
    section dated 2026-08-18 that records the per-kind timing rule, the
    bounded worker-side wait, the pre-event fallback, the shared-frame
    consequence at ~10 fps, and the source decision (issue #38).
  - `README.md`: replace the parenthetical "captured from a pre-event
    frame, so the screen shows the state *before* the action" with the
    per-kind statement (clicks: pre-event; typing: first frame after the
    key so the typed character is visible).
- Owned paths: `src-tauri/src`,
  `docs/adr/0001-pre-buffered-screen-capture.md`, `README.md`

## Slice Cohesion

- Primary outcome: Typing steps show the just-typed character while click
  steps keep the pre-click picture.
- Primary execution flow: Tap callback pins the pre-event snapshot and
  enqueues -> worker resolves metadata and selects the display -> for a
  key-down, bounded wait for the first later frame -> `build_packet` cuts
  the triple from the selected frame -> emitter.
- Owning observable seam: The packet emitter fed by the capture worker
  (`PacketEmitter` receives `CapturePacket`s in event order).
- Primary acceptance criterion: AC-002
- Regression guards: AC-003
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: The broker query, the worker wait, and the packet
  assembly are one selection rule observed at one seam (the emitted
  packet); the documentation states that same rule. None is testable or
  meaningful without the others.
- Path-count warning: none

## Non-Goals

- Changing stream configuration, frame rate, retained-frame depth, queue
  capacity, or the saturation policy (DEC-009).
- Post-event frames for clicks; capturing both frames; any schema change;
  any change to key-event element crops; any UI change.

## Dependencies

- Slice dependencies: none
- Wave: 1
- Execution mode: serial

## Acceptance Coverage

- AC-002: This slice implements the per-kind selection rule end to end and
  proves it with broker unit tests and worker-level tests at the emitter.
- AC-003: This slice lands the ADR-0001 amendment, the README sentence, and
  the `frame_age_ms` doc comments; the reviewer checks them at the PR head.

## Verification

- Rust (`cargo test` in `src-tauri`):
  - broker: click selection unchanged (existing tests stay green); the
    post-event query returns the oldest retained frame after the event,
    `newest` when only it is later, and `None` when no retained frame is
    later; equality with the event timestamp is not "after".
  - worker: (1) a key-down enqueued with a pinned pre-event frame and a
    later frame published from another thread emits a packet whose shots
    come from the later frame and whose `frame_age_ms` is `0`; (2) a
    key-down whose deadline has already passed (event timestamp far in
    the past under the injected clock) emits immediately from the pinned
    frame; (3) a click enqueued between key-downs emits from its pinned
    pre-event frame, and packets leave in event order.
  - packets: `build_packet` with an overridden/post-event frame keeps
    crop and window rules and reports `frame_age_ms == 0`.
- `cargo build` warnings-free for touched code; `cargo fmt --check` if the
  repository formats Rust (verify locally).
- Documentation diff reviewed: ADR-0001 amendment, README sentence,
  doc comments.
- Independent command: `cd src-tauri && cargo test`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: asynchronous, stateful work (worker wait against a
  concurrently advancing broker; timing and ordering edge cases) across
  multiple files
- Binding: Claude task adapter request
