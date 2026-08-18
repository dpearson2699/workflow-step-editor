# PR-03 Plan: Record flow and draft save/discard

## Outcome

Record on the landing page enters a live capture view whose only visible
action is a prominent Stop Recording banner while step rows stream in; Stop
enters draft review (`draft` badge, full editing, Discard with confirmation,
Save…); Save opens the naming dialog with a pre-selected timestamp default
and names the workflow. The app is demoable end to end.

## Scope and Ownership

- Behavior: Extend the live channel's step payload with the event timestamp
  (DEC-009). Build the record flow per the pinned prototype and
  DEC-002/DEC-005/DEC-009: recording mode driven by the retained channel
  and a frontend session token (rows stream in ordered and deduplicated by
  step id; stale-session messages are ignored; steps arriving before the
  start promise resolves are kept; a terminal received while start is
  pending wins; a Stop click during startup latches and issues once start
  resolves), the Stop Recording banner as the sole visible action, draft
  review as UI-session state (badge, full editing, Discard confirmation,
  Save…), the naming dialog pre-selecting the manifest's existing default
  name (no separate frontend timestamp), Save renaming via
  `rename_workflow` and exiting draft mode, and Discard deleting the draft
  folder through PR-02's `delete_workflow` and returning to the landing
  page. Draft exits only on command success: a failed Save keeps the
  draft state and naming dialog error visible, and a failed Discard keeps
  the draft state and its data with the error visible. A recording that
  ends in failure enters draft review with an error banner when its
  workflow still loads (the banner states the recording failed and may be
  incomplete); the frontend decides reviewability by loading the
  workflow, not by extra envelope fields, and a load failure surfaces the
  error on the landing page.
  Add the bundle-qualified final-gate material under `dev/review-ui-gate/`
  (the scripted final human loop for AC-001 on the locally built signed
  app — macOS TCC binds permission grants to the signed bundle identity)
  and add a one-line header note to `dev/proven-gate/script.md` naming it
  as the capture-pipeline bundle's historical gate so its AC-001/PR-03
  labels cannot be confused with this bundle's.
  Complete the final product documentation (README feature summary and
  usage walkthrough) against the actually shipped behavior.
- Owned paths: `README.md`, `dev`, `src`, `src-tauri/src`

## Slice Cohesion

- Primary outcome: A recording can be captured live, reviewed as a draft,
  and saved with a name or discarded.
- Primary execution flow: Record -> live rows stream over the channel ->
  Stop -> draft review -> Save names the workflow (or Discard removes its
  folder) -> landing list reflects stored state.
- Owning observable seam: The review UI over the production
  `start_recording` channel, `stop_recording`, `rename_workflow`, and
  `delete_workflow` commands.
- Primary acceptance criterion: AC-004
- Regression guards: AC-002, AC-003, AC-005
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: Record, live streaming, draft review, Save, and Discard
  are one user flow at one seam — the recording lifecycle observed through
  the review UI. The only backend change is the additive timestamp field
  that the flow's own rows require; every other command this flow consumes
  already exists. The substantial-semantics predicate reads `no` because
  this slice adds no production machinery beyond its own accepted UI
  outcome — no schema, storage, service, or command semantics — and in an
  all-UI capability the last UI feature must ship in the last UI slice;
  an additional empty final slice would carry only ceremony.
- Path-count warning: none

## Non-Goals

- The saved-workflow Delete… control and the removal primitive (PR-02;
  Discard consumes them unchanged).
- Any trash, restore, audit, or purge lifecycle; forensic erasure claims.
- Text-input grouping, keyboard shortcuts, re-parse, auto wait detection.
- Changes to capture, parsing, or schema v1.

## Dependencies

- Slice dependencies: PR-02
- Wave: 3
- Execution mode: serial

## Acceptance Coverage

- AC-004: This slice implements and proves the record flow — live
  streaming, Stop banner as sole action, draft badge, Discard confirmation,
  Save naming dialog with the pre-selected default, folder removal on
  Discard.

## Verification

- Rust: `cargo test` in `src-tauri` covering the timestamp field on the
  live step payload; existing capture ordering, terminal-last, fail-stop,
  channel, and deletion tests stay green.
- Frontend: `npx vitest run` component tests for the record-flow state
  transitions (double start/stop gating, startup Stop latching, early
  step and terminal arrival, stale-session suppression, both orders of
  stop-command resolution versus terminal-envelope arrival), the naming
  dialog preselecting the manifest default, the Discard confirmation,
  Save and Discard failure keeping draft state with a visible error, the
  failed-recording draft path, and the load-failure landing-page error
  path.
- Build: `npm run build` passes.
- Independent command: `npm run build && (cd src-tauri && cargo test)`
- UI gate: final_human_required
- Automated UI acceptance: AC-004
- UI proof target: stop-recording-banner
- Final UI slice: PR-03
- Final design acceptance: AC-001

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: asynchronous live-channel state with significant
  ordering edge cases; final-slice UI proof obligations
- Binding: Claude task adapter request
