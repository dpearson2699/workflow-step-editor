# Decisions

## DEC-001: Frame selection is per event kind

- Status: accepted
- Decision: Click events keep the current pre-event selection unchanged
  (`RetainedFrames::eligible`: the newest retained frame not later than the
  event, or the oldest retained frame when both retained frames are later —
  the existing bounded-retention approximation). Key-down events select the
  first buffered frame whose display timestamp is after the event; one
  frame interval of added latency is the expected cost and the wait is
  bounded at 250 ms (DEC-002). The tap callback still pins the pre-event
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
  retained frame from inside that window. The selected frame is the
  oldest eligible frame still retained (intended to be the first
  post-event frame under normal worker latency; the broker retains two
  frames per display). When no in-window frame exists, when the selected
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
  signed build from the PR-01 head: type `hello` into a native text field
  and click a menu item, then inspect the steps. It blocks the final PR
  merge until the user accepts; the root records the run under
  `review/timing-gate-run.md`.
- Rationale: User answer to Q-002 (2026-08-18). The issue's acceptance is
  a visible-outcome statement that only a real recording proves; the
  foundation bundle's proven gate is the precedent.
- Rejected alternatives: Automated selection tests only with AC-001
  waived.
- Canonical docs: none.
