# PR-01 Plan: Landing page over extended workflow summaries

## Outcome

The app opens on the product landing page: one row per saved workflow with a
real thumbnail, name, and `date · step count · duration`; a permission status
strip; a Record button gated on all three permissions with an explanatory
hint; hover Reveal-in-Finder per row; and row navigation into a detail shell
with `‹ Workflows` back. The dev-only capture trigger is replaced.

## Scope and Ownership

- Behavior: Extend the workflow-list summary with step count (from manifest
  steps), duration, and a thumbnail reference per DEC-006 — computed from
  stored artifacts at list time, newest first, with placeholder fallbacks
  for a damaged or missing event log; schema v1 files stay unchanged. Add
  the `reveal_workflow(id)` command (backend resolves and validates the
  folder through the store, then reveals it in Finder; no path crosses
  IPC). Add the scoped backend screenshot read per DEC-007 (workflow id +
  event id + allowlisted variant -> canonical PNG bytes; no asset-protocol
  root exposure; frontend caches blob URLs per view and revokes them on
  discard). Replace `src/App.tsx` with the product shell: a typed frontend
  API client as the only module touching Tauri `invoke`/`Channel`, a
  discriminated view reducer, the landing view per the pinned prototype
  direction (`prototype/map-1-8` variant D home), the permission strip
  over `check_permissions` / `request_permission`, Record gating with
  hint, and navigation into a detail shell (workflow name header plus back
  control; PR-02 fills the pane). Enlarge the default window in
  `src-tauri/tauri.conf.json` to fit the variant D composition. Introduce
  the frontend component-test harness (vitest, @testing-library/react,
  jsdom) used by this and later slices, and establish the
  repository-native UI-proof route (candidate direction: launch the
  locally built signed app — the signed artifact, not an unsigned debug
  binary, because macOS TCC binds permission grants to the signed bundle
  identity — and observe the proof target through unchanged production
  composition, for example through the app's macOS accessibility tree,
  emitting the typed `project-ui-proof` artifact; component tests do not
  satisfy this gate). Summary DTO guidance: duration is integer
  milliseconds and optional; the screenshot read is a command taking
  workflow id, event id, and variant and returning raw PNG bytes with
  string errors like the existing commands; blob URLs are revoked on view
  unmount as well as cache discard. README changes in this slice cover
  setup and command changes only; the final feature narrative belongs to
  PR-03.
- Owned paths: `package.json`, `package-lock.json`, `tsconfig.json`,
  `tsconfig.node.json`, `vite.config.ts`, `index.html`, `src`,
  `src-tauri/Cargo.lock`, `src-tauri/Cargo.toml`, `src-tauri/capabilities`,
  `src-tauri/src`, `src-tauri/tauri.conf.json`, `README.md`

## Slice Cohesion

- Primary outcome: The landing page lists real workflows and gates Record on
  permissions.
- Primary execution flow: App launch -> `list_workflows` summaries ->
  landing rows render with thumbnails -> Record gate reflects permissions ->
  row navigates to the detail shell and back.
- Owning observable seam: The rendered landing view over the production
  `list_workflows` summary data.
- Primary acceptance criterion: AC-002
- Regression guards: none
- New high-cost verification mechanism: frontend component-test harness
  plus the first repository UI-proof route
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: The landing page cannot render its decided row format
  without the summary extension and the screenshot display path; the
  extension has no consumer other than the landing view. They are
  inseparable at the rendered-landing seam.
- Path-count warning: none

## Non-Goals

- The detail review pane, step editing, and rename (PR-02).
- The record flow and draft states (PR-03); any deletion (PR-02).
- Schema changes to `events.jsonl` or `workflow.json`.

## Dependencies

- Slice dependencies: none
- Wave: 1
- Execution mode: serial

## Acceptance Coverage

- AC-002: This slice implements and proves the entire landing-page
  invariant — rows with thumbnail/name/`date · step count · duration`,
  permission strip, gated Record with hint, hover Reveal-in-Finder, and
  navigation both ways.

## Verification

- Rust: `cargo test` in `src-tauri` covering the summary extension (step
  count from manifest steps; zero-, one-, and multi-event duration;
  damaged-log placeholders keep the row listable), screenshot-read
  confinement (canonical ids/variants only, symlink rejection), and
  `reveal_workflow` validation (traversal, missing workflow, symlinked
  directory).
- Frontend: `npx vitest run` component tests for row rendering from summary
  fixtures (including placeholder fallbacks), Record gating on permission
  states, the hint, and reveal-versus-navigate row behavior.
- No per-row `get_workflow` calls from the landing page (summary data comes
  from `list_workflows` alone).
- Build: `npm run build` (tsc + vite) passes.
- UI-proof route established: the typed `project-ui-proof` artifact is
  produced from the launched real app, or the slice blocks explicitly.
- Independent command: `npm run build && (cd src-tauri && cargo test)`
- UI gate: snapshot_required_human_deferred
- Automated UI acceptance: AC-002
- UI proof target: landing-workflow-list
- Final UI slice: PR-03
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 high
- Selection predicates: multi-file backend/frontend integration; new
  verification mechanism (frontend test harness)
- Binding: Claude task adapter request
