# Plan Consensus Log

- Counterparty: `codex exec -m gpt-5.6-sol -c model_reasoning_effort="xhigh" -s read-only`
- MAX_ROUNDS: 5
- Counterparty session: codex thread 01a011fe-0746-7ab1-9915-cd8f6b268b19

## Round 1 — codex gpt-5.6-sol xhigh

The plan has material contract, concurrency, and deletion defects.

1. **AC-001 has invalid ownership and evidence.** It mixes the full functional loop with final visual approval. The Stop-banner target cannot prove that loop. See [ACCEPTANCE.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/ACCEPTANCE.md:3) and [PR-03/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:104).

   Fix: Make AC-001 feature-owned with full-loop evidence, then add a separate final design criterion.

2. **PR-03 contains two independent flows.** Saved-workflow deletion does not depend on recording or draft review. A shared removal primitive does not establish cohesion. It also adds substantial final-slice semantics while declaring none. See [PR-03/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:42).

   Fix: Split record flow from saved deletion, then select a later low-semantics final UI slice.

3. **Deletion lacks a race-safe operation.** The plan permits an argument or test instead of selecting a descriptor-relative operation. Current checks return a path after separate metadata checks. See [store.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/store.rs:230).

   Fix: Require descriptor-relative traversal and deletion without path re-resolution after validation.

4. **Deletion lacks a partial-failure boundary.** A recursive deletion can fail after removing the manifest. The workflow then disappears despite backend failure. This violates [ADR 0003](/Users/dpearson/repos/workflow-step-editor/docs/adr/0003-hard-delete-for-saved-workflows.md:39).

   Fix: Use same-root staging, delete `workflow.json` last, and restore the original name after failure.

5. **The active-workflow guard cannot cover finalization.** Current `Stopping` and `Failed` phases lose the workflow ID. See [coordinator.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/coordinator.rs:150).

   Fix: Preserve the workflow ID through finalization and define one mutation-lock ordering.

6. **Failed finalization is not observable.** A spontaneous `Failed` terminal omits manifest-save status. The worker keeps the original error when `save_manifest` also fails. See [coordinator.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/coordinator.rs:484).

   Fix: Add explicit finalization status to the failed terminal and test both save outcomes.

7. **Immediate Stop remains broken.** The plan handles early channel messages but not an immediate Stop click. The backend returns `StartInProgress` during startup. See [coordinator.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/coordinator.rs:300).

   Fix: Latch Stop during startup and invoke it after startup succeeds.

8. **Workflow removal can race queued autosaves.** PR-02 only suppresses updates after step deletion. Discard and whole-workflow deletion can leave stale requests and errors. See [PR-02/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:31).

   Fix: Invalidate the workflow generation before removal and ignore every completion from the old generation.

9. **The UI proof route is not executable yet.** PR-01 names no adapter, command, deterministic data seam, or adapter-owned path. Its high-cost mechanism field lists only Vitest. See [PR-01/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-01/PLAN.md:30).

   Fix: Probe and name the production UI route before adoption, or record the required blocker.

10. **The new IPC interfaces remain incomplete.** The plan omits exact summary types, duration units, screenshot command name, binary transport, and error modes. Its landing cache revokes URLs only on “discard.” See [PR-01/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-01/PLAN.md:13).

   Fix: Define exact Rust and TypeScript DTOs, raw binary transport, errors, and unmount cache cleanup.

11. **Final documentation has no owner.** PR-01 owns README without planning an update. Later slices cannot publish the required final summary or walkthrough. See [PROJECT_GOAL.md](/Users/dpearson/repos/workflow-step-editor/docs/PROJECT_GOAL.md:71).

   Fix: Move README and written-walkthrough ownership to the final implementation slice.

12. **The plan ignores the hard four-hour limit.** Three serial PRs add security, concurrency, UI automation, and extensive polish. No budget or cut line exists. See [AGENTS.md](/Users/dpearson/repos/workflow-step-editor/AGENTS.md:12).

   Fix: Add explicit time allocations and ask which accepted items to defer when the budget cannot hold.

13. **The task binding conflicts with the declared harness.** All plans use `codex_task_request`, but the bundle declares Claude Code routes. See [model-routing-and-delegation.md](/Users/dpearson/repos/workflow-step-editor/.agents/workflows/spec-work-orchestrator/references/model-routing-and-delegation.md:28).

   Fix: Use the Claude task adapter binding consistently, or resolve the lifecycle contract conflict first.

