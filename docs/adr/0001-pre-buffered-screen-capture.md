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
