# Interview

## GA-001: Coverage of the accepted capability decisions

- Status: closed
- Kind: fact
- Uncertainty: Whether the wayfinder decision tickets settle the bundle's
  behavior, architecture, and acceptance, or whether gaps remain.
- Why it matters: Unsettled consequential decisions block Plan.
- Evidence inspected: Issue #12 (owning issue); decision records on #6, #7,
  #9, #10, #11; ADRs 0001 and 0002; `CONTEXT.md`; MuninnDB decisions in vault
  `workflow-step-editor`. All load-bearing decisions are recorded and mutually
  consistent. One gap found: key-down window/element resolution (GA-002).
- Confidence: high
- Question: none

## GA-002: Window and element resolution for key-down events

- Status: closed
- Kind: decision
- Uncertainty: Clicks resolve the window by point hit test and the element by
  `AXUIElementCopyElementAtPosition` at the click point. A key-down has no
  point. The decision records fix a full screenshot triple and element
  metadata per event but never state how a key-down resolves its window and
  element.
- Why it matters: It decides what the element crop shows for every typing
  step, and the grouping capability (#14) later reuses the first event's
  triple. Wrong artifacts require re-recording.
- Evidence inspected: #6 decision 7 (triple per key-down), #7 schema v1
  (`window` and `element` fields uniform per event), #10 (auto-titles use the
  app name), #9 (first-event triple for groups). None names the key-down
  resolution path.
- Confidence: high
- Question: Q-001

## GA-003: UI acceptance policy classification

- Status: closed
- Kind: decision
- Uncertainty: Whether this bundle is UI-affecting and needs
  `final_pr_design_gate`.
- Why it matters: The policy adds a final UI slice and a human design
  checkpoint.
- Evidence inspected: Issue #11 decision 2 assigns the entire product UI to
  the review-UI capability (#13) with its own acceptance line; #11 decision 1
  and #12 scope allow only "a bare dev-only trigger" here and state "the
  product UI is the next capability". The bundle's human acceptance is the
  proven gate (file inspection), not a design checkpoint. Classified
  non-UI-affecting; the UI Acceptance Policy section is omitted per the
  bundle contract.
- Confidence: high
- Question: none

## GA-004: External crate and API facts

- Status: closed
- Kind: fact
- Uncertainty: Which crates and macOS APIs implement the accepted
  architecture.
- Why it matters: Wrong bindings cost rework inside a four-hour budget.
- Evidence inspected: Research records #2 (core-graphics 0.25.0 CGEventTap,
  dedicated CFRunLoop thread, rdev rejected, permission prompt-order caveat),
  #3 (screencapturekit crate 8.0.1, `CGWindowListCopyWindowInfo` not
  deprecated, xcap rejected), #4 (accessibility-sys 0.2.0 / axuielement
  0.9.1, fallback crop ~300x200 pt), #5 (create-tauri-app scaffold, Tauri
  Channels for streaming, signing for TCC persistence). Live-observed
  2026-08-16. Named open gaps (for example the axuielement hit-test API
  shape) are Plan-phase verification items, not user decisions.
- Confidence: high
- Question: none

## GA-005: Bundle identifier and signing identity

- Status: closed
- Kind: fact
- Uncertainty: Which fixed bundle identifier and signing identity dev builds
  use.
- Why it matters: TCC grants bind to bundle id plus signing identity; churn
  re-triggers permission prompts.
- Evidence inspected: #6 facts record the local identity "Apple Development:
  dpearson2699@gmail.com (86K7G9BGZ7)" and the fixed-identifier requirement.
  The identifier value itself is an engineering choice; DEC-006 records the
  coordinator selection.
- Confidence: high
- Question: none

## GA-006: Mid-recording failure handling depth

- Status: closed
- Kind: decision
- Uncertainty: Behavior when the tap is silently disabled, a stream fails, or
  a permission is revoked mid-recording.
- Why it matters: Determines error-path scope inside the four-hour budget.
- Evidence inspected: #2 records the silent-disable risk and the
  `CGEventTapIsEnabled` runtime check; no record demands recovery UX. The
  KISS default is fail-stop: stop the recording, keep `events.jsonl` (it is
  append-only and crash-safe per #7), surface one error to the caller.
  Recorded as DEC-007; not consequential enough to ask.
- Confidence: medium
- Question: none

## Q-001: What do a key-down's window and element metadata point at?

- Status: answered
- Recommendation: Resolve the focused UI element. Window = the focused
  window of the frontmost application; element = the system focused element
  (`AXFocusedUIElement`). The element crop is the focused element's frame cut
  from the buffered frame. Fallback (no AX data, Chromium apps): fixed-size
  crop centered inside the focused window's bounds, `source: "fallback"`.
  This shows the field the user types into, which is what a workflow reviewer
  needs, and it is what the grouping capability's first-event triple will
  inherit.
- Options and tradeoffs:
  1. Focused-element resolution (recommended): correct semantics for typing;
     one extra AX call path (focused element instead of point hit test).
  2. Mouse-position hit test for all events: uniform code path, but the crop
     shows whatever the cursor happens to hover, which is wrong whenever the
     user tabs between fields or moved the mouse away.
  3. No element resolution for key-downs: always a fixed-size crop at the
     window center; simplest, but drops real element metadata that native
     apps can provide, and weakens the proven gate's "element metadata on
     the native app" check for typing events.
- If wrong: every typing step carries a misleading element crop; artifacts
  can only be fixed by re-recording; grouping (#14) inherits the flaw.
- Answer/source: User selected option 1, focused-element resolution
  (interview answer, 2026-08-17).
- Closure reason: Consequential capture semantics decided by the user.
- Decision: DEC-008
- Canonical-doc impact: none; behavior recorded in DEC-008, FEATURE.md
  scope, and the owning slice plan.