14. **Owned paths are too broad.** The plans grant whole `src-tauri` trees. This permits unrelated capture changes and hides narrower ownership. See [PR-01/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-01/PLAN.md:36).

   Fix: List exact existing files and bounded new directories for each slice.

VERDICT: REVISE
### Root arbitration — Round 1

1. REJECTED. `ui-gate-ownership.md` requires the bundle-level design
   acceptance to be a stable AC owned by the final UI slice, not
   feature-owned. AC-001's evidence is the final human checkpoint (the
   user runs the full loop on the exact head), not the automated
   Stop-banner target — that target belongs to AC-004. The structure
   follows the contract as written.
2. ACCEPTED with modification. Saved-workflow deletion (AC-005, the
   removal primitive, `delete_workflow`, and the Delete… confirmation)
   moved into PR-02, whose seam is saved-workflow management (edit,
   rename, delete of the same stored folder authority). PR-03 becomes the
   thin record-flow final UI slice; its Discard consumes PR-02's command.
   No fourth slice: the accepted delivery stays three reviewed PRs and
   the MVP time budget rules out another PR/review cycle.
3. ACCEPTED in substance, mechanism adapted. Full descriptor-relative
   traversal exceeds this local single-user app's threat model. The plan
   now requires rename-to-staging inside the root before recursive
   removal, which closes the validate-then-remove substitution window at
   the rename boundary.
4. ACCEPTED. Staging rename with restore-on-failure is now the planned
   partial-failure boundary; a failed deletion stays visible per ADR
   0003.
5. ACCEPTED. PR-02 already required finalization-phase identity; wording
   retained and the lock-ordering requirement stands (no phase lock held
   during I/O).
6. ACCEPTED in substance, simpler mechanism. No envelope extension: the
   frontend decides reviewability after failure by loading the workflow;
   a load failure surfaces the error without draft review.
7. ACCEPTED. PR-03 now latches a Stop pressed during startup and issues
   it once start resolves.
8. ACCEPTED. PR-02 invalidates the workflow's autosave generation before
   removal; stale completions are ignored.
9. REJECTED as an adoption gate; ACCEPTED as guidance. The workflow
   contract discovers the proof route at the proof gate, and PR-01
   explicitly owns establishing it or blocking. The plan now names the
   candidate direction (launch the dev build; observe via the app's
   macOS accessibility tree) and counts the route as new high-cost
   verification.
10. ACCEPTED lightly. Units (integer milliseconds, optional), command
    shape, raw-bytes transport, string errors, and unmount revocation are
    now stated; exact DTO field lists stay implementer latitude.
11. ACCEPTED. README final narrative and walkthrough moved to PR-03
    (owns `README.md`); PR-01 README changes limited to setup/commands.
