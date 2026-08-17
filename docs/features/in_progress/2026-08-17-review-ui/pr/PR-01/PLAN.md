# PR-01 Plan: Landing page over extended workflow summaries

## Outcome

The app opens on the product landing page: one row per saved workflow with a
real thumbnail, name, and `date · step count · duration`; a permission status
strip; a Record button gated on all three permissions with an explanatory
hint; hover Reveal-in-Finder per row; and row navigation into a detail shell
with `‹ Workflows` back. The dev-only capture trigger is replaced.

## Scope and Ownership

- Behavior: Extend the workflow-list summary with step count, duration, and a
  thumbnail reference computed from stored artifacts at list time — schema v1
  files stay unchanged. Add the `reveal_workflow(id)` command (reveal the
  workflow folder in Finder). Add a scoped mechanism for the webview to
  display stored screenshot files (Tauri asset protocol with a scoped
  capability, or a thin command returning image bytes — the implementer
  selects per DEC-004/GA-003; either way it must not expose paths outside
  the workflow root). Replace `src/App.tsx` with the product shell: the
  landing view per the pinned prototype direction (`prototype/map-1-8`
  variant D home), the permission strip over `check_permissions` /
  `request_permission`, Record gating with hint, and navigation into a
  detail shell (workflow name header plus back control; PR-02 fills the
  pane). Introduce the frontend component-test harness (vitest,
  @testing-library/react, jsdom) used by this and later slices.
- Owned paths: `package.json`, `package-lock.json`, `tsconfig.json`,
  `tsconfig.node.json`, `vite.config.ts`, `index.html`, `src`,
  `src-tauri`, `README.md`

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
  (vitest + testing-library)
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
- The record flow, draft states, and any deletion (PR-03).
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

- Rust: `cargo test` in `src-tauri` covering the summary extension
  (step count, duration, thumbnail selection) and `reveal_workflow`
  validation.
- Frontend: `npx vitest run` component tests for row rendering from summary
  fixtures, Record gating on permission states, and the hint.
- Build: `npm run build` (tsc + vite) passes.
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
- Binding: codex_task_request
