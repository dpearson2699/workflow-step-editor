# PR-03 Plan: Real macOS capture adapter, dev trigger, and proven-gate readiness

## Outcome

The real macOS `CapturePipeline` implementation drops into the coordinator
that PR-02 already proved: ListenOnly event tap, pre-buffered per-display
streams, AX resolution, crop geometry, PNG encoding through the bounded
queue, native health/failure behavior, and a bare dev trigger. The
feature-owned proven gate (AC-001) runs on the signed build from this
slice's final code head before snapshot materialization, review, and
merge; only the authenticated bundle-snapshot commit follows that head.

## Scope and Ownership

- Behavior: macOS implementation of the PR-02 `CapturePipeline` trait.
  Internal owners (no one-file-per-item requirement): event-tap/run-loop
  driver (ListenOnly CGEventTap on a dedicated CFRunLoop thread; runtime
  `CGEventTapIsEnabled` health check; constant-bounded nonblocking
  callback that copies immutable event fields and pins an immutable
  eligible-frame snapshot before nonblocking enqueue, so a delayed worker
  cannot lose the required predecessor frame when the live broker
  advances), display stream manager (one
  continuous SCStream per active display; first-frame warm-up before the
  tap enables; restart on display-configuration changes; per-generation
  display sets published atomically; an event whose selected display
  retains any pre-event frame — including the outgoing generation's
  newest frame during warm-up — uses it, and an event whose selected
  display has no retained frame at all maps to the explicit fail-stop
  path, so no event is ever silently dropped; existing leases stay
  valid),
  frame-set broker (timestamped frames per display, event and frame
  timestamps converted to one monotonic host clock; each event selects
  the newest frame not later than its event timestamp; `frame_age_ms`
  derives from that comparison; leases carry the display frame geometry
  and scale metadata the crops need), window and AX resolver (the worker
  resolves metadata before PNG encoding; for key-downs it resolves the
  focused window first and then selects that display's frame; clicks:
  `CGWindowListCopyWindowInfo` hit test plus AX element at the click
  point; key-downs: frontmost app's focused window plus
  `AXFocusedUIElement` per DEC-008; deterministic implausible-frame
  fallback rule; fixed-size fallback crop about 300x200 pt clamped and
  scaled; key-down display selection uses the focused-element center,
  else the focused-window center, so a window spanning displays selects
  the display containing the element; null-window events follow DEC-011:
  display from the click point or main display, window crop = full
  display frame, element crop = fallback centered at the click point or
  display center), crop/scale geometry as pure tested arithmetic, PNG encoder
  behind the bounded queue with one ordered capture worker (capacity
  chosen from a retained-frame byte budget; overload signals out-of-band
  of the data queue), and the health/failure adapter mapping tap disable,
  stream failure, permission loss, queue saturation (DEC-009 fail-stop
  with one explicit capture-overloaded error), and an event without any
  retained frame for its display into the coordinator's single fail-stop
  path (DEC-007). Recording gating
  consumes the PR-01 production permission module unchanged; this slice
  adds no second permission implementation. A bare dev-only trigger in
  the shell page starts and stops recording and shows minimal live output
  (latest step title and received count) to prove the channel.
- Owned paths: `src-tauri/src/capture/`, `src-tauri/src/lib.rs`,
  `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`,
  `src-tauri/tauri.conf.json`, `src/`, `dev/proven-gate`
  (`dev/proven-gate/fixture.html` is the fixed Chrome fixture page;
  `dev/proven-gate/script.md` is the frozen gate script)
- Pinned invariants (from the accepted plan authority):
  1. A capture packet is one event plus one screenshot triple derived from
     one retained pre-event frame.
  2. `WorkflowStore::append_event` owns shot-file persistence and the
     JSONL append; capture code never writes workflow data directly.
  3. A step is published only after its event and all three shots commit.
  4. Queue saturation fail-stops explicitly (DEC-009); the tap callback
     never blocks and no event is silently dropped.

## Slice Cohesion

- Primary outcome: A recording session captures real global input into
  persisted schema v1 workflow data with per-event screenshot triples on
  a live macOS system.
- Primary execution flow: tap event -> pre-event frame snapshot ->
  window/element resolution -> triple encoding through the bounded queue
  -> PR-02 coordinator commit and step emission.
- Owning observable seam: The `CapturePipeline` trait boundary realized by
  the macOS adapter, observed through the unchanged PR-02 command layer.
