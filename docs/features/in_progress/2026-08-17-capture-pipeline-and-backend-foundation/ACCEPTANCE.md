# Acceptance

## AC-001: Proven capture gate

- Ownership: feature
- Invariant: With the signed dev build, one scripted manual sequence recorded
  across a native app (TextEdit or Finder) and a Chromium app (Chrome or VS
  Code), including typing while the recorder window is focused and while it
  is unfocused, produces on disk: `events.jsonl` with one line per event,
  three screenshot PNGs per event under `shots/`, element metadata with
  `source: "ax"` on the native app, and fallback element crops on the
  Chromium app. This disproves the tauri-apps/tauri#14770 class of failure
  for the raw CGEventTap path.
- Owning seam: The recorded workflow folder on disk, inspected by the user.
- Evidence required: The user inspects the recorded files and accepts.

## AC-002: Key-event classification invariant

- Ownership: slice
- Invariant: `KeySemantics` is pure, stateless, and unpersisted. It
  classifies a key-down as a shortcut chord exactly when a non-Shift
  semantic modifier is held, with no timing rules; holding one modifier
  through repeated key-downs yields one chord verdict per key-down. Auto
  titles route through it, and no classifier verdict is written to
  `events.jsonl`.
- Owning seam: The classifier's public API in the recording/parser core.
- Evidence required: Focused unit tests: plain key, Shift-only key,
  modifier chord, repeated key-downs under a held modifier.

## AC-003: Schema v1 persistence invariant

- Ownership: slice
- Invariant: During capture the store appends exactly one JSON line per
  event to `events.jsonl`. `stop_recording` saves `workflow.json` with
  `schema_version: 1` and steps referencing events via `event_ids` arrays.
  Each event's three screenshots persist as PNGs under `shots/` at the paths
  the event line records. All persistence goes through the `WorkflowStore`
  trait, into one per-workflow folder with a readable name under the
  app-data directory.
- Owning seam: The `WorkflowStore` JSON implementation against a temporary
  directory.
- Evidence required: Integration tests at the store seam with
  schema-v1-shaped fixture events, asserting file layout, JSONL
  append-only behavior, and manifest shape.

## AC-004: Live 1:1 parsing and auto-titles

- Ownership: slice
- Invariant: Each captured event parses into exactly one step immediately,
  emitted at the step-emission boundary the capture channel consumes.
  Classification defaults are click -> `click` and key-down -> `type`; no
  synthetic `wait` or `assert` steps.
  Titles follow the decided formats: `Click "OK" — TextEdit`,
  `Click at (512, 384) — TextEdit` (fallback), `Press H — Chrome`,
  `Press Cmd+S — TextEdit`. Description defaults to empty.
- Owning seam: Parser output at the capture-channel payload boundary.
- Evidence required: Unit tests on the parser with synthetic click and
  key-down events covering titled elements, fallback coordinates, plain
  keys, and modifier chords.

## AC-005: Permission gating and command surface

- Ownership: slice
- Invariant: `check_permissions` reports each of Input Monitoring,
  Accessibility, and Screen Recording; `request_permission(kind)` triggers
  the matching system request, except that a request whose ordering
  prerequisite has not yet been requested returns a
  `blocked_by_prerequisite` status without touching the Accessibility
  API. No Accessibility API call happens before Input Monitoring has been
  requested (prompt-suppression caveat).
  `start_recording` refuses to start unless all three permissions pass, and
  exactly one recording can be active at a time.
- Owning seam: The permission module and the Tauri command layer.
- Evidence required: Focused tests where the seam is fakeable (gating and
  single-active-recording logic against a faked permission source), plus
  the AC-001 manual run for the real TCC prompts and ordering.
