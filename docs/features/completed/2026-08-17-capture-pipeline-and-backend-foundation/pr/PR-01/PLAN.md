# PR-01 Plan: Tauri scaffold, signed dev build, and permission commands

## Outcome

A runnable, signed Tauri v2 application shell with the permission module and
its two Tauri commands. The app builds with `npm run tauri build`, carries the
fixed bundle identifier, signs with the local Apple Development identity, and
reports and requests all three TCC permissions in the decided order.

## Scope and Ownership

- Behavior: Project scaffold (Vite + React + TypeScript, Rust backend in
  `src-tauri/`); fixed bundle identifier `com.dpearson.workflow-step-editor`;
  dev-build signing with "Apple Development: dpearson2699@gmail.com
  (86K7G9BGZ7)" injected through the local build environment so a clone
  without that certificate still builds with standard Tauri commands;
  permission module with kinds `input_monitoring`, `accessibility`,
  `screen_recording` and a status model rich enough to express
  "Accessibility deliberately not checked yet" (for example `granted`,
  `denied`, `not_requested`, `blocked_by_prerequisite`) — verify the
  native Input Monitoring status API first and let it fix the final shape;
  ordered status aggregation so no Accessibility API call happens before
  Input Monitoring has been requested, with permission operations
  serialized so concurrent commands cannot violate that order; an
  out-of-order `request_permission(accessibility)` returns
  `blocked_by_prerequisite` without touching the Accessibility API
  (DEC-011), covered by a direct out-of-order test; commands
  `check_permissions()` and `request_permission(kind)`; minimal README
  with setup commands, the macOS-only limitation, and the window-crop
  bounds caveat. This module is the sole production permission
  implementation for the bundle; later slices consume it unchanged.
- Owned paths: `package.json`, `package-lock.json`, `tsconfig.json`,
  `tsconfig.node.json`, `vite.config.ts`, `index.html`, `src/`, `src-tauri/`,
  `README.md`, `.gitignore`

## Slice Cohesion

- Primary outcome: A signed app shell whose permission commands report and
  request all three TCC permissions.
- Primary execution flow: App launch -> permission check commands ->
  system request prompts.
- Owning observable seam: The Tauri command layer (`check_permissions`,
  `request_permission`) over the permission module.
- Primary acceptance criterion: Explicit observable criterion — the signed
  build launches cleanly and `check_permissions` returns a status for each of
  the three permission kinds; `request_permission(kind)` triggers the
  matching system request path, except that an out-of-order request
  returns `blocked_by_prerequisite` without touching the Accessibility
  API.
- Regression guards: none
- New high-cost verification mechanism: none
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: The scaffold, signing configuration, and permission
  commands are one inseparable startup surface: TCC grants bind to the
  signed bundle identity, so permission behavior is only observable through
  the signed scaffold, and every later slice builds inside this project
  skeleton.
- Path-count warning: Large inventory is scaffold-generated boilerplate from
  `create-tauri-app`; behavior added by hand is the permission module and
  signing configuration only. Not a split signal.

## Non-Goals

- Event capture, screenshot capture, AX resolution, parsing, persistence.
- The recording lifecycle commands and the capture channel.
- Any product UI beyond the empty shell page.

## Dependencies

- Slice dependencies: none
- Wave: 1
- Execution mode: serial

## Acceptance Coverage

- Owns no acceptance criterion. It contributes the permission-module
  foundation and ordered request paths that the recording-gating criterion
  (owned by PR-02) completes, and the signed dev build the feature-owned
  proven gate runs on.

## Verification

- `npm run tauri build` succeeds and the bundle identifier and signing
  identity are visible in the built app (`codesign -dv`); the build also
  succeeds without the personal certificate present.
- `npm run tauri dev` starts cleanly.
- Focused Rust unit tests against a fake permission source: query order,
  first-launch/unknown Input Monitoring state, a spy assertion that no
  Accessibility call precedes the Input Monitoring request, one native
  request path per kind, invalid kind. The real TCC prompts are exercised
  manually through the feature-owned proven gate.
- A real built-app manual check invokes each request path in order (do
  not rely solely on `tauri dev`, whose TCC attribution is undocumented).
- Verify live crate versions on crates.io before locking dependencies
  (research observed 2026-08-16: tauri 2.11.5).
- Independent command: `npm run tauri build`
- UI gate: not_applicable
- Automated UI acceptance: none
- UI proof target: none
- Final UI slice: none
- Final design acceptance: none

## Implementation Route

- Requested model and effort: claude-fable-5 medium
- Selection predicates: localized known seams; decision-complete behavior;
  focused tests
- Binding: claude_task_request (Claude task adapter)

## Parallelization Assessment

- No same-wave pair: every wave in this bundle holds exactly one slice and
  runs serially, so no pair record applies.
