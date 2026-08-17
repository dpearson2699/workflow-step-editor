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
  `Press {modifier prefixes}{character or key name} — {app}`, with
  modifier prefixes joined by `+` in the order Fn, Ctrl, Opt, Shift, Cmd
  when present — the form the accepted examples use
  (`Press Cmd+S — TextEdit`). Description defaults to empty.
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


## DEC-009: Queue-saturation fail-stop

- Status: accepted
- Decision: When the bounded capture queue between the ListenOnly tap and
  the screenshot worker saturates, the recording fail-stops with one
  explicit capture-overloaded error. Every event and screenshot already
  committed is preserved. The tap callback never blocks, and no event is
  silently dropped or coalesced.
- Rationale: The per-event screenshot-triple guarantee is literal
  (AC-001); silent loss would pass superficial checks while violating it,
  and blocking the tap risks timeout-driven disablement. Extends DEC-007's
  fail-stop posture to overload.
- Rejected alternatives: Silent drop with a degraded marker; blocking the
  tap; unbounded queue.
- Canonical docs: none; the PR-03 plan carries the tested invariant.

## DEC-010: Non-UI classification of the dev-only trigger

- Status: accepted
- Decision: This bundle is classified non-UI-affecting. The bare dev-only
  trigger (start/stop control with minimal live output) is developer
  scaffolding outside product UI; the bundle declares no UI Acceptance
  Policy, and every slice uses `UI gate: not_applicable`. The proven gate
  (AC-001) is the bundle's human acceptance. The review-UI capability
  (issue #13) owns the product UI and its design gate.
- Rationale: The user's capability split (#11, #12) excluded product UI
  from this bundle; Tauri's UI automation driver does not support macOS,
  so a truthful automated UI proof route is not available inside the
  budget. Decided by the user as interview Q-003 after the plan-consensus
  reviewer raised the contract-literal reading.
- Rejected alternatives: `final_pr_design_gate` on PR-03 with an
  automated trigger proof and final human design acceptance.
- Canonical docs: none.

## DEC-011: Command and schema edge-case clarifications (coordinator)

- Status: accepted
- Decision: Four clarifications of unaddressed edge cases inside accepted
  contracts. (1) `request_permission(kind)` returns
  `blocked_by_prerequisite` without touching the Accessibility API when
  the ordering prerequisite (Input Monitoring requested) is not yet
  satisfied; AC-005 carries the matching clause. (2) The schema's
  existing null pattern for inapplicable fields (`key: null` on clicks,
  `button: null` on key-downs) extends to `window: null` when no window
  resolves (desktop clicks, focusless key-downs); the named fields from
  issue #7 are unchanged. For a null window: the display is the one
  containing the click point, or the main display for key-downs; the
  window crop is the full display frame; the element crop is the
  fixed-size fallback centered at the click point or display center; the
  serialized element records `role: null`, `title: null`, the fallback
  crop rectangle as `frame`, and `source: "fallback"`; the title's
  `{app}` is the frontmost application name, else `Unknown`.
  (3) `start_recording` takes an optional name; a missing or blank name
  defaults to a timestamp, per issue #7 decision 5. (4) `KeySemantics`
  chord presentation order is Fn, Ctrl, Opt, Shift, Cmd around the key.
- Rationale: Each case was unaddressed by the decision records; the
  clarifications keep every user-fixed field and invariant unchanged and
  were surfaced by the cross-model plan consensus loop.
- Rejected alternatives: Synthetic window metadata for unresolvable
  windows; refusing out-of-order permission requests with an opaque
  error; making the name mandatory.
- Canonical docs: none.
