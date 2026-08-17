# PR-02 Plan: Recording domain core, persistence seam, and fake-driven recording coordinator

## Outcome

A deterministic, macOS-API-free backend that runs an entire recording
lifecycle against a fake `CapturePipeline`: start, receive capture packets,
persist screenshots and events through `WorkflowStore`, parse one step per
event, emit steps in order over the typed channel envelope, stop, and save a
schema v1 manifest. PR-03 then only supplies the real macOS adapter.

## Scope and Ownership

- Behavior: Schema v1 types with golden fixtures pinning the exact decided
  field shapes (issue #7 examples); `KeySemantics` (pure, stateless,
  unpersisted; chord = any held non-Shift semantic modifier; no timing
  rules); parser mapping one event to one step with auto-titles
  (`Click "OK" — TextEdit`, `Click at (512, 384) — TextEdit`,
  `Press H — Chrome`, `Press Cmd+S — TextEdit`) and classification defaults
  click -> `click`, key-down -> `type`; `WorkflowStore` trait (`create`,
  `append_event`, `load`, `save_manifest`, `list`) with the JSON
  implementation. `append_event` is the compound per-event persistence
  operation: it accepts the event plus its three screenshot payloads,
  writes the PNG files under `shots/`, then appends one JSONL line — no
  other module writes workflow data (AC-003 owning seam). `create` writes
  the readable per-workflow folder, `shots/`, and an initially valid empty
  v1 manifest so interrupted recordings stay listable. The store takes its
  root directory as a parameter (no ambient reads). This slice also owns
  the platform-independent recording orchestration: the `CapturePipeline`
  trait definition (start/stop; ordered capture packets carrying raw
  facts, resolved metadata, frame age, and one encoded screenshot triple),
  a deterministic fake pipeline, the recording coordinator state machine
  (`Idle -> Starting -> Recording -> Stopping/Failed -> Idle`, exactly one
  active recording, one terminal outcome), permission gating at start
  through the PR-01 permission seam, the capture-lifecycle application
  services with thin Tauri command wiring (`start_recording`,
  `stop_recording`, `list_workflows`, `get_workflow`), and the typed
  live-channel envelope: tagged `Step` items plus terminal `Stopped` and
  `Failed` variants, terminal-last ordering; a channel disconnect never
  interrupts disk persistence. Stop and fail-stop share one finalization
  owner: drain and join accepted capture work before the manifest saves
  and the terminal variant emits, ignore stale callbacks that arrive
  after finalization, define stop-during-start and concurrent
  stop-versus-fail outcomes (exactly one terminal), and remove the
  created folder when startup fails before the workflow ID publishes.
  `append_event` writes each PNG to a temporary file and renames it into
  place, then appends and flushes the complete JSONL line; the claim is
  process-level consistency, not power-loss durability. The store
  validates workflow IDs (no traversal), uses no-follow file operations
  confined to its root, creates folders and files owner-only, and `load`
  tolerates a torn final JSONL line by skipping it. The window field
  extends the schema's existing null pattern (DEC-011): when no window
  resolves (desktop clicks, focusless key-downs), the event records
  `window: null`, fixtures pin that shape, and the parser's `{app}` falls
  back to the frontmost application name, else `Unknown`.
  `start_recording` takes an optional name; a missing or blank name
  defaults to a timestamp (issue #7 decision 5). The coordinator and
  store take one injected wall-clock source — system in production,
  fixed in tests — so default names, folder names, and manifest
  timestamps are tested under fixed time. `create` returns a store-owned
  unpublished guard: dropping it before the workflow ID publishes rolls
  the folder back through the store, which is startup rollback, not
  user-facing workflow deletion. `KeySemantics` pins the
  exact semantic modifier mask (Command, Control, Option, Fn; Shift and
  lock keys excluded) and the normalized chord presentation order (Fn,
  Ctrl, Opt, Shift, Cmd around the key, Shift shown only inside a
  chord), with Fn-combination parser tests. Pinned invariant: a step is published to
  the channel only after its event line and all three screenshots are
  committed.
- Owned paths: `src-tauri/src/domain/`, `src-tauri/src/recording/`,
  `src-tauri/src/commands/`, `src-tauri/src/lib.rs`,
  `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`

## Slice Cohesion

- Primary outcome: A complete recording lifecycle turns capture packets
  into titled, classified, persisted schema v1 data and ordered live
  steps, proven deterministically.
- Primary execution flow: `start_recording` -> fake capture packet ->
  `KeySemantics`/parser -> `WorkflowStore.append_event` (shots then JSONL)
  -> channel step emission -> `stop_recording` manifest save.
- Owning observable seam: The capture-lifecycle command layer over the
  coordinator, exercised end to end with the fake `CapturePipeline`
  against a temporary directory.
- Primary acceptance criterion: AC-003
- Regression guards: AC-002, AC-004, AC-005
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Classifier, parser, store, coordinator, and channel are
  one execution flow over one shared schema: the coordinator's lifecycle
  is only observable through persisted store output and emitted steps, and
  the persistence-before-channel invariant can only be proven where all of
  them meet. Splitting them would ship a lifecycle with no observable
  output or a store with no production caller.
- Path-count warning: none

## Non-Goals

- Any macOS API call (CGEventTap, ScreenCaptureKit, Accessibility, TCC).
- Real frame acquisition or crop geometry; the fake pipeline supplies
  fixture PNG payloads.
- The real permission source (PR-01 owns it; this slice consumes the seam
  with a fake).
- Burst grouping, synthetic `wait`/`assert` steps, re-parse.

## Dependencies

- Slice dependencies: PR-01
- Wave: 2
- Execution mode: serial

## Acceptance Coverage

- AC-002: `KeySemantics` unit tests — plain key, Shift-only, one non-Shift
  modifier, multiple modifiers, repeated key-downs under a held modifier;
  serialization test asserts no verdict is persisted.
- AC-003: store-seam integration tests with golden schema v1 fixtures:
  folder layout, `shots/` PNG writes through `append_event`, JSONL
  append-only behavior (existing lines byte-identical after later
  appends), `event_ids` arrays, atomic manifest replacement, explicit
  unsupported-version error, failure injection (shot-write failure yields
  no JSONL line; append failure yields no channel step; orphan shots do
  not break load/list).
- AC-004: parser unit tests over synthetic click and key-down events —
  titled element, coordinate fallback, plain key, chord titles (including
  Fn combinations in Fn, Ctrl, Opt, Shift, Cmd order), defaults, empty
  description, and the null-window title fallbacks (frontmost app, then
  `Unknown`) — plus fake-pipeline channel tests proving one ordered step
  per event at the emission boundary.
- AC-005: coordinator tests against a fake permission source — start
  refuses unless all three permissions pass, concurrent starts yield
  exactly one success, stop without an active recording returns a defined
  error, pipeline startup failure returns to `Idle`, one fail-stop
  transition under simultaneous fake failures. The bound evidence is the
  workspace test run, which also executes PR-01's permission-ordering and
  `blocked_by_prerequisite` tests alongside this slice's gating and
  single-session tests.

## Verification

- `cargo test` in `src-tauri` covering the four criteria above.
- Golden serialization of the exact issue-#7 event and manifest examples;
  serde round-trips pin exact field names; null-window fixtures pin the
  DEC-011 shape.
- Two fake events yield two JSONL lines, six PNGs, two manifest steps, and
  two channel items in identical order; channel emission happens only
  after store commit.
- Deterministic tests for the accepted race, rollback, and confinement
  behavior: stop-during-start and concurrent stop-versus-fail yield
  exactly one terminal; startup failure before ID publication removes
  the folder; no-follow confinement rejects symlinked paths; created
  folders and files carry owner-only modes; blank or missing recording
  name takes the timestamp default.
- `cargo clippy` clean for touched code; `npm run tauri build` still
  succeeds.
- Independent command: `cargo test --manifest-path src-tauri/Cargo.toml`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: multi-file integration; significant edge cases
  (key-character mapping, chord titles, append-only JSONL layout,
  lifecycle state machine with fake concurrency)
- Binding: claude_task_request (Claude task adapter)

## Parallelization Assessment

- No same-wave pair: every wave in this bundle holds exactly one slice and
  runs serially, so no pair record applies.
