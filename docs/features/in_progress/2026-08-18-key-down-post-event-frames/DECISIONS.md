# Decisions

## DEC-001: Frame selection is per event kind

- Status: accepted
- Decision: Click events keep the pre-event frame (the newest retained frame
  not later than the event). Key-down events select the first buffered
  frame whose display timestamp is after the event; one frame interval of
  added latency is acceptable. The tap callback still pins the pre-event
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
  whose display timestamp is after the event. The wait deadline is
  anchored to the event timestamp (about 250 ms, roughly 2.5 minimum
  frame intervals at the ~10 fps stream rate); a job that reaches the
  worker after its deadline does not wait. When no such frame exists at
  the deadline, the step uses its pinned pre-event frame. The wait never
  fail-stops the recording by itself and never runs in the tap callback.
- Rationale: User answer to Q-001 (2026-08-18). A screen that publishes no
  new frame did not change, so the pre-event frame is also the post-event
  picture. Anchoring the deadline to the event timestamp means a burst of
  silent key-downs (key repeat on a static screen) shares one wait rather
  than stacking waits, so waiting alone cannot saturate the bounded queue
  (DEC-009).
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
