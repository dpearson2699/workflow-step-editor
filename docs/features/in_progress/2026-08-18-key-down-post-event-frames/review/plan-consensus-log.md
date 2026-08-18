# Plan Consensus Log

- Counterparty: Codex CLI `codex exec -m gpt-5.6-sol -c model_reasoning_effort="xhigh" -s read-only` (root harness: Claude Code)
- MAX_ROUNDS: 5
- Counterparty session id: codex thread 01a01559-5765-7fc0-853a-b35dc0c85d65

## Round 1 — codex (gpt-5.6-sol, xhigh, read-only)

The plan is not ready. These defects can change behavior or violate the lifecycle contract.

1. The UI classification is false. AC-001 inspects changed PNG output through the review UI. The contract classifies displayed data as UI work. The plan instead declares `not_applicable`. See ACCEPTANCE.md, pr/PR-01/PLAN.md, and ui-gate-ownership.md.
   Fix: Apply `final_pr_design_gate` to PR-01 with separate automated and human criteria; block if no production proof route exists.
2. The signed-build gate lacks a build and launch route. The plan runs only Cargo checks. AC-001 requires the signed PR head.
   Fix: Add an exact-head `npm run tauri build` and real-product launch to the final proof route.
3. The accepted latency conflicts. FEATURE.md allows one frame interval. DECISIONS.md allows 250 ms, or about 2.5 intervals.
   Fix: Make 250 ms the single accepted latency limit across all semantic artifacts.
4. DEC-001 misstates existing click behavior. `RetainedFrames::eligible` can select a newer frame when both retained frames follow the event. AC-002 preserves this approximation.
   Fix: Rewrite DEC-001 to preserve current `eligible` semantics, including its bounded-retention approximation.
5. The wait loop can exceed its absolute deadline. PLAN.md always sleeps for one full poll.
   Fix: Sleep for `min(poll, remaining)`, define the final deadline query, and assert requested wait never exceeds 250 ms.
6. The shutdown regression lacks production-path proof. The sender-close test only proves worker drain. It passes with the current incorrect stop order.
   Fix: Prove `CapturePipeline::stop` with a pending key and verify packet emission before stream shutdown.
7. The owned path `src-tauri/src` is an invalid umbrella. It permits unrelated changes and hides ownership conflicts.
   Fix: Replace it with the six planned source files; add paths only through accepted scope expansion.

VERDICT: REVISE

### Root arbitration (round 1)

- Finding 1 — rejected. The bundle changes no UI code; the review UI renders stored PNGs unchanged. The user already decided this acceptance shape in interview Q-002 (DEC-003: feature-owned user-run recording, same shape as the foundation proven gate), and the foundation bundle's DEC-010 classified capture output as non-UI with the human proven gate as acceptance. The typed `project-ui-proof` artifact requires a scalar `displayed_value` or `control_present`, which cannot express pixel content of a screenshot; a `final_pr_design_gate` would demand a proof route this change cannot truthfully satisfy. Not the root's to change: it is a user decision already recorded, so no `Q-*` is reopened.
- Finding 2 — accepted. AC-001 evidence now names the exact-head `npm run tauri build` signed-app build and launch, the recorded facts (head, app identity, workflow id, event ids, shot files, `frame_age_ms`, max queue depth, verdict), and adds a prompt-Stop run.
- Finding 3 — accepted. FEATURE.md now states one frame interval as the expected cost and 250 ms as the single accepted bound.
- Finding 4 — accepted. DEC-001 and FEATURE.md now state that click selection keeps `RetainedFrames::eligible` unchanged including its bounded-retention approximation.
- Finding 5 — accepted. PLAN.md now specifies `wait_for(min(poll, remaining))`, one final query at the deadline, and a test asserting the requested waits never exceed the remaining window.
- Finding 6 — accepted in part. The pure worker drain test stays; PLAN.md now records that `MacosCapturePipeline::stop` order is verified by review of `pipeline.rs` and by the AC-001 prompt-Stop live run. The composition root drives real SCStream and CGEventTap objects; faking the tap would add a seam outside accepted scope (non-goal), so a unit test of `stop` is not required.
- Finding 7 — accepted in part. Owned paths narrowed to `src-tauri/src/capture` (the owning module of every planned source change: broker, worker, packets, pipeline, and any tests or small helper types), `src-tauri/src/domain/schema.rs`, `src-tauri/src/recording/pipeline.rs`, the ADR, and README. Listing only six files would force a pre-PR scope-expansion cycle if the implementer needs a sibling capture file (for example `capture/mod.rs` for a helper module); the capture directory is the cohesive module, not an umbrella hiding a conflict, and this bundle has one slice.

