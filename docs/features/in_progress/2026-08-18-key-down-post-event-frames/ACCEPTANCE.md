# Acceptance

## AC-001: Real recording shows typed characters and pre-click menus

- Ownership: feature
- Invariant: On the signed build from the PR-01 head, a recording in which
  the user types `hello` into a native text field produces five `type`
  steps whose screenshots each include the just-typed character, and a
  click on a menu item produces a `click` step whose screenshots show the
  menu as it looked at click time.
- Owning seam: The recorded workflow folder on disk and the review UI,
  inspected by the user.
- Evidence required: The user runs the recording and accepts; the root
  records the run under `review/timing-gate-run.md`.

## AC-002: Per-kind frame selection invariant

- Ownership: slice
- Invariant: For a click, the selected frame is the newest retained frame
  with a display timestamp not later than the event (unchanged). For a
  key-down, the selected frame is the oldest retained frame with a display
  timestamp after the event once such a frame exists; the wait for it runs
  on the capture worker with a deadline anchored to the event timestamp,
  and when the deadline passes without such a frame the pinned pre-event
  frame is used. Event order at the emitter is preserved, the tap callback
  never blocks, and `frame_age_ms` reports `0` for a post-event frame.
- Owning seam: `FrameBroker` selection plus the capture worker's packet
  assembly, observed at the packet emitter.
- Evidence required: Rust unit tests on the broker selection rule (click
  unchanged, key-down post-event, deadline fallback) and a worker-level
  test that emits packets in event order with the post-event frame chosen
  when it arrives after enqueue.

## AC-003: Documentation records the per-kind timing rule

- Ownership: slice
- Invariant: `docs/adr/0001-pre-buffered-screen-capture.md` records the
  per-kind timing rule and the no-frame fallback; the README no longer says
  every screenshot precedes the action; the `frame_age_ms` doc comments
  state the post-event value.
- Owning seam: The named documentation files at the PR head.
- Evidence required: Review of the changed documentation in the PR.
