# Plan Consensus Log

- Counterparty model: gpt-5.6-sol (codex exec, model_reasoning_effort=xhigh, sandbox read-only)
- MAX_ROUNDS: 5
- Counterparty thread id: codex thread 01a00ff0-07c6-7b32-978f-52876225deaf

## Round 1 — gpt-5.6-sol (xhigh, read-only)

The plan is not ready for implementation. I found these material defects.

1. The scope exceeds the hard four-hour limit in [PROJECT_GOAL.md](/Users/dpearson/repos/workflow-step-editor/docs/PROJECT_GOAL.md:15). It includes three native frameworks, hot-plug recovery, persistence, parsing, signing, and three reviewed PRs.  
   Fix: Return to Discuss and obtain a user-approved, timed must-have cut before Delivery.

2. Required Plan probes remain unresolved. PR-01 defers permission status shape, while [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:107) defers AX binding and frame ownership.  
   Fix: Complete the live probes now, then freeze one dependency set and exact interfaces.

3. PR-03 adds visible controls and displayed values but declares `not_applicable` at [line 134](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:134). The UI contract covers all displayed controls and values.  
   Fix: Split a thin final UI slice and add `final_pr_design_gate`, automated acceptance, and human design acceptance.

4. PR-03 says AC-001 runs after merge at [line 130](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:130). The lifecycle requires non-design acceptance before the final snapshot and merge.  
   Fix: Run AC-001 against the exact final PR head before snapshot freeze and review.

5. All plans request `claude-fable-5` but declare `codex_task_request`. That combination has no valid route.  
   Fix: Use the Claude task adapter binding throughout the plans and final proposal.

6. Permission ownership conflicts across slices. PR-01 owns native requests, but [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:31) claims the real source. Concurrent commands can also violate request ordering.  
   Fix: Keep the production adapter in PR-01 and serialize permission operations behind one prerequisite state.

7. The pre-event guarantee is unproven. The broker does not compare SCStream `displayTime` with the CGEvent timestamp.  
   Fix: Keep a timestamped frame ring and select the newest valid frame not later than each event.

8. Window and AX lookup occur after event delivery and behind PNG work. Menus or focus can change first.  
   Fix: Snapshot event identity before enqueue, resolve metadata before encoding, and reject stale PID or generation results.

9. The frame lease omits valid statuses, `contentRect`, `screenRect`, scaling metadata, and a display generation.  
   Fix: Bind these fields to each lease and test mixed-scale displays plus atomic display replacement.

10. The recording state machine in [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:31) has no stop linearization. Accepted jobs or stale callbacks can write into a later workflow.  
    Fix: Bind a session generation; drain and join all owners before saving, emitting terminal status, and entering `Idle`.

11. The channel envelope carries steps only, although fail-stop requires one terminal error. Disconnect behavior is undefined.  
    Fix: Define tagged `Step`, `Stopped`, and `Failed` envelopes, terminal-last ordering, and a disk-safe disconnect policy.

12. The crash-safe claim is unsupported. JSONL can tear, and `workflow.json` remains empty until stop. No re-parse path recovers those steps.  
    Fix: Add temporary PNG publication, flushed records, torn-tail recovery, and atomic manifest persistence after each committed event.

13. A full data queue cannot carry its own failure signal. Retained high-resolution frames also need a memory-based bound.  
    Fix: Select capacity from a byte budget and use one out-of-band atomic terminal signal.

14. `get_workflow(id)` and screenshot paths lack confinement rules. The stored files contain global keys and screen content.  
    Fix: Validate internal IDs, reject traversal and symlinks, and create owner-only folders and files.

15. [AC-001](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/ACCEPTANCE.md:6) requires Chromium fallback, but Chromium can return valid AX elements. The manual gate is nondeterministic.  
    Fix: Name a reproducible inaccessible surface, or return the fallback proof requirement to Discuss.

