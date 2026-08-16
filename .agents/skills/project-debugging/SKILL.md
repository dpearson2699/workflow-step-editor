---
name: project-debugging
description: >-
  Diagnose or fix runtime, data-flow, persistence, concurrency, performance,
  integration, backend, external-service, navigation, or displayed-behavior
  defects in the current repository. Use it to establish executable root-cause
  evidence, map the blast radius, design a generalized regression, maintain the
  exact GitHub issue owner, and choose the diagnosis-only, fast-fix, or full
  spec-work route before production edits. Do not use for feature requests,
  routine builds/tests, architecture explanation, or pull-request review alone.
---

# Project Debugging

Own defect classification, evidence sequencing, regression abstraction, and
bug-route selection. Let `AGENTS.md` and installed project/domain skills own
framework commands, runtime tooling, build/test mechanics, external-service
recipes, and UI proof acquisition.

## Classify intent

- `diagnose_only`: investigate or explain without a requested correction. Keep
  product and test code read-only. Do not start a spec-work bundle, planner
  pass, implementation task, PR, or merge flow.
- `fix_or_ship`: establish the evidence gates below, then run
  `scripts/bug-fix-route`. Do not initialize a bundle before it selects
  `fix_full`.

Issue maintenance is independent of product mutation. Apply
`.github/issue-label-policy.json` when the user supplies an issue, the active
route selects an existing owner, or a verified actionable defect falls outside
the current accepted scope. Do not create an issue solely for an in-scope
defect that this accepted fix will fully resolve. Search all issue states for
the exact canonical marker, authoritatively fetch every exact candidate, and
reuse only the unique verified open non-duplicate owner. Preserve existing
prose, append only missing verified evidence, add the marker when absent,
reconcile exact labels, and close only after the correction is durably
effective in the authoritative owning system. Closed matches are history, not
reusable owners. After mutation, repeat exact-marker discovery and
authoritative verification. Never suppress or create an issue from title
similarity alone.

## Establish root cause

For a non-trivial, intermittent, cross-layer, performance, memory, state, or
prior symptom-only failure, read `references/root-cause-orchestration.md` and
build its hub-spike evidence map before edits. Select project capabilities from
`references/debugging-playbook-routing.md` and `AGENTS.md`.

Use the closest executable observation at the owning boundary: debugger or
process trace, logs, request/response capture, persisted-state inspection,
profiling, UI automation, or a failing behavior check. Source inspection,
community reports, and old logs may form hypotheses; they do not alone prove
the current owning seam. Query live external systems during research when the
defect depends on their current data or contract.

Before naming or writing regression coverage or editing production code, read
`references/regression-test-abstraction.md` and complete every field in its
record. Identify the generalized invariant, observable owning seam,
equivalence classes, essential inputs, outcomes, and visible proof obligation.
An incident-shaped or incomplete record blocks implementation.

## Choose the fix lifecycle

For `fix_or_ship`, run `scripts/bug-fix-route --help` and pass the observed
facts. `fix_fast` requires one confirmed owning seam, a complete regression
abstraction, one cohesive PR, resolved consequential decisions, and no
cross-repository coordination, destructive data work, auth/secret-policy
change, unrelated scope expansion, or unchecked applicable external data.

`fix_full` remains a one-repository delivery lifecycle. If the classifier names
`cross-repository`, use the current bundle to record evidence and the dependency
or blocking issue, but stop before Delivery until the other repository has its
own accepted owner and durable completion/merge evidence. Do not treat this
core as authority to mutate or coordinate another repository.

- On `fix_fast`, read `references/fast-fix-lane.md` completely and follow its
  implementation, independent-review, GitHub issue, and merge contract. Do not
  load the shared core or create a spec-work bundle.
- On `fix_full`, create this descriptor and initialize the canonical bundle;
  then read the shared core completely and let it own the lifecycle:

```text
workKind: bug_fix
bundleParent: docs/bug_fixes
primaryArtifact: BUG_FIX.md
planningGate: confirmed_root_cause_and_regression_abstraction
```

```sh
.agents/workflows/spec-work-orchestrator/scripts/work-state init \
  --work-kind bug_fix \
  --work-id <YYYY-MM-DD-slug> \
  --work-bundle docs/bug_fixes/in_progress/<YYYY-MM-DD-slug>
```

Persist confirmed evidence in `discovery/root-cause.md` and the completed
record in `discovery/regression-test-abstraction.md`. Keep the generated
interview ledger even when evidence resolves every gray area. Do not read
`.agents/workflows/spec-work-orchestrator/CORE.md` until the root cause,
owning seam, blast radius, and regression abstraction are confirmed.

If a fast-route predicate later fails, stop further mutation and restart as
`fix_full`; do not create a hybrid or migrate partial fast-lane state.

## Evidence and implementation rules

1. Route the symptom to the smallest applicable capability route.
2. Record a compact research receipt: skills/docs consulted, runtime or service
   evidence gathered, and which hypotheses were confirmed or rejected.
3. Prove the owning seam and inspect its blast radius with an applicable
   code-intelligence capability discovered from the current repository and
   installed tools.
4. Complete the canonical regression abstraction.
5. Write the failing owning-seam regression or executable acceptance check.
6. Make the smallest complete correction at the owner of the invariant.
7. Run focused and relevant broader verification plus the repository-native UI
   proof route when the change is user-visible.
8. Remove temporary probes before publication.

Project/domain skills may refine mechanics but may not weaken this evidence
gate, issue-owner identity, or route boundary.

## Reference routing

- `references/debugging-playbook-routing.md`: read when the symptom or evidence
  capability is unclear or crosses layers.
- `references/root-cause-orchestration.md`: read for non-trivial, intermittent,
  cross-layer, performance, memory, state, or suspected symptom-only failures.
- `references/regression-test-abstraction.md`: read before any bug-fix code or
  test edit.
- `references/fast-fix-lane.md`: read only after the classifier returns
  `fix_fast`.

## Non-goals

Do not use this skill for a new feature, a routine command run, an explanation
without a defect symptom, an already-captured artifact analysis with its own
specialist, or PR review alone. Do not copy framework/provider command recipes
into this generic router; discover them from repository-owned instructions,
tooling, and installed domain skills.

## Bundled script

`scripts/bug-fix-route` emits a deterministic route, failed predicates, review
risk, independent-lens count, remote-provider requirement, and UI checkpoint
count as JSON.