12. REJECTED. The capability scope is user-accepted (issues #11 and #13)
    and the operator explicitly directs full-speed delivery of both MVP
    capabilities; recording the four-hour tradeoff belongs to the
    submission write-up, not this plan. No scope cut without a changed
    user decision.
13. ACCEPTED. All three plans now bind "Claude task adapter request".
14. ACCEPTED partially. PR-01 narrowed to
    `src-tauri/src`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities`,
    and the Cargo manifests; `src` remains whole because the frontend
    shell is genuinely new. PR-02/PR-03 already owned `src-tauri/src`
    prefixes; exact per-file lists would fight legitimate new-file
    creation inside owned directories.

## Round 2 — codex gpt-5.6-sol xhigh

Several Round 1 issues are fixed. These material defects remain.

1. **Failure reviewability is still unsound.** Recording creation persists an empty manifest. A failed final save can leave a loadable, stale workflow. See [store.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/store.rs:376) and [PR-03/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:26).

   Fix: Return explicit finalization status and gate draft review on that status.

2. **Staging restoration is not rollback.** Recursive removal can delete `workflow.json` before another removal fails. `list` then skips the restored directory. See [store.rs](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/store.rs:459).

   Fix: Delete `workflow.json` last and verify listability before reporting a restored failure.

3. **Staging creates an unhandled crash state.** A crash after rename leaves a hidden directory containing keystrokes. That conflicts with the no-trash decision. See [PR-02/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:38) and [ADR 0003](/Users/dpearson/repos/workflow-step-editor/docs/adr/0003-hard-delete-for-saved-workflows.md:13).

   Fix: Avoid persistent staging, or obtain a decision for authenticated startup cleanup.

4. **Staging rename does not authenticate the validated target.** Another directory or symlink can replace the source before rename. The command then deletes the substitute.

   Fix: Compare file type, device, inode, and manifest identity after rename and before deletion.

5. **AC-005 has no valid automated UI evidence.** It requires automated UI proof. PR-02’s only typed proof binds AC-003 and `step-detail-pane`. See [ACCEPTANCE.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/ACCEPTANCE.md:62) and [PR-02/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:133).

   Fix: Remove that proof requirement from AC-005, or give deletion its own UI slice and receipt.

6. **PR-03 is not a thin final UI slice.** It adds the asynchronous recording reducer, draft lifecycle, Save, Discard, and failure recovery. Its route calls the ordering cases significant. See [PR-03/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:13).

   Fix: Move these semantics earlier and reserve the final slice for final UI work, or add another slice.

7. **PR-02 adds two unsupported Delete surfaces.** FEATURE requires one non-primary control. The pinned landing prototype contains no Delete control. See [FEATURE.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/FEATURE.md:34) and [PR-02/PLAN.md](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:45).

   Fix: Select one Delete location and remove the unsupported second affordance.

VERDICT: REVISE
### Root arbitration — Round 2

1. ACCEPTED with a simpler mechanism. DEC-009 no longer distinguishes
   "finalization persisted": every failed recording whose workflow loads
   enters draft review behind a banner stating the recording failed and
   may be incomplete; a non-loading workflow surfaces its error on the
   landing page. No envelope extension.
2. ACCEPTED via mechanism change. Staging is gone (see 3), so restore
   and listability interactions no longer exist. Partial removal failure
   surfaces its error and a retried delete completes; missing pieces
   count as removed.
3. ACCEPTED. The staging design is withdrawn — no hidden directory can
   survive a crash.
4. ACCEPTED via mechanism change. The primitive now uses
   `std::fs::remove_dir_all`, whose post-CVE-2022-21658 implementation
   removes descriptor-relative and does not follow or traverse symlinks,
   after the store's existing symlink/non-directory validation. A
   same-user process substituting one real in-root directory for another
   could equally delete either directly; that is outside the app's threat
   model.
5. ACCEPTED. AC-005's evidence no longer claims typed automated UI
   proof; it binds Rust and frontend tests, and the deletion flow is
   exercised live inside AC-001's final human loop. PR-02's single typed
   proof remains AC-003 at `step-detail-pane`.
6. REJECTED. The record flow must ship in some slice and must follow the
   editing surface it reuses; a "final UI polish slice" would be a fourth
   PR, which the MVP time budget and the accepted three-PR delivery rule
   out. The final slice's production (backend) semantics are one additive
   timestamp field; the flow itself is the slice's accepted outcome.
7. ACCEPTED. One Delete… control, in the detail header only; the landing
   row affordance is removed.

## Round 3 — codex gpt-5.6-sol xhigh

Round 2 findings 4, 5, and 7 are resolved. Rust 1.97.1 confirms the macOS `remove_dir_all` security claim. Material defects remain.

1. Partial deletion still violates [ADR 0003](/Users/dpearson/repos/workflow-step-editor/docs/adr/0003-hard-delete-for-saved-workflows.md:39). [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:44) permits failure after removing `workflow.json`. [list()](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/store.rs:459) then hides the remaining folder.

   Fix: Obtain approval to relax ADR 0003, or select deletion semantics that preserve listability after every error.

2. The revised failed-recording decision lacks complete acceptance coverage. [AC-004](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/ACCEPTANCE.md:46) omits both failure destinations. [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:87) tests only the loadable draft path.

   Fix: Add both destinations to AC-004, then test load failure and both stop-result versus terminal-envelope orders.

3. PR-03 still gives a false cohesion answer. It adds the production reducer, live ordering, draft lifecycle, Save, Discard, and failure recovery. Yet [PR-03](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-03/PLAN.md:51) declares no substantial production semantics. The [lifecycle contract](/Users/dpearson/repos/workflow-step-editor/.agents/workflows/spec-work-orchestrator/references/work-bundle-contract.md:315) requires a split for a truthful `yes`.

   Fix: Recompose the three slices so the final slice contains only final UI integration and proof.

4. Draft Save and Discard lack failure-state contracts. Verification covers only default naming and confirmation. A failed command can hide or mislabel retained data.

   Fix: Exit draft only after command success; preserve draft state and show errors after Save or Discard failures.

VERDICT: REVISE
### Root arbitration — Round 3

1. ACCEPTED. The removal primitive now deletes in listability-preserving
   order: children except `workflow.json` first, the manifest last, the
   empty directory last. A failure before manifest removal leaves the
   workflow listed per ADR 0003, and keystroke data is removed before
   the manifest. No ADR relaxation needed.
2. ACCEPTED. AC-004 now names both failure destinations, and PR-03's
   verification covers load failure and both stop-result versus
   terminal-envelope orders.
3. REJECTED. The predicate guards against a final UI slice bundling
   production machinery beyond its UI outcome — schema, storage,
   services, command semantics. PR-03 adds none (one additive timestamp
   field); its UI flow is the slice's accepted outcome. In an all-UI
   capability the last UI feature necessarily ships in the last UI
   slice; the alternative is an empty ceremony PR, which the accepted
   three-PR delivery and the MVP budget rule out. The interpretive
   reading is now recorded in PR-03's cohesion proof.
4. ACCEPTED. Draft exits only on command success; failed Save keeps the
   draft state and dialog error; failed Discard keeps the draft state
   and data with the error visible. AC-004 and PR-03 verification updated.

## Round 4 — codex gpt-5.6-sol xhigh

Round 3 addresses failed-recording coverage and draft action failures. I found no new defect in the recorded PR-03 interpretation.

1. The ordered traversal reopens the substituted-link race. [workflow_dir()](/Users/dpearson/repos/workflow-step-editor/src-tauri/src/recording/store.rs:230) returns a path after separate metadata checks. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:39) applies `remove_dir_all` only to child paths. A replacement symlink in the workflow-directory component can redirect those paths outside the root. Final-component `O_NOFOLLOW` does not protect that intermediate component. This violates [AC-005](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/ACCEPTANCE.md:68).

   Fix: Anchor every operation to an `O_NOFOLLOW` directory descriptor, and test directory replacement after validation.

2. A final directory-removal failure still breaks [ADR 0003](/Users/dpearson/repos/workflow-step-editor/docs/adr/0003-hard-delete-for-saved-workflows.md:39). Removing the root needs parent write access. Removing its files needs workflow-directory write access. Therefore, manifest deletion can succeed before root deletion fails. Listing then skips the remaining folder, although the backend reports failure. Tests cover only failures before manifest removal.

   Fix: Define manifest removal as successful deletion with best-effort empty-shell cleanup, or restore it before returning an error.

VERDICT: REVISE
### Root arbitration — Round 4

1. ACCEPTED via composite mechanism. The ordered per-child traversal is
   withdrawn; the primitive is back to one validated
   `std::fs::remove_dir_all` call (the round-2-accepted, symlink-race-safe
   mechanism), so no intermediate-component path traversal exists to
   exploit.
2. ACCEPTED using the finding's own suggested semantics. The manifest is
   the success boundary: a removal error with `workflow.json` present
   reports failure (workflow stays listed per ADR 0003); an error after
   the manifest is gone reports success with one best-effort shell
   cleanup retry. Verification updated to inject both cases.

## Round 5 — codex gpt-5.6-sol xhigh (final round)

The single `remove_dir_all` call resolves the prior substituted-link traversal issue. One material defect remains.

1. Manifest absence does not prove hard deletion. `remove_dir_all` can delete `workflow.json`, then fail on retained events or screenshots. [PR-02](/Users/dpearson/repos/workflow-step-editor/docs/features/in_progress/2026-08-17-review-ui/pr/PR-02/PLAN.md:43) incorrectly calls the remainder a shell and reports success. This hides sensitive data that [ADR 0003](/Users/dpearson/repos/workflow-step-editor/docs/adr/0003-hard-delete-for-saved-workflows.md:3) requires deletion to remove. The planned test checks manifest state but does not require all sensitive artifacts to be absent.

   Fix: Cache the manifest; after errors, require directory absence for success, or restore the manifest and report failure.

VERDICT: REVISE
### Root arbitration — Round 5 and loop closure

1. ACCEPTED exactly as proposed. The primitive caches the manifest bytes
   before removal; success requires directory absence; on residual
   failure the manifest is restored when missing and failure is
   reported, so no sensitive remnant hides behind a success result.
   Verification updated to require directory absence for success and to
   inject a remnant-leaving failure.

The loop reached MAX_ROUNDS with the final finding accepted and folded
in. No finding remains contested: the counterparty confirmed in round 4
that it found no new defect in the recorded PR-03 interpretation, and
round 5 confirmed the substituted-link traversal resolved. There are no
unresolved decision-shaped items to route to the user.