16. PR-03 leases whole `recording/`, `commands/`, and `src/` trees while forbidding coordinator-contract changes.  
    Fix: Limit PR-03 to `capture/` and named composition files; give the final UI slice exact frontend paths.

17. Signed proof identity is ambiguous. PR-01 checks a built app, while PR-03 calls the gate a signed dev build.  
    Fix: Name one exact signed artifact containing the trigger, and test certificate-free clone builds separately.

VERDICT: REVISE
### Root arbitration (round 1)

- 1 Scope cut: REJECTED. The user accepted this exact capability scope and
  order at map exit (issue #11) with the four-hour constraint in view, and
  gates delivery after every bundle. Re-opening Discuss would re-litigate a
  settled user decision.
- 2 Resolve probes during Plan: PARTIALLY REJECTED. Version facts were
  live-verified in research (#2-#5). The remaining API-shape proofs need
  real compilation, so they stay the named first implementation step of
  PR-03 behind the trait seam; freezing exact interfaces now would cost a
  spike without reducing risk the seam does not already isolate.
- 3 UI design gate: REJECTED. GA-003 classified the bundle non-UI-affecting
  on the user's #11/#12 decisions: the dev-only trigger is excluded from
  product UI and the review-UI capability owns the design gate.
- 4 AC-001 timing: ACCEPTED. The proven gate must run on the signed build
  at the exact final PR head before snapshot freeze, review, and merge.
  Plans corrected; the criterion text itself never said "after merge".
- 5 Binding mismatch: ACCEPTED. All plans now bind claude_task_request
  (Claude task adapter), matching the requested claude-fable-5 routes.
- 6 Permission ownership: ACCEPTED as wording. PR-01 owns the sole
  production permission module, serialized behind ordered aggregation;
  PR-03 consumes it unchanged and adds no second implementation.
- 7 Pre-event frame selection: ACCEPTED. The broker keeps timestamped
  frames and selects the newest frame not later than the event timestamp;
  frame_age_ms derives from that comparison.
- 8 Resolution race: PARTIALLY ACCEPTED. The worker resolves metadata
  before encoding and snapshots event identity at enqueue. Stale-PID or
  generation rejection machinery is REJECTED as speculative for the MVP:
  a small async-resolution window is inherent to the accepted nonblocking
  tap and carries no data-loss risk.
- 9 Lease geometry: ACCEPTED. Leases carry display frame geometry and
  scale metadata; mixed-scale and replacement tests already planned.
- 10 Stop linearization: ACCEPTED. One finalization owner; drain and join
  before manifest save and terminal emission; stale post-stop callbacks
  are ignored.
- 11 Envelope terminals: ACCEPTED. Tagged Step plus terminal
  Stopped/Failed variants, terminal-last ordering; channel disconnect
  does not interrupt disk persistence.
- 12 Crash safety: PARTIALLY ACCEPTED. Store load tolerates a torn final
  JSONL line. Per-event manifest rewrites are REJECTED: the lossless raw
  log plus manifest-at-stop is the user-accepted schema decision (#7,
  #10), and re-parse is recorded future work.
- 13 Byte-budget capacity and out-of-band terminal signal: ACCEPTED as
  one-line implementation guidance in PR-03.
- 14 Confinement: ACCEPTED minimally. The store validates workflow IDs
  against its own listing (no traversal or symlinks) and writes only
  under its app-data root.
- 15 Chromium fallback determinism: REJECTED as a Discuss trigger. The
  deterministic implausible-frame rule makes coarse Chromium containers
  take the fallback path; the gate script targets a coarse surface. The
  user personally inspects the gate and can rule on any residual
  divergence there.
- 16 PR-03 path lease: ACCEPTED. PR-03 no longer owns
  src-tauri/src/recording/; it keeps capture/, composition files,
  commands wiring, and the dev trigger.
- 17 Gate artifact identity: ACCEPTED. The gate artifact is the locally
  built signed .app with the injected identity at the final PR head;
  certificate-free clone builds are verified separately in PR-01.

## Round 2 — gpt-5.6-sol (xhigh, read-only)

The revised files contain many accepted changes. However, the plan still has material defects.

1. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:8) still says AC-001 runs after merge. It later says before merge. Snapshot materialization also creates a later PR head.  
   Fix: Run AC-001 on the final code head, then permit only the authenticated snapshot commit before review.

2. The dev trigger has visible controls and values, yet [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:39) marks UI proof not applicable. “Dev-only” does not override the [UI contract](/Users/dpearson/repos/workflow-step-editor/.agents/workflows/spec-work-orchestrator/references/ui-gate-ownership.md:27).  
   Fix: Make PR-03 the final UI slice and add automated UI and final human acceptance.

3. AC-005 has no consistent owner. [PR-01](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-01/PLAN.md:76) names PR-03, while PR-03 owns no criterion. PR-02 treats AC-005 as a guard.  
   Fix: Split permission behavior and coordinator gating into criteria with one named producer each.

4. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:21) writes final PNG paths before JSONL. It defines no `fsync` or atomic PNG publication. A crash can leave committed lines referencing corrupt files.  
   Fix: Write, flush, and rename PNGs atomically; then append and synchronize the complete JSONL record.

5. PR-03 can change Cargo and Tauri configuration after PR-01 proves certificate-free builds. PR-03 repeats only the signed build.  
   Fix: Repeat signed and certificate-free builds at the final code head.

6. The tap callback owns enqueue, so the worker cannot snapshot event identity “at enqueue.” Key-down display selection also needs focused-window resolution first.  
   Fix: Copy immutable event fields in the callback, then resolve the key-down display before frame selection.

7. Frame comparison defines no shared clock for CGEvent and ScreenCaptureKit timestamps. Numerically ordered values can represent different time domains.  
   Fix: Convert both timestamps to one monotonic host clock and test delayed callbacks and equal timestamps.

8. Display replacement keeps old leases without defining current-generation selection. A reused display ID can attach stale geometry to a new event.  
   Fix: Publish display generations atomically and restrict new events to warmed frames from the current generation.

9. The claimed Chromium gate script is absent. [AC-001](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/ACCEPTANCE.md:6) still names only a general Chromium app. Valid Chromium AX elements can prevent fallback.  
   Fix: Name and verify one reproducible Chromium surface that triggers the fallback rule.

10. The schema requires window metadata, but no rule handles missing windows or focus. Desktop and windowless-app events can lack usable bounds.  
    Fix: Define schema-valid window and crop fallbacks, then test desktop clicks and focusless key-downs.

11. Listing-based ID validation does not prevent symlink time-of-check races. The plan also omits owner-only permissions for sensitive capture files.  
    Fix: Use root-relative no-follow file operations and create directories and files with owner-only permissions.

12. Coordinator tests omit stop-versus-fail and stop-during-start races. Pipeline startup failure can also leave an empty published workflow.  
    Fix: Define both linearization outcomes and use two-phase startup with rollback of unpublished workflows.

13. Issue #12 requires an exact modifier mask and normalized chord representation. The plan defines neither the modifier set nor output ordering.  
    Fix: Freeze the accepted modifier set, ordering, casing, and Shift-only title behavior in tests.

14. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:42) owns `commands/` while promising an unchanged PR-02 command layer.  
    Fix: Remove that directory from PR-03 ownership or name only the required composition file.

