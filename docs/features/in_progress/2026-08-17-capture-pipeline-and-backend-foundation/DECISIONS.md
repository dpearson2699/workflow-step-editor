# Decisions

DEC-001 through DEC-005 restate user decisions already accepted in the
wayfinder decision tickets so the bundle is self-contained. Their canonical
owners are the cited tickets and ADRs; this file does not amend them.

## DEC-001: Capture architecture

- Status: accepted
- Decision: One `CapturePipeline` trait is the platform boundary. ListenOnly
  CGEventTap on a dedicated CFRunLoop thread with runtime
  `CGEventTapIsEnabled` verification. Continuous SCStream per active display;
  the buffered pre-event frame yields all three screenshot-triple artifacts.
  AX element resolution at the click point with a fixed-size fallback crop.
  Every key-down gets a full triple through a bounded async queue.
- Rationale: The recorder must never delay real input, and post-event capture
  can miss pre-action UI state.
- Rejected alternatives: Default (modifying) tap; on-demand post-event
  capture; rdev.
- Canonical docs: issue #6, `docs/adr/0001-pre-buffered-screen-capture.md`.

## DEC-002: Storage, schema v1, and command surface

- Status: accepted
- Decision: Per-workflow folder under app-data with a readable name.
  Append-only `events.jsonl` during capture; editable `workflow.json`
  manifest with `schema_version: 1` and steps referencing events via
  `event_ids`; `shots/` PNGs. `WorkflowStore` trait (`create`,
  `append_event`, `load`, `save_manifest`, `list`) with the JSON
  implementation. This bundle ships the capture-lifecycle commands:
  `check_permissions`, `request_permission(kind)`,
  `start_recording(name, channel)`, `stop_recording`, `list_workflows`,
  `get_workflow`. The step-edit commands belong to the review-UI capability.
- Rationale: Crash-safe capture writes, lossless events under editable
  steps, SQLite possible later behind one trait.
- Rejected alternatives: SQLite now; single mutable file; capture-time step
  persistence as the only record.
- Canonical docs: issue #7, issue #12 (bundle command scope).

## DEC-003: Live 1:1 parsing and auto-titles

- Status: accepted
- Decision: Each event becomes exactly one step immediately and streams over
  the capture channel. No re-parse, no synthetic `wait`/`assert` steps.
  Click -> `click`, key-down -> `type`. Titles:
  `Click "{element title | role | 'at (x, y)'}" — {app}` and
  `Press {character or key name}{" + modifiers" when present} — {app}`.
  Description defaults to empty.
- Rationale: Satisfies "understandable steps" inside the MVP line while the
  lossless log keeps grouping possible later.
- Rejected alternatives: Batch parsing at stop; burst collapsing in the MVP.
- Canonical docs: issue #10.

## DEC-004: KeySemantics classifier ownership

- Status: accepted
- Decision: One pure, stateless, unpersisted key-event classifier ships in
  the recording/parser core. Chord detection uses the semantic non-Shift
  modifier mask only, never timing. Auto-titles route through it; verdicts
  are never persisted in `events.jsonl`.
- Rationale: Both stretch capabilities need the same boundary rule; a
  capture-time verdict is policy, not fact.
- Rejected alternatives: Stretch-capability ownership; persisted verdicts.
- Canonical docs: issue #9, `docs/adr/0002-key-event-semantic-classifier.md`.

## DEC-005: Capability boundary and acceptance line

- Status: accepted
- Decision: This bundle is the first of four, strictly before the review UI.
  Its acceptance is the proven gate: a scripted recording across a native app
  and a Chromium app, typing with the recorder window focused and unfocused,
  event log plus triples on disk, element metadata on the native app,
  fallback crop on Chromium, inspected and accepted by the user. Only a bare
  dev-only trigger; the product UI is the next capability.
- Rationale: The gate disproves the tauri-apps/tauri#14770 class of failure
  before any UI investment.
- Rejected alternatives: Building the UI first; automated-only acceptance.
- Canonical docs: issue #6 (decision 8), issue #11, issue #12.

## DEC-006: Bundle identifier and signing (coordinator)

- Status: accepted
- Decision: Fixed bundle identifier `com.dpearson.workflow-step-editor`. Dev
  builds sign with "Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)".
- Rationale: TCC grants bind to bundle id plus signing identity; a fixed
  pair keeps grants across rebuilds. The identifier value is a
  non-consequential engineering choice; the constraint (fixed, signed) is
  the user's decision in issue #6.
- Rejected alternatives: Ad-hoc signing (unstable identity, re-prompts).
- Canonical docs: none.

## DEC-007: Mid-recording failure handling (coordinator)

- Status: accepted
- Decision: Fail-stop. If the tap is disabled, a stream fails, or a
  permission disappears mid-recording, stop the recording, keep the
  append-only `events.jsonl` and already-saved shots, and surface one error
  through the command/channel boundary. No automatic recovery UX in this
  bundle.
- Rationale: KISS inside the four-hour budget; the crash-safe log preserves
  user work; recovery UX has no recorded requirement.
- Rejected alternatives: Silent tap re-enable loops; recovery dialogs.
- Canonical docs: none.
## DEC-008: Key-down window and element resolution

- Status: accepted
- Decision: A key-down event resolves its window as the focused window of
  the frontmost application and its element as the system focused UI element
  (`AXFocusedUIElement`). The element crop is the focused element's frame
  cut from the buffered pre-event frame. When AX data is unavailable
  (Chromium apps, errors, implausible frames), the fallback is a fixed-size
  crop centered inside the focused window's bounds with
  `element.source: "fallback"`.
- Rationale: The element crop for a typing step should show the field the
  user types into; the mouse position is unrelated to typing focus. The
  grouping capability's first-event triple inherits this behavior.
- Rejected alternatives: Mouse-position hit test for key-downs; no element
  resolution for key-downs.
- Canonical docs: none; slice plan carries the behavior.

