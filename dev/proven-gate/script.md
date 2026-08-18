# Proven capture gate — frozen script (AC-001)

> Historical gate of the capture-pipeline bundle: its AC-001/PR-03 labels belong to that completed bundle, not to the review-UI bundle, whose final gate is `dev/review-ui-gate/script.md`.

This is the frozen, user-run acceptance script for the macOS capture
pipeline. It runs on the locally built, signed `.app` from the exact
final code head of PR-03, before snapshot materialization, review, and
merge. The implementer readies this script and the fixture; the user
runs the gate and inspects the recorded files.

AC-001 is owned by the feature, not this slice. This document only
freezes the exact sequence and the pass criteria.

## Prerequisites

1. Build the signed dev app from the final code head:
   ```
   APPLE_SIGNING_IDENTITY="Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)" \
     npm run tauri build
   ```
   The bundle identifier must be `com.dpearson.workflow-step-editor`
   (fixed, DEC-006) so TCC grants persist across rebuilds.
2. Launch the built app from
   `src-tauri/target/release/bundle/macos/workflow-step-editor.app`.
   Launch the `.app`, not `cargo run` / `npm run tauri dev`: TCC binds
   grants to the signed bundle identity, and the dev binary is a
   different identity.
3. In the app, click **Check permissions**, then **Request** for
   Input Monitoring first, then Accessibility, then Screen Recording.
   Grant all three in System Settings when prompted. Input Monitoring
   must be requested before Accessibility (prompt-suppression caveat,
   AC-005 / DEC-011). Re-check until all three read `granted`.

## Frozen targets

- Native app target: **TextEdit** with one open document window.
- Chromium target: **Google Chrome** showing the frozen fixture page
  `dev/proven-gate/fixture.html` (open it via
  `file://` in Chrome; maximize the window so the plain region is
  large). This exact page is the frozen Chromium target — do not
  substitute another URL.

## Sequence

Run one continuous recording that covers every required case:

1. In the app, click **Start recording**. Confirm the status shows
   `recording` and a workflow id.
2. **Native, recorder window focused:** with the app window focused,
   press a few keys (for example type `hi` and press `Cmd+S`).
3. **Native app clicks and typing:** switch to TextEdit. Click a
   toolbar or window control (a titled button), then click into the
   text area and type a short sentence. This exercises click element
   resolution and key-down focused-element resolution on a native app.
4. **Chromium fallback:** switch to Chrome showing the fixture page.
   Click inside the large plain region, then type a few characters.
5. **Recorder window unfocused typing:** click back into TextEdit (so
   the recorder window is unfocused) and type another short phrase.
6. (Optional, when hardware allows) trigger one native display change
   — change the arrangement or resolution, or hot-plug a display —
   while still recording, then click once more. Streams restart on the
   change; capture must continue.
7. In the app, click **Stop recording**. Confirm the status shows
   `stopped` with the workflow id, and note the received-step count.

## Inspect the recorded workflow

The workflow folder is under the app data directory:
`~/Library/Application Support/com.dpearson.workflow-step-editor/workflows/<id>/`.

Verify:

- `events.jsonl` has exactly one JSON line per recorded event, and the
  line count matches the received-step count shown in the app.
- `shots/` holds three decodable PNGs per event
  (`<event_id>.full.png`, `<event_id>.window.png`,
  `<event_id>.element.png`). Open a few to confirm they decode and show
  pre-event screen state.
- At least one native-app event (TextEdit) has
  `element.source: "ax"` with a tight `element.frame`.
- At least one Chromium event (the fixture page) has
  `element.source: "fallback"` with the fixed-size element crop.
- Step titles follow the decided forms:
  `Click "OK" — TextEdit`, `Click at (x, y) — <app>`,
  `Press H — <app>`, `Press Cmd+S — TextEdit`.
- Every event's `capture.frame_age_ms` is a nonnegative integer.
- Steps stream in event order (the app's latest-step title updates in
  order; the manifest `steps` preserve that order).
- `workflow.json` has `schema_version: 1` and `steps[*].event_ids`
  referencing the events.

## Fail-stop spot checks (optional but recommended)

- **Queue saturation (DEC-009):** not directly user-triggerable; covered
  by the automated `cargo test` case
  `capture::queue::tests::saturation_is_an_explicit_error_and_earlier_jobs_survive_in_order`
  and the coordinator fail-stop tests.
- **Tap disable:** revoking Input Monitoring in System Settings during a
  recording must fail-stop the session (the app shows a `failed:`
  terminal) while preserving already-committed events and shots.

## Pass criteria

The gate passes when the user confirms: the event-line count matches the
step count, three decodable PNGs exist per event, a native event shows
`source: "ax"`, a Chromium event shows `source: "fallback"`, titles take
the accepted forms, steps arrived in order, and every `frame_age_ms` is
nonnegative. This disproves the tauri-apps/tauri#14770 class of failure
for the raw `CGEventTap` path.