15. The [four-hour limit](/Users/dpearson/repos/workflow-step-editor/docs/PROJECT_GOAL.md:15) still has no timed budget or stop boundary. Prior capability acceptance does not satisfy this execution constraint.  
    Fix: Obtain a timed scope cut that prioritizes must-have capture before optional parsing and display recovery.

VERDICT: REVISE
### Root arbitration (round 2)

- 1 Residual after-merge wording: ACCEPTED. PR-03's Outcome now states the
  gate runs on the final code head before snapshot materialization,
  review, and merge, with only the authenticated snapshot commit after.
- 2 UI gate: ESCALATED TO THE USER as Q-003 (the finding would change
  acceptance structure). The user decided: keep the non-UI
  classification; DEC-010 records it with rationale (dev trigger outside
  product UI per #11/#12; tauri-driver does not support macOS). Closed.
- 3 AC-005 owner: ACCEPTED as a stale-wording fix; PR-01 now names PR-02.
  Splitting AC-005 into two criteria is REJECTED: one owner (PR-02) is
  now consistent, and criterion churn adds no coverage.
- 4 PNG atomicity: ACCEPTED. Temp-write plus rename per PNG, then append
  and flush the JSONL line; the claim is process-level consistency, not
  power-loss durability.
- 5 Cert-free build at final head: ACCEPTED into PR-03 verification.
- 6 Enqueue wording and key-down display order: ACCEPTED. The callback
  copies immutable fields plus the frame-set handle; the worker resolves
  the focused window before selecting that display's frame.
- 7 Clock domains: ACCEPTED. Both timestamps convert to one monotonic
  host clock, with delayed-callback and equal-timestamp tests.
- 8 Display generations: ACCEPTED. Per-generation display sets published
  atomically; new events select only current-generation warmed frames.
- 9 Chromium surface: ACCEPTED as a named script target (Chrome's
  rendered web-content area; coarse AXWebArea triggers the
  implausible-frame rule). Changing AC-001 remains rejected.
- 10 Missing window/focus: ACCEPTED. `window` is nullable for desktop
  clicks and focusless key-downs; fixtures pin the shape; the fallback
  crop rule covers the missing-window case.
- 11 TOCTOU and permissions: ACCEPTED. No-follow operations confined to
  the store root; owner-only folders and files.
- 12 Coordinator races: ACCEPTED. Stop-during-start and stop-versus-fail
  produce exactly one terminal; startup failure before ID publication
  removes the created folder.
- 13 Modifier normalization: ACCEPTED. Semantic mask pinned to Command,
  Control, Option, Fn; Shift and lock keys excluded from chord
  detection; presentation order Ctrl, Opt, Shift, Cmd pinned in tests.
- 14 PR-03 commands ownership: ACCEPTED. `src-tauri/src/commands/`
  removed from PR-03; composition happens in lib.rs/main.rs.
- 15 Budget cut: REJECTED again. The user accepted this scope at map exit
  (#11) and gates delivery per bundle; the four-hour ceiling is a
  prioritization constraint the user owns, not a plan defect.

## Round 3 — gpt-5.6-sol (xhigh, read-only)

Most round-two corrections now appear in the plans. I treat DEC-010 and the bundle scope as settled user decisions.

1. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:110) still uses two gate identities. The outcome names the final code head. Acceptance and verification name the final PR head before snapshot creation.  
   Fix: Use “final code head” consistently and permit only the authenticated snapshot commit afterward.

2. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:13) claims issue #7’s exact shapes, then makes `window` nullable. [FEATURE](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/FEATURE.md:81) says issue #7 fixes those shapes. This schema change bypassed the interview route.  
   Fix: Retain the fixed shape or ask the user and update FEATURE, DEC-002, DEC-008, and acceptance.

