# PR-02 Plan: Recording domain core — KeySemantics, parser, schema v1 store

## Outcome

The pure Rust recording domain core: schema v1 event and manifest types, the
`KeySemantics` key-event classifier, the live 1:1 parser with auto-titles,
and the `WorkflowStore` trait with its JSON filesystem implementation. All of
it is macOS-API-free and fully covered by deterministic tests.

## Scope and Ownership

- Behavior: Schema v1 types serializing to the decided `events.jsonl` and
  `workflow.json` shapes; `KeySemantics` (pure, stateless, unpersisted;
  chord = any held non-Shift semantic modifier; no timing rules); parser
  mapping one event to one step with auto-titles
  (`Click "OK" — TextEdit`, `Click at (512, 384) — TextEdit`,
  `Press H — Chrome`, `Press Cmd+S — TextEdit`) and classification defaults
  click -> `click`, key-down -> `type`; `WorkflowStore` trait (`create`,
  `append_event`, `load`, `save_manifest`, `list`) with the JSON
  implementation writing one readable-named folder per workflow, append-only
  `events.jsonl`, `workflow.json`, and `shots/` paths. The store takes its
  root directory as a parameter (no ambient reads).
- Owned paths: `src-tauri/src/domain/`, `src-tauri/src/lib.rs`,
  `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`

## Slice Cohesion

- Primary outcome: Raw events become titled, classified, persisted schema v1
  data through one pure domain core.
- Primary execution flow: Synthetic event -> `KeySemantics` classification ->
  parsed step with auto-title -> `WorkflowStore` persistence as schema v1
  files.
- Owning observable seam: The domain core's public Rust API exercised end to
  end at the `WorkflowStore` seam against a temporary directory.
- Primary acceptance criterion: AC-003
- Regression guards: AC-002, AC-004
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Classifier, parser, and store are one execution flow over
  one shared schema: the classifier exists only to title parsed steps, the
  parser's output is exactly what the store persists, and all three are
  proven together at the store seam. Splitting them would ship schema types
  with no consumer or a parser with no observable output.
- Path-count warning: none

## Non-Goals

- Any macOS API call (CGEventTap, ScreenCaptureKit, Accessibility, TCC).
- PNG encoding of real frames; the store only records shot paths given to it.
- Recording lifecycle, commands, channel wiring, dev trigger.
- Burst grouping, synthetic `wait`/`assert` steps, re-parse.

## Dependencies

- Slice dependencies: PR-01
- Wave: 2
- Execution mode: serial

## Acceptance Coverage

- AC-002: `KeySemantics` unit tests — plain key, Shift-only, modifier chord,
  repeated key-downs under a held modifier; no verdict persisted in
  `events.jsonl`.
- AC-003: store-seam integration tests with schema-v1-shaped fixture events
  asserting folder layout, JSONL append-only behavior, manifest shape, and
  `event_ids` references.
- AC-004: parser unit tests over synthetic click and key-down events —
  titled element, coordinate fallback, plain key, chord; classification
  defaults; empty description.

## Verification

- `cargo test` in `src-tauri` covering the three criteria above.
- `cargo clippy` clean for touched code; `npm run tauri build` still
  succeeds.
- Serde round-trip tests pin the exact decided field names of schema v1.
- Independent command: `cargo test --manifest-path src-tauri/Cargo.toml`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: multi-file integration; significant edge cases
  (key-character mapping, chord titles, append-only JSONL layout)
- Binding: codex_task_request

## Parallelization Assessment

- No same-wave pair: every wave in this bundle holds exactly one slice and
  runs serially, so no pair record applies.
