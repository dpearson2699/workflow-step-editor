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