3. `window: null` still lacks artifact and title semantics. A focusless key-down has no display, crop center, window bounds, or `{app}` source.  
   Fix: Define display selection, both fallback crops, element metadata, and title text for every null-window case.

4. The semantic mask includes Fn, but the normalized presentation order omits Fn. Fn chords therefore have no defined title.  
   Fix: Add Fn at one exact presentation position and cover Fn combinations in parser tests.

5. [PR-02 verification](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:106) omits newly accepted security and concurrency behavior. No tests cover stop races, startup rollback, no-follow confinement, file modes, or null-window fixtures.  
   Fix: Add deterministic tests for every named race, rollback, confinement, permission-mode, and nullable-schema invariant.

6. `request_permission(accessibility)` remains undefined when Input Monitoring was never requested. Calling AX breaks the prerequisite. Refusing breaks AC-005’s matching-request promise.  
   Fix: Define automatic prerequisite sequencing or a blocked result, amend AC-005 accordingly, and test direct out-of-order calls.

7. The callback copies a frame-set handle, but the plan never says that handle is immutable. A delayed worker can lose the required predecessor frame.  
   Fix: Pin an immutable eligible-frame snapshot at enqueue and test selection after the live broker advances.

8. Chrome’s “rendered web-content area” is not a reproducible target. A click can return a valid nested AX element instead of AXWebArea.  
   Fix: Use a fixed local page and exact target, then verify its AX role before freezing the script.