## Round 2 — codex (resumed thread, read-only)

Findings 2 through 5 are addressed. Finding 6 now has code review and a prompt-Stop live run. I do not re-raise finding 1.

Three material problems remain.

1. AC-001 requires maximum queue depth, but no production path exposes it. `max_queue_depth()` exists only on `MacosCapturePipeline` (`src-tauri/src/capture/pipeline.rs:48`). The coordinator stores `Box<dyn CapturePipeline>`, whose interface (`src-tauri/src/recording/pipeline.rs:90`) exposes only `start` and `stop`. `stop` also removes `running`, so a later concrete call returns zero.
   Fix: Remove maximum queue depth from AC-001, or plan the trait, coordinator, and ownership changes needed to record it.
2. Finding 7 remains unresolved. `src-tauri/src/capture` includes 17 files, including queue, stream, tap, and macOS adapter code. Most files are explicit non-goals. The contract (`work-bundle-contract.md:615`) forbids an umbrella path that hides narrower ownership.
   Fix: List the four planned capture files; add `capture/mod.rs` only if the plan requires that exact change.
3. The planned README sentence overstates actual behavior. It promises the first frame after the key and a visible character in `PLAN.md`. DEC-002 permits the oldest retained frame or a pinned pre-event fallback.
   Fix: Describe post-event selection as best-effort within 250 ms, then state the pinned fallback.

VERDICT: REVISE

### Root arbitration (round 2)

- Finding 1 — accepted. Verified: `max_queue_depth()` is a concrete-type method and the coordinator holds `Box<dyn CapturePipeline>`; recording it would require trait and coordinator changes outside scope. Removed maximum queue depth from AC-001's evidence list.
- Finding 2 — accepted. Owned paths now list the four planned capture files plus `domain/schema.rs`, `recording/pipeline.rs`, the ADR, and README; the plan states that the selected-frame value and wait-runtime seam live inside `packets.rs` and `worker.rs` with no new module file.
- Finding 3 — accepted. The planned README sentence now describes post-event selection as best-effort within 250 ms with the pre-event fallback.

## Round 3 — codex (resumed thread, read-only)

All round-two findings are addressed.

- AC-001 no longer requires inaccessible queue-depth evidence.
- Owned paths now match the planned files.
- README wording now includes best-effort timing and fallback behavior.

I found no new material problem.

VERDICT: APPROVED

### Root disposition

Converged in round 3 of 5. Rejected finding (round 1, item 1: UI classification) stands with its logged reason; every other finding was folded into the canonical artifacts before adoption.

## Round 4 — codex (resumed thread, read-only) — replan after AC-001 gate (DEC-004, PR-02)

The settle loop is mostly precise. The bundle still has material contradictions.

1. [ACCEPTANCE.md]ACCEPTANCE.md:6 and DEC-003 still bind the blocking recording to superseded PR-01. That head already failed this gate.  
   Fix: Bind the gate to the exact PR-02 head and preserve PR-01 only as failed historical evidence.

2. [DECISIONS.md]DECISIONS.md:9 and [FEATURE.md](/Users/dpearson/repos/workflow-step-editor/.claude/worktrees/spec-driven-orchestrator-issue-38-072a42/docs/features/in_progress/2026-08-18-key-down-post-event-frames/FEATURE.md:15) still require the first post-event frame. This contradicts DEC-004.  
   Fix: Make DEC-004 supersede DEC-001’s key-down clause, then remove the stale first-frame language.

3. [PLAN.md]pr/PR-02/PLAN.md:99 repeats PR-01’s failed README text. It omits the settle threshold and newest-frame selection.  
   Fix: Document the 100 ms settle, newest in-window selection, 250 ms deadline, and pinned fallback.

4. [PLAN.md]pr/PR-02/PLAN.md:37 incorrectly requires display selection in both the worker and `build_packet`. PR-01 already makes `build_packet` frame-explicit.  
   Fix: Retain `select_display` and `select_pinned_frame`; let only the worker select the frame.

