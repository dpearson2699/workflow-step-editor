# AC-001 proven-gate run — 2026-08-17

User-run gate on the signed build from PR-03 final code head
`bc1fc6557081a231a56fd207caf41cf8863e9e42`
(`dev/proven-gate/script.md`, frozen fixture page in Chrome). The user
drove the recording; the coordinator spot-checked every artifact.

## Recording

- Workflow: `2026-08-17-205948-0f00` under
  `~/Library/Application Support/com.dpearson.workflow-step-editor/workflows/`
- 76 events, 76 manifest steps, 228 shot files (exactly 3 per event)
- Apps exercised: workflow-step-editor (recorder), TextEdit (native),
  Google Chrome (frozen fixture page), Dock, one permission dialog

## Frozen pass criteria — all met

1. Event-line count equals step count: 76 == 76.
2. Three decodable PNGs per event: all 228 carry the PNG signature; a
   6-event sips decode sample passed; visual inspection of
   `evt_0004.element.png` (tight crop of the fixture's static text) and
   `evt_0013.window.png` (full pre-event Chrome window) confirmed
   correct content.
3. Native `source: "ax"`: 59 TextEdit events (e.g. `AXTextArea`).
4. Chromium `source: "fallback"`: `evt_0003` click
   (`Click at (768, 785) — Google Chrome`) and key-downs
   `evt_0006..0013` (`Press H — Chrome`, `Press Cmd+S — Chrome`). Note:
   the user reported "couldn't type" in the fixture — the plain region
   is not editable so nothing echoed, but every key-down was captured
   through the fallback path as required.
5. Titles take the decided forms (`Click "Cancel" — Google Chrome`,
   `Press P — TextEdit`, `Press Cmd+S — Chrome`, coordinate fallback).
6. Steps streamed and persisted in event order (sequential ids,
   nondecreasing timestamps; live title updates observed by the user).
7. `capture.frame_age_ms` nonnegative for all 76 events (min 4,
   max 1478).

This disproves the tauri-apps/tauri#14770 class of failure for the raw
`CGEventTap` path. Verdict: PASS.

## Recorded deviations (non-blocking; routed to review triage)

- D1: Untitled AX elements render the role string as the quoted title:
  `Click "AXStaticText" — Google Chrome` (evt_0004/0005/0009). The
  decided grammar expects a usable element title or the coordinate
  fallback form.
- D2: App-name inconsistency for one app in one recording: key-down
  titles say `Chrome` (frontmost-app path) while click titles say
  `Google Chrome` (window-owner path).
- D3: DEC-011 shape deviation: four null-window Dock clicks
  (evt_0002/0015/0016/0075) record `window: null` with element
  `{role: "AXDockItem", source: "ax"}`; the declared null-window shape
  pins `role: null, title: null, source: "fallback"`. The data is
  richer than the declared shape but contradicts it.
- D4: One unmapped-key rendering: `Press Ctrl+Opt+Key 9 — TextEdit`
  (raw keycode fallback name).

None of these is a capture failure; the core AC-001 claim holds.
