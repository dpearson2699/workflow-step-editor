# Interview

Issue #38 records the user's design-gate decision (clicks pre-event, typing
post-event; everything-post and both-frames rejected). The gray areas below
are what the issue and the current code do not settle.

## GA-001: A post-event frame is not guaranteed to arrive

- Status: closed
- Kind: fact
- Uncertainty: Whether the per-display SCStream delivers a new frame after
  every key-down. If the screen does not change, no frame with a later
  display timestamp may ever be published.
- Why it matters: The key-down selection rule "first buffered frame after
  the event" needs a defined outcome when no such frame exists, or the
  worker would stall and the queue would saturate (DEC-009 fail-stop).
- Evidence inspected: `src-tauri/src/capture/macos/stream.rs`
  (`setMinimumFrameInterval(CMTime::new(1, 10))`, ~10 fps; `copy_frame`
  returns `None` for idle frames without an image buffer, so idle periods
  publish nothing); `src-tauri/src/capture/broker.rs` (only `newest` and
  `previous` frames are retained per display); ScreenCaptureKit delivers
  screen samples on content change, subject to the minimum interval.
- Confidence: high
- Question: Q-001 (the behavior when no frame arrives is a decision)

## GA-002: Where the post-event wait runs

- Status: answered_by_docs
- Kind: decision
- Uncertainty: The tap callback pins the frame snapshot synchronously
  before enqueue (`pipeline.rs::start_tap`). A post-event frame does not
  exist at that instant, so something must wait for it: the tap callback,
  the stream sink, or the capture worker.
- Why it matters: Blocking the tap risks `kCGEventTapDisabledByTimeout`;
  moving enqueue to the stream sink breaks event ordering (a click after
  a key-down would enqueue first).
- Evidence inspected: DEC-009 (foundation bundle): "The tap callback never
  blocks, and no event is silently dropped or coalesced";
  `src-tauri/src/capture/queue.rs` and `worker.rs` (single FIFO worker is
  the ordering guarantee); `packets.rs::build_packet` (the key-down
  display is chosen from worker-resolved metadata, so the display to wait
  on is only known on the worker).
- Confidence: high
- Question: none
- Closure: The wait runs on the capture worker after metadata resolution
  and display selection; the tap still pins the pre-event snapshot for
  every event (DEC-001). Pro primary (2026-08-18) confirmed the seam and
  added: the query must be bounded on both sides of the window, and
  orderly stop must join the worker before stopping the streams.

## GA-003: `frame_age_ms` for a post-event frame

- Status: closed
- Kind: fact
- Uncertainty: `CaptureMeta.frame_age_ms: u64` is documented as "Age of the
  buffered pre-event frame when the event fired". A post-event frame has a
  negative age.
- Why it matters: Decides whether the schema needs a signed or new field.
- Evidence inspected: `src-tauri/src/domain/schema.rs:126`,
  `src-tauri/src/recording/pipeline.rs:34`; `rg frame_age_ms src/` finds
  no frontend consumer; `FrameSnapshot::frame_age_ms` already saturates at
  zero; issue #38 scope excludes schema work.
- Confidence: high
- Question: none
- Closure: Keep `u64`; a post-event frame reports `0`; the doc comments
  state "0 for a post-event key-down frame". No schema change (coordinator
  engineering decision inside accepted scope).

## GA-004: Several queued key-downs may share one post-event frame

- Status: closed
- Kind: fact
- Uncertainty: At ~10 fps, several key-downs can precede one frame; each
  selects that same frame, which then shows more than its own character.
- Why it matters: Confirms the acceptance wording "each include the
  just-typed character" (inclusion, not exclusivity).
- Evidence inspected: issue #38 acceptance text; `stream.rs` frame
  interval.
- Confidence: high
- Question: none
- Closure: Accepted consequence of the issue's decision; recorded in
  DEC-001 and the ADR amendment.

## GA-005: How the accepted outcome is proven

- Status: closed
- Kind: decision
- Uncertainty: The issue's acceptance is a real-recording statement
  ("Typing hello ... clicking a menu item ..."). Unit tests prove the
  selection rule; only a real recording proves the visible outcome.