5. [PLAN.md]pr/PR-02/PLAN.md:42 omits the settle value from the interface and production wiring. Line 58 later calls it a `WaitRuntime` value.  
   Fix: Define `POST_EVENT_SETTLE_NS = 100_000_000` explicitly and require all production and test paths to use it.

6. [PLAN.md]pr/PR-02/PLAN.md:175 does not lock the inclusive 100 ms boundary. “At or after” permits an implementation test above the boundary.  
   Fix: Test exact `event + 100 ms` success and `event + 100 ms - 1 ns` non-settle behavior.

7. PR-01 names the query `oldest_in_window` and documents oldest selection across the broker and worker. PR-02 omits these semantic updates.  
   Fix: Rename the query to `newest_in_window` and update all related comments and test names.

8. [DEC-005]DECISIONS.md:101 justifies the waiver as work inside the same slice. PR-02 is explicitly a replacement slice.  
   Fix: Describe the waiver as the same accepted goal and seam through replacement PR-02.

VERDICT: REVISE
### Root arbitration (round 4)

All eight findings accepted:
1. AC-001 and DEC-003 now bind the gate to the head of the final implementation slice (PR-02); the PR-01 head `176be565` run is failed historical evidence (GA-007).
2. DEC-001's key-down clause and FEATURE.md now state the DEC-004 rule (newest in-window after a 100 ms settle) and record that DEC-004 supersedes the original "first buffered frame" clause.
3. PR-02 PLAN.md README wording now states the settle, newest-in-window selection, 250 ms deadline, and pinned fallback, and names PR-01's wording as replaced.
4. PR-02 PLAN.md now retains PR-01's `select_display`/`select_pinned_frame`; `build_packet` stays frame-explicit; only the worker selects the frame.
5. `POST_EVENT_SETTLE_NS = 100_000_000` is defined explicitly beside the window constant; `WaitRuntime` gains `settle_ns()`; production and test paths must use it.
6. Boundary tests: exactly `event + 100 ms` satisfies the settle; `event + 100 ms - 1 ns` does not.
7. Rename `oldest_in_window` -> `newest_in_window` and update related comments and test names.
8. DEC-005 rationale now describes the waiver as the same accepted goal and seam delivered through replacement slice PR-02.

## Round 5 — codex (resumed thread, read-only) — MAX_ROUNDS reached

Findings 1–3 and 5–8 are addressed. Finding 4 still has one conflicting instruction.

1. [PLAN.md]pr/PR-02/PLAN.md:54 calls both `select_display` and `select_pinned_frame`. The latter already calls `select_display` in PR-01.  
   Fix: Call only `select_pinned_frame`, then use `SelectedFrame.display.id` for the broker query.

2. [DECISIONS.md]DECISIONS.md:13 and [FEATURE.md](/Users/dpearson/repos/workflow-step-editor/.claude/worktrees/spec-driven-orchestrator-issue-38-072a42/docs/features/in_progress/2026-08-18-key-down-post-event-frames/FEATURE.md:20) still promise one frame interval of expected latency. A frame arriving before 100 ms cannot satisfy the settle. The worker can require another interval or the deadline.  
   Fix: State a 100 ms minimum settle and a 250 ms cap without promising one-frame latency.

3. [PLAN.md]pr/PR-02/PLAN.md:111 says README will promise the newest frame captured within 250 ms. The broker only searches its two retained frames. A captured frame can be evicted before a late worker runs.  
   Fix: Say “newest retained in-window frame” and state the pinned fallback when no such frame remains.

VERDICT: REVISE
### Root disposition (round 5, cap reached)

The loop reached MAX_ROUNDS = 5 with `VERDICT: REVISE`, so no further round
runs and no convergence is claimed. All three round-5 findings are accepted
and folded before adoption (no counter-position, so no user tie-break is
required):
1. PR-02 PLAN.md now calls only `select_pinned_frame` (which applies `select_display`) and uses `SelectedFrame.display.id` for the broker query.
2. DEC-001 and FEATURE.md now state a 100 ms minimum settle and a 250 ms cap on added latency instead of promising one-frame latency.
3. PR-02 PLAN.md README wording now says "newest retained in-window frame" and states the pinned fallback when no such frame remains.
The replan's earlier rounds: round 4 (eight findings, all accepted) and this round; the original plan converged APPROVED in round 3.