- Primary acceptance criterion: Explicit observable criterion — a short
  real recording on the implementer's machine writes `events.jsonl`, a
  manifest, and three decodable PNGs per event through the unchanged
  PR-02 lifecycle, with fail-stop proven for tap disable and queue
  saturation.
- Regression guards: AC-002, AC-003, AC-004, AC-005 (consumed, not
  modified)
- New high-cost verification mechanism: signed-build manual capture check
  (the feature-owned proven gate's dry run; scripted with a recorded
  receipt)
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Tap, streams, broker, resolver, geometry, queue, and
  health adapter are one live capture flow behind the single trait
  boundary PR-02 already consumes; none of them is observable in
  production except through that seam, and the proven gate exercises them
  only together.
- Path-count warning: none

## Non-Goals

- The product review UI, permission status strip, and step-edit commands
  (review-UI capability, issue #13).
- Changes to the PR-02 coordinator contract, store seam, schema, parser,
  or channel envelope beyond realizing the trait.
- Burst grouping, keyboard shortcuts, synthetic `wait`/`assert` steps.
- Automatic recovery beyond fail-stop (DEC-007, DEC-009).
- Windows or Linux `CapturePipeline` implementations.

## Dependencies

- Slice dependencies: PR-02
- Wave: 3
- Execution mode: serial

## Acceptance Coverage

- Owns no acceptance criterion. It supplies the real-environment
  regression pass for the criteria PR-02 owns and readies the
  feature-owned proven gate, which runs on the signed build from the
  exact final code head before snapshot materialization, review, and
  merge.

## Verification

- Compile/API proof before broad integration: prove the selected
  ScreenCaptureKit crate's continuous stream startup, callback,
  first-frame signal, stop, and cross-thread frame lifetime; prove the
  selected AX binding's point hit test, focused-element and
  focused-window lookup, role/title, and position/size calls; prove a raw
  ListenOnly tap sees key-downs while the Tauri window is focused. Choose
  exactly one AX binding after this proof. Verify live crate versions
  before locking (research observed 2026-08-16: core-graphics 0.25.0,
  screencapturekit 8.0.1, accessibility-sys 0.2.0, axuielement 0.9.1).
- `cargo test` for pure geometry (rectangle normalization and clamping
  with negative display origins, Retina point-to-pixel scaling, windows
  partly outside the selected display, implausible AX frames fall back,
  key-down fallback centered in focused-window bounds, null-window
  display selection and both DEC-011 fallback crops), queue-saturation
  fail-stop per DEC-009, single fail-stop transition for tap disable,
  stream failure, and permission loss, display-set replacement keeping
  existing leases alive — an event with an outgoing-generation frame
  still captures, and an event whose display retains no frame at all
  takes one explicit fail-stop, a
  broker-advance test (advance the live broker after enqueue and verify
  the pinned predecessor frame is still selected), a
  display-spanning-window selection test on mixed-scale geometry, and a
  stream-manager restart test for display-configuration changes. The
  manual smoke check includes one native display-change case (an
  arrangement or resolution change; hot-plug when hardware allows).
- Real-capture smoke check on the implementer's machine with the signed
  dev build: scripted short recording; verify event-line count, three
  decodable PNGs per event, `source: "ax"` on a native app, fallback on a
  Chromium app (use a fixed local HTML fixture page opened in Chrome with
  a large plain content region; verify during the smoke check that its
  observed AX result is coarse — AXWebArea-class — so the
  implausible-frame rule triggers, and freeze that exact target into the
  gate script), accepted title forms, ordered channel delivery, and
  nonnegative `frame_age_ms`; record apps, counts, workflow folder, and
  observed maximum queue depth in the receipt. Clock-domain conversion is
  covered by a test with delayed callbacks and equal timestamps. The user-facing proven
  gate itself is AC-001: it runs on the locally built signed `.app`
  (identity injected) from the exact final code head, before snapshot
  materialization, review, and merge.
- `cargo clippy` clean for touched code. At the final code head both
  builds succeed: `npm run tauri build` with the injected identity
  (`codesign -dv` shows the fixed bundle identifier and accepted
  identity) and a certificate-free build.
- Independent command: `cargo test --manifest-path src-tauri/Cargo.toml`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 xhigh
- Selection predicates: interacting concurrency and state invariants (tap
  thread, stream callbacks, frame leases, bounded queue, ordered worker);
  cross-module coordination (tap -> broker -> resolver -> encoder ->
  coordinator seam)
- Binding: claude_task_request (Claude task adapter)

## Parallelization Assessment

- No same-wave pair: every wave in this bundle holds exactly one slice and
  runs serially, so no pair record applies.
