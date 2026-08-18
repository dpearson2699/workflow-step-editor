# PR-01 Plan: Per-kind frame selection with a post-event key-down frame

## Outcome

A key-down step's screenshot triple is cut from the oldest retained frame on
its display inside the bounded window `(event_ts, event_ts + 250 ms]`
(bounded wait on the capture worker, pinned pre-event fallback per DEC-002),
a click step's triple is unchanged, orderly stop lets an accepted key-down
finish its wait, and ADR-0001, the README, and the `frame_age_ms` doc
comments record the per-kind rule.

## Scope and Ownership

- Behavior: Implement DEC-001 and DEC-002 in the capture pipeline without
  changing its architecture:
  - `FrameBroker` gains one pure bounded range query for one display:
    the retained frame with the smallest `ts_ns` such that
    `event_ts_ns < ts_ns <= event_ts_ns + POST_EVENT_FRAME_WINDOW_NS`
    (250 ms), or `None`. Choose the minimum eligible timestamp across
    the two retained slots (do not assume `previous` is older than
    `newest`). Equality with the event is not eligible; equality with the
    deadline is eligible (deterministic boundary convention). Use
    `saturating_add`/`saturating_sub` for timestamp arithmetic. Do not
    change `eligible` or `snapshot`: click and fallback selection stay
    byte-for-byte as they are (including the existing bounded-retention
    approximation). The tap callback keeps pinning the pre-event snapshot
    for every event (`pipeline.rs::start_tap` behavior unchanged).
  - Extract the existing display selection (click: display under the
    point else main; key-down: focused element center, else focused
    window center, else main — DEC-008/DEC-011) from
    `packets.rs::build_packet` into one pure shared function that the
    worker and `build_packet` both use.
  - The capture worker (`worker.rs::run_capture_worker`) receives the
    shared broker handle and an injectable wait runtime: `now_ns()`,
    `wait_for(duration)`, the window/deadline constant, and the poll
    interval, so tests advance a fake clock and publish frames into the
    real broker inside `wait_for` without real sleeping. Production wiring
    in `pipeline.rs` (the macOS composition root) supplies
    `hostclock::host_now_ns`, `std::thread::sleep`, 250 ms, and a short
    poll interval (about 5 ms); the pure worker module stays buildable
    off macOS.
  - For every job the worker resolves metadata, selects the display with
    the shared function, and reads the pinned frame for that display from
    the job snapshot (its absence remains the existing fail-stop). A click
    uses the pinned frame directly and never consults the live broker. A
    key-down runs the bounded query/wait on that display ID: query first,
    then loop `wait_for(min(poll, remaining))` and re-query until a frame
    is found or `now_ns() >= event_ts + window`, with one final query at
    the deadline; the total requested wait never exceeds the remaining
    window (never past 250 ms after the event); a job that reaches the
    worker after its deadline queries once and never waits. It must not
    hold the broker mutex while waiting. If the candidate frame's `display`
    geometry differs from the selected event-time display geometry, use
    the pinned frame (GA-006). Otherwise use the candidate.
  - `packets.rs::build_packet` takes the explicit selected frame (a small
    value carrying the selected display and `Arc<FrameData>`) instead of
    reading `snapshot.frame_for` itself; every crop, fallback, and
    null-window rule is unchanged and all three shots come from that one
    frame. `frame_age_ms` keeps the existing saturating computation, so a
    post-event frame reports `0`. Never mutate or reinterpret
    `FrameSnapshot`: it remains "what the tap pinned at event time".
  - `pipeline.rs::stop` order becomes: stop the health monitor; stop and
    join the tap (closes the last `JobSender`); drop the display
    reconfiguration observer; join the capture worker (streams keep
    publishing during bounded waits); stop the stream manager; close the
    emitter guard. The quiescence contract (no emission after `stop`
    returns) is preserved.
  - Missing post-event frames fall back and never call `EmitterGuard::fail`;
    existing missing-pinned-frame and encode failures remain fail-stops.
    Queue capacity, saturation policy (DEC-009), schema, triples, crops,
    and UI are unchanged.
  - Doc comments on `CaptureMeta.frame_age_ms` (`domain/schema.rs`) and
    `CapturePacket.frame_age_ms` (`recording/pipeline.rs`): "Nonnegative
    event-to-selected-frame age in milliseconds, saturating at zero; a
    post-event key-down frame therefore reports 0."
  - `docs/adr/0001-pre-buffered-screen-capture.md`: append an amendment
    section dated 2026-08-18 recording the per-kind rule (clicks
    pre-event; key-downs the oldest retained frame in the bounded
    post-event window, intended to be the first post-event frame under
    normal worker latency), the worker-side bounded wait, the pinned
    fallback set (no in-window frame, no live frame, geometry mismatch),
    the shared-frame consequence at ~10 fps, that the event-anchored
    deadline prevents waits from stacking linearly (not that the queue
    cannot fill), the shutdown-order consequence, and the source
    decision (issue #38).
  - `README.md`: replace the parenthetical "captured from a pre-event
    frame, so the screen shows the state *before* the action" with the
    per-kind statement: click steps are cut from a pre-event frame so the
    screen shows the state before the click; typing steps use the first
    frame captured after the key when one arrives within 250 ms
    (best-effort, so the typed character is normally visible), otherwise
    the pre-event frame.
- Owned paths: `src-tauri/src/capture/broker.rs`,
  `src-tauri/src/capture/worker.rs`, `src-tauri/src/capture/packets.rs`,
  `src-tauri/src/capture/pipeline.rs`, `src-tauri/src/domain/schema.rs`,
  `src-tauri/src/recording/pipeline.rs`,
  `docs/adr/0001-pre-buffered-screen-capture.md`, `README.md`. The
  selected-frame value and the wait-runtime seam live inside `packets.rs`
  and `worker.rs`; no new module file is planned.

## Slice Cohesion

- Primary outcome: Typing steps show the just-typed character while click
  steps keep the pre-click picture.
- Primary execution flow: Tap callback pins the pre-event snapshot and
  enqueues -> worker resolves metadata and selects the display -> for a
  key-down, bounded wait for the oldest in-window frame -> `build_packet`
  cuts the triple from the selected frame -> emitter; orderly stop drains
  the worker before the streams stop.
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
- Cohesion proof: The broker query, the worker wait, the packet assembly,
  and the stop order are one selection rule observed at one seam (the
  emitted packet); the documentation states that same rule. None is
  testable or meaningful without the others.
- Path-count warning: none

## Non-Goals

- Changing stream configuration, frame rate, retained-frame depth, queue
  capacity, or the saturation policy (DEC-009); a condition-variable
  broker refactor; parallel per-event waiters.
- Post-event frames for clicks; capturing both frames; any schema change;
  any change to key-event element crops; any UI change; cross-generation
  crop translation.

## Dependencies

- Slice dependencies: none
- Wave: 1
- Execution mode: serial

## Acceptance Coverage

- AC-002: This slice implements the per-kind selection rule end to end and
  proves it with broker unit tests and deterministic worker-level tests at
  the emitter (matrix below).
- AC-003: This slice lands the ADR-0001 amendment, the README sentence, and
  the `frame_age_ms` doc comments; the reviewer checks them at the PR head.

## Verification

- Rust (`cd src-tauri && cargo test`), all deterministic (fake clock and
  injected `wait_for`; no real 250 ms sleeps); worker tests use frames
  with visibly different pixel payloads and timestamp gaps over 1 ms, and
  decode the emitted PNGs to assert which frame was used
  (`frame_age_ms == 0` alone is not proof):
  - broker: two in-window frames -> the smaller timestamp; frame equal to
    the event -> excluded; frame equal to the deadline -> included; only
    retained later frame after the deadline -> `None`; a later frame on
    another display -> `None` for the selected display; existing click
    snapshot tests unchanged.
  - worker: post frame published during the scripted wait -> emitted PNGs
    decode to the post-frame pixels and `frame_age_ms == 0`; no post frame
    before an actively reached deadline -> pinned pixels, no failure, no
    real sleep; job reaching the worker after its deadline with an
    in-window frame retained -> that frame with zero waits; job after its
    deadline with only an after-deadline frame -> pinned immediately;
    several key-downs before one frame -> may share it, later jobs do not
    stack full waits; key-down, click, key-down -> emitter order equals
    enqueue order and the click uses its pinned pixels despite newer
    broker frames; key-down resolved to a secondary display -> a
    primary-display post frame is ignored; same display ID with changed
    geometry -> pinned fallback; sender closes while a key-down waits and
    a frame is then published -> the job emits and the worker exits;
    the wait runtime records requested waits and the test asserts their
    sum never exceeds the remaining window.
  - `MacosCapturePipeline::stop` order (worker join before stream stop)
    is verified by review of `pipeline.rs` and by the AC-001 live run that
    presses Stop promptly after the final key; the composition root
    drives real SCStream/CGEventTap objects and is not unit-fakeable
    without adding seams outside this scope.
  - packets: `build_packet` with an explicit post frame -> full, window,
    and element PNGs decode from that frame and `frame_age_ms == 0`.
  - queue: existing saturation and FIFO tests unchanged.
- `cargo build` warnings-free for touched code on macOS; `cargo fmt --check`.
- Documentation diff reviewed: ADR-0001 amendment, README sentence, doc
  comments.
- Independent command: `cd src-tauri && cargo test`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: asynchronous, stateful work (worker wait against a
  concurrently advancing broker; timing, ordering, and shutdown edge
  cases) across multiple files
- Binding: Claude task adapter request