9. Issue #7 makes the workflow name optional with a timestamp default. Current command signatures and plans omit this behavior.  
   Fix: Specify `Option<String>`, blank-name handling, the timestamp default, and command tests.

VERDICT: REVISE
### Root arbitration (round 3)

- 1 Gate-head wording: ACCEPTED. "Final code head" used consistently;
  only the authenticated snapshot commit follows it.
- 2 Window nullability route: ACCEPTED as a coordinator clarification,
  not a schema amendment. The schema's own null pattern for inapplicable
  fields (key: null, button: null) extends to window when no window
  resolves; every field issue #7 fixed is unchanged. Recorded as
  DEC-011 with the full rationale.
- 3 Null-window semantics: ACCEPTED. DEC-011 defines display selection,
  both fallback crops, and the frontmost-app/Unknown title rule; plans
  and fixtures pin them.
- 4 Fn ordering: ACCEPTED. Presentation order is Fn, Ctrl, Opt, Shift,
  Cmd, with Fn-combination parser tests.
- 5 Missing verification bullets: ACCEPTED. PR-02 verification now names
  the race, rollback, confinement, file-mode, null-window, and
  name-default tests.
- 6 Out-of-order permission request: ACCEPTED. DEC-011 defines
  blocked_by_prerequisite without touching the Accessibility API; AC-005
  carries the matching clause; PR-01 adds the direct out-of-order test.
- 7 Frame-handle immutability: ACCEPTED. The callback pins an immutable
  eligible-frame snapshot at enqueue; selection tested after the live
  broker advances.
- 8 Chromium target: ACCEPTED. The gate script uses a fixed local HTML
  fixture page in Chrome and verifies its coarse AX result during the
  smoke check before freezing the exact target.
- 9 Optional name default: ACCEPTED. start_recording takes an optional
  name with the timestamp default from issue #7 decision 5, with tests.

## Round 4 — gpt-5.6-sol (xhigh, read-only)

Most round-three corrections are present. The plan still has several material synchronization and verification gaps.

1. [DEC-011](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/DECISIONS.md:156) is not synchronized with [FEATURE.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/FEATURE.md:47). FEATURE still shows a mandatory name and omits null-window behavior. Its authority table still names issue #7 alone.  
   Fix: Add every DEC-011 contract to FEATURE and record DEC-011 as current schema and command authority.

2. The null-window branch defines image crops but not serialized element metadata. `element.role`, `title`, `frame`, and `source` remain undefined.  
   Fix: Define every element field for null-window events and pin the complete JSON shape.

3. Null-window adapter and parser behavior lacks tests. Current verification covers serialization but not display selection, crop geometry, or `Unknown` titles.  
   Fix: Add tests for desktop clicks, focusless key-downs, both crops, main-display selection, and both title fallbacks.

4. [PR-03 verification](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:121) omits the promised broker-advance selection test. Display replacement does not exercise frame advancement after enqueue.  
   Fix: Add a test that advances the broker after enqueue and verifies the pinned predecessor frame.

5. [PR-01’s observable criterion](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-01/PLAN.md:44) still says every permission request triggers its native path. It omits `blocked_by_prerequisite`. PR-02’s AC-005 coverage also omits this dependency evidence.  
   Fix: Add the exception to PR-01’s criterion and include PR-01’s ordering tests in PR-02’s AC-005 evidence.

