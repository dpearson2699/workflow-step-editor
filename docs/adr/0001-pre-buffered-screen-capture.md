# Pre-buffered screen capture

The capture pipeline must attach a screenshot triple to every recorded input
event. Capturing on demand after the event races the UI: a click that opens a
menu or navigates can repaint the screen before the screenshot lands, so the
artifact would show post-action state. We therefore run a continuous
ScreenCaptureKit stream per active display and keep the latest frame in
memory; when an event arrives, the pipeline saves the buffered pre-event frame
and derives all three artifacts from it.

## Considered options

- On-demand `SCScreenshotManager` capture after each event: simpler and no
  standing stream, but the artifact can miss the pre-action UI state — a
  failure mode we chose not to introduce.

## Consequences

- The window crop is a bounds crop of the display frame, not an isolated
  window capture, so overlapping windows can appear inside it.
- One stream runs per active display; streams restart when the display
  configuration changes.
- Recording pays a standing capture cost (a running stream), not a per-event
  capture cost.
- All three artifacts share one capture instant, so they are consistent with
  each other and precede the event.

## Amendment 2026-08-18: per-kind frame timing (issue #38)

The pre-event rule above now applies to click events only. Key-down events
select a post-event frame, because the pre-event frame of a typing step shows
the field without the typed character and was judged useless
([issue #38](https://github.com/dpearson2699/workflow-step-editor/issues/38)).

- **Clicks** keep the pre-event selection unchanged: the tap callback pins the
  newest retained frame not later than the event, and the artifact shows the
  control before the UI repaints.
- **Key-downs** use the oldest retained frame on the selected display whose
  display timestamp lies in the bounded window `(event_ts, event_ts + 250 ms]`.
  Under normal worker latency this is the first post-event frame; the broker
  retains two frames per display. Frames equal to the event timestamp are not
  eligible; a frame equal to the deadline is.
- The tap callback still pins the pre-event snapshot for every event and never
  blocks. The bounded wait for the post-event frame runs on the capture worker
  after it has resolved metadata and chosen the display, with the deadline
  anchored to the event timestamp. Because the deadline is event-anchored, a
  burst of key-downs on a static screen shares one wait instead of stacking
  waits linearly; a burst can still fill the bounded queue during that first
  wait, and the existing saturation fail-stop remains the policy for that case.
- **Fallback to the pinned pre-event frame** when no in-window frame exists at
  the deadline, when the selected display retains no live frame, or when the
  candidate frame's display geometry differs from the event-time display
  geometry. A screen that publishes no new frame did not change, so the
  pre-event frame is also the post-event picture; the wait never fail-stops
  the recording by itself.
- `frame_age_ms` keeps its saturating computation and reports `0` for a
  post-event frame.

### Consequences

- At the ~10 fps stream rate, several key-downs that precede one frame share
  that frame, so a typing step's screenshots may show more than its own
  character.
- A key-down step pays up to one frame interval (about 100 ms, bounded at
  250 ms) of added latency before its packet is emitted.
- Orderly stop joins the capture worker before it stops the display streams,
  so a key-down accepted just before Stop can still receive its post-event
  frame; the streams stop afterwards and the emitter quiesces last.