- Why it matters: Decides whether a user-run recording is a blocking
  acceptance criterion before merge (foundation-bundle precedent, AC-001
  proven gate) or the automated selection tests suffice.
- Evidence inspected:
  `docs/features/completed/2026-08-17-capture-pipeline-and-backend-foundation/review/proven-gate-run.md`
  (user-run gate on the PR head signed build); issue #38 origin (user
  observation on a real recording).
- Confidence: medium
- Question: Q-002
- Closure: User answered Q-002 (2026-08-18): AC-001 stays feature-owned and
  blocks the final PR merge (DEC-003).

## GA-006: Display geometry change inside the key-down timing window

- Status: closed
- Kind: decision
- Uncertainty: A post-event frame can belong to a newer display-set
  generation than the event-time snapshot (`FrameBroker::publish_displays`
  replaces the set while pinned leases survive), so its geometry can differ
  from the geometry the crops are computed against.
- Why it matters: Mixing generations would produce inconsistent crop
  geometry; the plan needs a defined outcome.
- Evidence inspected: `src-tauri/src/capture/broker.rs`
  (`publish_displays`, `FrameData.display`); Pro primary
  (`discovery/pro-lifecycle-evidence/aa6bf429...md`, "Display selection
  and selected-frame handoff").
- Confidence: high
- Question: none
- Closure: Coordinator engineering default inside accepted scope: when the
  candidate post-event frame's `display` geometry differs from the selected
  event-time display, use the pinned pre-event frame (DEC-002). Capturing
  through a display-topology change is outside this feature's goal
  (Non-Goals: no architecture change beyond what post-event selection
  needs); a reconfiguration inside a 250 ms window is not a supported use
  case worth a product decision.

## Q-001: What does a key-down step use when no post-event frame arrives?

- Status: answered
- Recommendation: Wait on the worker for a frame with a display timestamp
  after the event, with a deadline anchored to the event timestamp (about
  250 ms, ~2.5 minimum frame intervals). If none arrives, use the pinned
  pre-event frame. Rationale: a screen that publishes no new frame did not
  change, so the pre-event frame is also the post-event picture; the
  recording never fail-stops for a silent key; the event-anchored deadline
  means a burst of silent key-downs (key repeat on a static screen) shares
  one wait instead of stacking waits, so the queue cannot saturate from
  waiting alone.
- Options and tradeoffs: (a) Recommended: bounded wait, then the pinned
  pre-event frame. (b) Fail-stop the recording with an explicit error:
  literal to "post-event frame" but ends recordings on any key that
  changes nothing on screen (Cmd+S, arrow at end of text). (c) Wait
  without a deadline: worker stalls, queue saturates, DEC-009 fail-stop.
- If wrong: With (a) the only miss is a key whose visual effect lands
  after ~250 ms with no other screen change in between; the step then
  shows the pre-event field. With (b) or (c), ordinary recordings end
  early.
- Answer/source: User answer 2026-08-18: option (a), bounded wait then the
  pinned pre-event frame.
- Closure reason: The user selected the recommended behavior; the deadline
  value is an implementation constant inside accepted scope.
- Decision: DEC-002
- Canonical-doc impact: `docs/adr/0001` amendment records the fallback.

## Q-002: Is a user-run real recording a blocking acceptance before merge?

- Status: answered
- Recommendation: Yes. Keep AC-001 as a feature-owned, user-run recording on
  the PR-01 head signed build (same shape as the foundation proven gate):
  type "hello" in a native app, click a menu item, and inspect the steps.
  It is the only proof of the visible outcome the issue asks for, and it
  guards the click regression on a real display.
- Options and tradeoffs: (a) Recommended: user-run recording is AC-001 and
  blocks the final PR merge until passed. (b) Automated selection tests
  only; AC-001 waived: faster, but the visible outcome stays unverified
  until someone records again.
- If wrong: With (a) the cost is one short recording session by the user.
  With (b) a timing or rendering-latency miss ships unnoticed.
- Answer/source: User answer 2026-08-18: option (a), the user-run recording
  is AC-001 and blocks the final PR merge.
- Closure reason: The user selected the recommended acceptance ownership.
- Decision: DEC-003
- Canonical-doc impact: none
