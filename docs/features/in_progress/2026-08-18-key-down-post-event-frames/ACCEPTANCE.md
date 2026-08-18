# Acceptance

## AC-001: Real recording shows typed characters and pre-click menus

- Ownership: feature
- Invariant: On the signed build from the head of the final implementation
  slice (PR-02; the PR-01 head `176be565` run of 2026-08-18 is failed
  historical evidence, GA-007), a recording in which
  the user types `hello` into a native text field produces five `type`
  steps whose screenshots each include the just-typed character, and a
  click on a menu item produces a `click` step whose screenshots show the
  menu as it looked at click time.
- Owning seam: The recorded workflow folder on disk and the review UI,
  inspected by the user.
- Evidence required: Build the exact final-slice (PR-02) head with `npm run tauri build`
  (the signed app, because macOS TCC binds permission grants to the signed
  bundle identity) and launch it; the user records the `hello` typing, a
  menu-item click, and one run where Stop is pressed promptly after the
  final key; the user inspects the steps and accepts. The root records the
  exact PR head, app identity, workflow id, event ids, inspected shot
  files, `frame_age_ms` values for the key-down events, and the verdict
  under `review/timing-gate-run.md`.

## AC-002: Superseded — PR-01 selection invariant (oldest in-window frame)

- Ownership: slice
- Invariant: Historical. This criterion described the "oldest retained
  in-window frame" key-down rule implemented by PR-01 (PR #39, head
  `176be565`). The rule was superseded on 2026-08-18 by DEC-004 after the
  AC-001 gate (GA-007); the live invariant is AC-005, owned by PR-02. No
  implementation is expected to satisfy this criterion; it is closed as
  waived (superseded) at completion.
- Owning seam: none (superseded).
- Evidence required: none beyond the supersession record in
  `pr/PR-01/RECEIPT.md` and DEC-004.

## AC-003: Superseded — PR-01 documentation of the oldest-frame rule

- Ownership: slice
- Invariant: Historical. This criterion covered the ADR-0001 amendment,
  README sentence, and doc comments as written by PR-01 for the superseded
  rule. The live documentation invariant is AC-006, owned by PR-02. Closed
  as waived (superseded) at completion.
- Owning seam: none (superseded).
- Evidence required: none beyond the supersession record in
  `pr/PR-01/RECEIPT.md`.

## AC-004: Superseded PR-01 attempt is closed unmerged and integrated

- Ownership: feature
- Invariant: The PR-01 implementation of the superseded "oldest in-window
  frame" rule (PR #39, head `176be565676b5abc383f684a81597893c3260524`) is
  never merged: PR #39 is closed as superseded, and its commit is an
  ancestor of the PR-02 head so no work is lost.
- Owning seam: GitHub PR #39 state and PR-02 branch ancestry.
- Evidence required: `gh pr view 39` showing `CLOSED` and not merged, and
  `git merge-base --is-ancestor 176be565… <PR-02 head>` succeeding;
  recorded by the root under `review/pr-01-supersession.md`.

## AC-005: Per-kind frame selection invariant (DEC-004)

- Ownership: slice
- Invariant: For a click, frame selection is byte-for-byte the current
  pre-event behavior (`RetainedFrames::eligible` and the pinned snapshot
  are unchanged, including the existing bounded-retention approximation).
  For a key-down, the capture worker waits until a retained frame with
  `ts >= event_ts + 100 ms` exists on the selected display or the 250 ms
  deadline passes (deadline anchored to the event timestamp), then selects
  the newest retained frame whose display timestamp lies in
  `(event_ts, event_ts + 250 ms]`; when no in-window frame exists, when the
  display retains no live frame, or when the candidate frame's geometry
  differs from the event snapshot, the pinned pre-event frame is used. Event order at the emitter
  is preserved, the tap callback never blocks, orderly stop lets an
  accepted key-down finish its bounded wait before the streams stop, and
  `frame_age_ms` reports `0` for a post-event frame.
- Owning seam: `FrameBroker` selection plus the capture worker's packet
  assembly, observed at the packet emitter.
- Evidence required: Rust unit tests on the broker range query (in-window
  frame chosen, newest of two, event-equal excluded, deadline-equal
  included, after-deadline excluded, other display ignored; existing click
  tests unchanged) and deterministic worker-level tests under an injected
  wait runtime that decode emitted PNG pixels: an early in-window frame
  followed by a settle-satisfying frame selects the later frame; a single
  early frame with no later frame is selected at the deadline; active
  deadline timeout to the pinned frame; late job with an in-window frame;
  late job with only an after-deadline frame; key-down then click then
  key-down in emitter order; and drain after sender close.

## AC-006: Documentation records the per-kind timing rule (DEC-004)

- Ownership: slice
- Invariant: `docs/adr/0001-pre-buffered-screen-capture.md` records the
  per-kind timing rule and the no-frame fallback; the README no longer says
  every screenshot precedes the action; the `frame_age_ms` doc comments
  state the post-event value.
- Owning seam: The named documentation files at the PR head.
- Evidence required: Review of the changed documentation in the PR.
