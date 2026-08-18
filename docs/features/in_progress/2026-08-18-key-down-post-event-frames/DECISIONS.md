# Decisions

## DEC-001: Frame selection is per event kind

- Status: accepted
- Decision: Click events keep the current pre-event selection unchanged
  (`RetainedFrames::eligible`: the newest retained frame not later than the
  event, or the oldest retained frame when both retained frames are later —
  the existing bounded-retention approximation). Key-down events select a
  post-event frame: the newest retained frame inside the bounded window
  `(event_ts, event_ts + 250 ms]`, chosen after a 100 ms settle (DEC-004,
  which supersedes this decision's original "first buffered frame after
  the event" clause); the added latency is at least the 100 ms settle and
  at most the 250 ms deadline (DEC-002). The tap callback still pins the pre-event
  snapshot for every event; the capture worker performs the key-down
  post-event selection after it has resolved metadata and chosen the
  display (DEC-008 rule unchanged). Several key-downs that precede one
  frame share that frame.
- Rationale: User decision at the review-UI final design gate (issue #38):
  the pre-event frame for typing shows the field without the character and
  was judged useless; the click semantics of ADR-0001 must stay.
- Rejected alternatives: Everything post-event (breaks menu/navigation
  clicks); both frames per event (storage and viewer cost); waiting in the
  tap callback (DEC-009 forbids blocking the tap); enqueue from the stream
  sink (breaks event ordering).
- Canonical docs: `docs/adr/0001-pre-buffered-screen-capture.md`
  amendment; `README.md` capture sentence.

## DEC-002: Bounded post-event wait with pre-event fallback

- Status: accepted
- Decision: For a key-down, the capture worker waits for a retained frame
  whose display timestamp lies in the bounded window
  `(event_ts, event_ts + 250 ms]` on the selected display. The deadline is
  anchored to the event timestamp (about 250 ms, roughly 2.5 minimum
  frame intervals at the ~10 fps stream rate); a job that reaches the
  worker after its deadline does not wait, but it may still use a
  retained frame from inside that window. Which in-window frame is
  selected is decided by DEC-004 (the original "oldest eligible frame"
  clause of this decision was superseded on 2026-08-18 after the AC-001
  gate). When no in-window frame exists, when the selected
  display retains no live frame, or when the candidate frame's display
  geometry differs from the event-time snapshot's geometry, the step uses
  its pinned pre-event frame. The wait never fail-stops the recording by
  itself and never runs in the tap callback. Orderly pipeline stop keeps
  the display streams publishing until the worker has drained the
  accepted jobs, so a key-down accepted just before Stop can still receive
  its post-event frame.
- Rationale: User answer to Q-001 (2026-08-18). A screen that publishes no
  new frame did not change, so the pre-event frame is also the post-event
  picture. Anchoring the deadline to the event timestamp means a burst of
  silent key-downs (key repeat on a static screen) shares one wait rather
  than stacking waits linearly; a burst can still fill a small queue during
  that first wait, and DEC-009's explicit fail-stop remains the policy for
  that case. The bounded upper edge of the window keeps a late worker from
  selecting a frame taken after the deadline (Pro primary, 2026-08-18).
- Rejected alternatives: Fail-stop on a missing post-event frame (ends
  recordings on ordinary silent keys); unbounded wait (worker stall,
  queue saturation, DEC-009 fail-stop); waiting in the tap callback.
- Canonical docs: `docs/adr/0001-pre-buffered-screen-capture.md`
  amendment records the fallback.

## DEC-003: Real-recording acceptance ownership

- Status: accepted
- Decision: AC-001 is a feature-owned, user-run real recording on the
  signed build from the head of the final implementation slice (originally
  PR-01; since 2026-08-18 the replacement slice PR-02 — the PR-01 head
  `176be565` run is failed historical evidence, GA-007): type `hello` into
  a native text field and click a menu item, then inspect the steps. It
  blocks the final PR merge until the user accepts; the root records the
  run under `review/timing-gate-run.md`.
- Rationale: User answer to Q-002 (2026-08-18). The issue's acceptance is
  a visible-outcome statement that only a real recording proves; the
  foundation bundle's proven gate is the precedent.
- Rejected alternatives: Automated selection tests only with AC-001
  waived.
- Canonical docs: none.

## DEC-004: Newest in-window frame after a 100 ms settle

- Status: accepted
- Decision: For a key-down, the worker waits until either a retained frame
  with `ts >= event_ts + 100 ms` (one minimum frame interval,
  `POST_EVENT_SETTLE_NS`) exists on the selected display or the 250 ms
  deadline (`event_ts + POST_EVENT_FRAME_WINDOW_NS`) passes; it then
  selects the newest retained frame with `ts` in
  `(event_ts, event_ts + 250 ms]`. A job that reaches the worker after its
  deadline queries once (newest in-window) and never waits. When no
  in-window frame exists, DEC-002's fallback set applies (pinned pre-event
  frame). This supersedes DEC-002's "oldest eligible frame" clause; every
  other DEC-002 rule (worker-side, event-anchored deadline, fallback set,
  never fail-stop, never in the tap, stop order) is unchanged.
- Rationale: User answer to Q-003 (2026-08-18) after the AC-001 gate on PR
  #39 showed the first keystroke's oldest in-window frame captured the
  dirty-state title repaint before the glyph (GA-007). Newest-in-window
  lets the glyph frame supersede an intermediate repaint; the settle bound
  keeps latency low while the screen keeps changing (typing) and caps an
  isolated key at 250 ms.
- Rejected alternatives: Newest in-window frame only at the 250 ms
  deadline (every typing step pays 250 ms); keeping the oldest-frame rule.
- Canonical docs: `docs/adr/0001-pre-buffered-screen-capture.md`
  amendment; `README.md` sentence.

## DEC-005: Pro-primary waiver for the DEC-004 change

- Status: accepted
- Decision: The user explicitly stopped further ChatGPT Pro sends for the
  DEC-004 rule change (Q-004). The material specification change is
  adopted under the `adopt-plan` Pro-primary waiver with a fresh CLEAN
  blind-completeness receipt; the immutable Pro response
  (`discovery/pro-lifecycle-evidence/aa6bf429...md`) remains recorded
  evidence.
- Rationale: Bounded rule correction for the same accepted goal, owning
  seam, and architecture, delivered through the replacement slice PR-02
  (PR-01 superseded); time budget of the take-home project.
- Rejected alternatives: Fresh Pro primary plus consensus loop.
- Canonical docs: none.

## DEC-006: Content-aware post-event frame selection

- Status: accepted
- Decision: For a key-down, the worker waits (bounded by the 250 ms
  event-anchored deadline, DEC-002) for the OLDEST retained in-window frame
  on the selected display whose pixels inside the selected element crop
  rectangle differ from the pinned pre-event frame's pixels in that same
  rectangle; it selects that frame as soon as it exists. If no differing
  frame exists at the deadline, it selects the newest in-window frame; if
  no in-window frame exists (or geometry mismatches), the pinned pre-event
  frame (DEC-002 fallback set). This supersedes DEC-004's 100 ms settle;
  the newest-in-window range query remains as the deadline fallback and
  `POST_EVENT_SETTLE_NS` is removed.
- Rationale: User answer to Q-005 (2026-08-18) after the PR #41 gate
  (GA-009): the typed glyph always appears inside the focused element,
  while the first-key title-only repaint does not, and any time-based
  settle overshoots at typing speed with ~10 fps capture.
- Rejected alternatives: Oldest in-window frame without settle (first-key
  repaint); settle-then-newest (later characters visible); raising the
  capture frame rate (cost, and still time-based).
- Canonical docs: `docs/adr/0001-pre-buffered-screen-capture.md`
  amendment; `README.md` sentence.