6. The fixed Chrome fixture and gate script have no named repository paths. [PR-03 ownership](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:52) therefore does not clearly cover them.  
   Fix: Name exact fixture and script paths and add both paths to PR-03 ownership.

7. [DEC-003](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/DECISIONS.md:45) still places modifiers after the key. Its examples and DEC-011 use modifier prefixes.  
   Fix: Replace the title grammar with the exact `Fn+Ctrl+Opt+Shift+Cmd+Key` order.

VERDICT: REVISE
### Root arbitration (round 4)

All seven findings ACCEPTED as synchronization fixes:
- 1 FEATURE.md now carries the DEC-011 contracts (optional name with
  timestamp default, tagged channel envelope, null-window and
  blocked_by_prerequisite clarifications) and its authority table names
  issue #7 plus DEC-011.
- 2 DEC-011 pins the complete null-window element shape: role null,
  title null, fallback crop rectangle as frame, source "fallback".
- 3 PR-02 gains null-window title-fallback parser tests; PR-03 gains
  null-window display-selection and crop-geometry tests.
- 4 PR-03 gains the broker-advance pinned-predecessor-frame test.
- 5 PR-01's observable criterion carries the blocked_by_prerequisite
  exception; PR-02's AC-005 evidence names the workspace test run that
  includes PR-01's ordering tests.
- 6 The fixture and gate script have exact owned paths:
  dev/proven-gate/fixture.html and dev/proven-gate/script.md, owned by
  PR-03.
- 7 DEC-003's title grammar now reads modifier-prefix form (Fn, Ctrl,
  Opt, Shift, Cmd) matching its accepted examples.

## Round 5 — gpt-5.6-sol (xhigh, read-only) — final round at MAX_ROUNDS

All seven round-four findings are addressed. Four new material gaps remain.

1. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:55) derives timestamp defaults without an injected clock. This violates the controlled-time contract.  
   Fix: Inject one wall-clock source and test exact names and manifest timestamps with fixed time.

2. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-02/PLAN.md:43) deletes an unpublished folder, but [FEATURE](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/FEATURE.md:76) forbids all workflow deletion. The store trait has no rollback operation.  
   Fix: Make `create` return a store-owned unpublished guard, and exempt its rollback from the deletion non-goal.

3. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:32) does not define key-down display selection for windows that span displays. The selected frame can exclude the focused element.  
   Fix: Select by focused-element center, then window center, and test a spanning mixed-scale window.

4. [FEATURE](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/FEATURE.md:28) requires stream restart after display changes. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-capture-pipeline-and-backend-foundation/pr/PR-03/PLAN.md:140) tests set replacement, but not native restart wiring.  
   Fix: Add a production-manager restart test and one native display-change smoke check.

VERDICT: REVISE
### Root arbitration (round 5, terminal)

The cap was reached with VERDICT: REVISE. The four findings are not
contested positions: the root accepts every one and folded each fix into
the plans after the round. No unresolved disagreement remains between the
root and the counterparty, so no tie-breaking user question arises from
this loop; the loop ends at the cap without a counterparty APPROVED and
this log records that honestly.

- 1 Injected clock: ACCEPTED. Coordinator and store take one injected
  wall-clock source; default names and manifest timestamps tested under
  fixed time.
- 2 Startup rollback vs deletion non-goal: ACCEPTED as wording plus a
  store-owned unpublished guard from `create`; FEATURE clarifies that the
  non-goal is user-facing deletion of saved workflows.
- 3 Spanning-window key-down selection: ACCEPTED. Selection by
  focused-element center, else focused-window center, with a
  mixed-scale spanning-window test.
- 4 Stream-restart wiring: ACCEPTED. Stream-manager restart test plus a
  native display-change smoke case (arrangement or resolution change;
  hot-plug when hardware allows).
