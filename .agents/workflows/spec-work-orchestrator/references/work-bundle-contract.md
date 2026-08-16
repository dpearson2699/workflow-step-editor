# Spec-Work Bundle Contract

Read this reference when creating, planning, resuming, starting a wave,
or closing a work bundle. Markdown is semantic authority. `state.json` is
operational authority. `STATUS.md` is generated and must never be hand-edited.

## Contents

- [Bundle shape](#bundle-shape)
- [Semantic artifacts](#semantic-artifacts)
- [Planning and routing receipts](#planning-and-routing-receipts)
- [Parallelization assessment](#parallelization-assessment)
- [Plan support agents](#plan-support-agents)
- [Coordinator and worktree contract](#coordinator-and-worktree-contract)
- [State tool](#state-tool)
- [Parallel and stacked execution](#parallel-and-stacked-execution)
- [Activation and closure](#activation-and-closure)

## Bundle shape

The invocation descriptor selects exactly one closed pair:

- `feature` -> `docs/features` -> `FEATURE.md`.
- `bug_fix` -> `docs/bug_fixes` -> `BUG_FIX.md`.

```text
<bundle-parent>/in_progress/<date>-<work-slug>/
  FEATURE.md | BUG_FIX.md     # exactly one descriptor-selected primary
  INTERVIEW.md
  DECISIONS.md
  ACCEPTANCE.md
  state.json
  STATUS.md
  discovery/                  # create only files the work needs
    planning-routing.md       # required from the first planning dispatch
    planning-continuation.json # bridge-owned parent Plan cursor, once Pro runs
    architecture.md
    data.md
    domain.md
    ui.md
  pr/
    PR-01/
      PLAN.md
      RECEIPT.md
      evidence/              # typed attempt evidence only
```

Do not create a hand-maintained queue, worker ledger, registry, lease, or heartbeat
file. `planning-continuation.json` is a deterministic bridge-owned receipt, not a
general queue or work-state store. Slice state and dependencies live in
`state.json`; `STATUS.md` renders the human view. Keep the bundle in one root-owned coordinator branch/worktree. Every
slice uses exactly `pr/<PR-id>/PLAN.md`, `pr/<PR-id>/RECEIPT.md`, and
`pr/<PR-id>/evidence/**`; root-level or overlapping slice directories are invalid.

## Semantic artifacts

### Primary specification

For `feature`, use `FEATURE.md`:

```markdown
# <Feature title>

## Source / Issue

<Exactly one owning GitHub issue URL — the `enhancement` issue the root
creates at bundle initialization per `github-board-sync.md`, or a
user-supplied issue verified as the owner. This line is the authoritative
owning-issue identity for the board card and completion closure.>

## Goal

<Desired user or system outcome.>

## Scope

- <Included behavior.>

## Non-Goals

- <Explicit boundary.>

## Doc Authority

| Subject | Current authority | Conflict or obligation | Owning update |
| --- | --- | --- | --- |
| <behavior or policy> | `<evidence path>` | <none or conflict> | `<canonical path>` or none |

## Open decision IDs

- <Q-* IDs or none.>

## Codex Task Roster

- Status: draft | registered
- Entry: PR-01 | implementation | <requested model/effort> | <named refresh/reroute predicates>
- Entry: PR-01 | review | gitnexus-pr-review native | <named stale-head/blocker predicates>

## UI Acceptance Policy

- Policy: final_pr_design_gate
- Final UI slice: PR-*

## Durable Sources

- `<canonical doc or code/test anchor>`
```

Omit `UI Acceptance Policy` only when no slice affects user-visible UI, displayed
data, navigation-visible values, or model/service output feeding a screen. Follow
`interview-and-doc-authority.md` for evidence order and canonical ownership.
The Markdown task roster is a readable semantic mirror. New Plan authority comes
from the complete proposal's structured task entries; do not parse wrapped or
multiline Markdown to reconstruct the executable route or replacement predicates.

For `bug_fix`, use `BUG_FIX.md` with these required sections in addition to the
shared task-roster and UI-policy contracts:

```markdown
# <Bug-fix title>

## Source / Incident
## Observed Behavior
## Expected Behavior
## Failed Invariant
## Confirmed Owning Seam and Root Cause
## Goal
## Scope
## Non-Goals
## Compatibility Constraints
## Document Authority
## Open Decisions
## Codex Task Roster
```

Add `## UI Acceptance Policy` to `BUG_FIX.md` only when the bug-fix bundle is
UI-affecting, using the shared `final_pr_design_gate` contract above. Omit the
section for a wholly non-UI bug fix.

A bug-fix bundle also requires `discovery/root-cause.md` with `Status:
confirmed`, owning seam, failed invariant, and evidence; and
`discovery/regression-test-abstraction.md` with `Status: complete` plus every field
from the debugging skill's canonical regression abstraction record.

### INTERVIEW.md

```markdown
# Interview

## GA-001: <Gray area>

- Status: open | answered_by_docs | answered_by_user | question_required | closed | deferred | blocked
- Kind: fact | decision
- Uncertainty: <What is not established.>
- Why it matters: <Effect on behavior, architecture, risk, slices, or acceptance.>
- Evidence inspected: <Current anchors.>
- Confidence: high | medium | low
- Question: Q-001 | none

## Q-001: <Product-facing decision>

- Status: open | answered | answered_by_docs | superseded | deferred | blocked
- Recommendation: <Recommended default and why.>
- Options and tradeoffs: <Concrete alternatives.>
- If wrong: <The concrete failure if this is guessed wrong.>
- Answer/source: <User answer or evidence anchor.>
- Closure reason: <Why no further question is needed.>
- Decision: DEC-001 | none
- Canonical-doc impact: <Path/obligation or none.>
```

Use stable IDs and preserve closed entries across resumes. Decision closure follows
`interview-and-doc-authority.md`; it is not complete until every affected semantic
artifact and canonical obligation is synchronized.

### DECISIONS.md

```markdown
# Decisions

## DEC-001: <Decision title>

- Status: accepted | superseded
- Decision: <Selected behavior.>
- Rationale: <Why this option fits the goal and evidence.>
- Rejected alternatives: <Brief list.>
- Canonical docs: <Paths to update, or none.>
```

Keep product and implementation decisions here before planning. Promote durable
project-wide decisions into their canonical docs before dependent code ships.

### ACCEPTANCE.md

```markdown
# Acceptance

## AC-001: <Behavioral outcome>

- Ownership: feature | slice
- Invariant: <Generalized behavior that must remain true.>
- Owning seam: <Closest observable production boundary.>
- Evidence required: <Tests, runtime proof, UI gate, or other check.>
```

Keep IDs stable. Revise prose only through an explicit decision update; do not
renumber criteria to match implementation. Each criterion is one canonical
`## AC-NNN: title` block. State stores a SHA-256 digest of its normalized block
(LF line endings, trailing whitespace removed, boundary blank lines ignored).
Duplicate, deleted, or malformed blocks and semantic digest drift fail closed.
State acceptance records carry `locked: false` until a successful completion
transaction atomically locks every passed/waived record.
Each record also stores `evidence_digests`, mapping every evidence path to the
lowercase SHA-256 of its exact file bytes. Pending records keep both evidence
collections empty; other statuses require identical nonempty path sets.
Non-pending records also bind `scope`, producing slice/head, and the current
acceptance-assignment digest. Every criterion explicitly declares `Ownership: feature`
or `Ownership: slice`. Feature-owned criteria must have no slice-plan owner. Slice-owned
criteria require exactly one current plan owner and cannot survive a head rotation or
an omitted/misassigned owner. Feature-level evidence carries no slice producer.
Every executable acceptance block requires exactly one explicit `Ownership` value.
Top-level `completion_binding` is nullable operational provenance. UI completion
atomically binds the exact typed receipt and completion attestation paths, digests,
head/tree, PR, final slice, acceptance ID, and verdict. Reopen preserves it; a later
UI completion may replace it. It does not own semantic UI policy.

### Per-slice PLAN.md

```markdown
# PR-01 Plan: <Slice title>

## Outcome

<Smallest reviewable vertical result.>

## Scope and Ownership

- Behavior: <Owned behavior.>
- Owned paths: <Repo-relative non-glob file or directory prefixes.>

## Slice Cohesion

- Primary outcome: <One user-visible or system outcome.>
- Primary execution flow: <One end-to-end flow.>
- Owning observable seam: <One production boundary where success is observed.>
- Primary acceptance criterion: <One AC-* or one explicit observable criterion.>
- Regression guards: <Other AC-* criteria or none.>
- New high-cost verification mechanism: <One mechanism or none.>
- Independent execution flows: no
- Persistence/schema compatibility plus cross-screen consumer sweep: no
- New acceptance harness plus unrelated production behavior: no
- Final UI slice adds substantial production semantics: no
- Aggregate/closure/final integration slice: no
- Unresolved implementation work: no
- Cohesion proof: <Why the listed work is inseparable at the owning seam.>
- Path-count warning: <Large inventory rationale or none; never a split decision alone.>

## Non-Goals

- <Excluded behavior.>

## Dependencies

- Slice dependencies: <PR-* IDs or none.>
- Wave: <positive integer>
- Execution mode: serial | parallel

## Acceptance Coverage

- <Every ordinary AC-* owned by this slice and how it contributes.>

## Verification

- <Focused tests and proportionate broader gates.>
- <Required project skills and UI/live-data gates.>
- Independent command: <Command that succeeds in this slice worktree.>
- UI gate: not_applicable | snapshot_required_human_deferred | final_human_required
- Automated UI acceptance: <AC-* for a UI slice; otherwise none>
- UI proof target: <stable identifier for the changed value or control; otherwise none>
- Final UI slice: <PR-* or none>
- Final design acceptance: <AC-* only on final UI slice; otherwise none>

## Implementation Route

- Requested model and effort: <route>
- Selection predicates: <named predicates from the routing reference>
- Binding: codex_task_request

## Parallelization Assessment

### Pair PR-01 + PR-02

- Dependency edge: none | <details>
- Owned-path overlap: none | <details>
- Shared API: none | <details>
- Shared schema or migration: none | <details>
- Shared canonical document: none | <details>
- Shared test fixture or generated file: none | <details>
- Shared backend resource: none | <details>
- Shared UI acceptance dependency: none | <details>
- Separate branches and worktrees: planned | not satisfied
- Independent build and verification: yes | no, <details>
- Verdict: independent | coupled
```

Assess the complete proposed wave with one canonical sorted block per pair. Record
`independent` only when every row is affirmative and current repository evidence
supports it. Any uncertainty,
implicit integration dependency, shared semantic surface, or unowned output means
`coupled` and `execution_mode: serial`. Do not use diff size or vague complexity labels.
Every `AC-*` in `Acceptance Coverage`, plus the automated and final-design fields,
is owned by this plan. Invalidating an attempt resets all of those records even when
old evidence lived outside the slice directory; unrelated acceptance stays unchanged.

Every executable plan must pass the `Slice Cohesion` parser. Split when any of
the four independent-concern predicates is `yes`, except for the evidence-bound
required-by-intent consumer continuation below. Without that evidence, a consumer
sweep remains a split reason and the diagnostic points to authenticated
required-by-intent reauthorization rather than suggesting false plan metadata.
An aggregate/closure/final-integration slice that still owns unresolved
implementation work requires a concrete cohesion proof.

During review, newly discovered out-of-scope work defaults to a follow-up issue
or a new slice. Expand the active owned-path lease only when exact evidence
shows the path is inseparable from the frozen core criterion at the same owning
observable seam. Reachability, convenience, shared terminology, or a broad
consumer sweep is not sufficient. Path count is only a warning signal;
execution-flow and owning-seam cohesion decide the boundary.

If the root selects a follow-up issue, load
`.github/issue-label-policy.json`, classify one automated issue type and
P1/P2/P3 severity (`bug` is for product behavior that does not work as intended;
`harness` is for defects in the agent harness or the repository's workflows),
and compute the fingerprint exactly from
`fingerprint_contract` in `.github/issue-label-policy.json`; do not restate a
divergent algorithm. An open issue explicitly supplied by the user or selected
by the active route is the in-scope owner only after an authoritative fetch
verifies its repository, open state, and defect identity and the all-state
exact-fingerprint inventory establishes that no different open exact owner
exists. If a different unique open exact owner exists, select it and do not
mutate the supplied issue; a supplied closed issue follows closed-history
handling. Maintaining and terminally closing the verified owner requires no
separate user authorization. Search all issue
states for the exact canonical
`<!-- spec-work-follow-up:<fingerprint> -->` marker. Follow
`issue_reuse_contract` in the same policy. Only an exact canonical fingerprint
match can suppress issue creation; title, body, or search-term similarity is
related non-owning evidence. Treat search-result state as discovery metadata
only. Before any mutation, directly fetch every exact-fingerprint candidate from
the owning repository and verify its authoritative current state; missing or
failed authoritative state verification fails closed. An initial authoritative
read with multiple open matches or a selected open issue labeled `duplicate`
fails closed. Reuse only when exactly one authoritatively verified open
exact-fingerprint candidate remains and it is not labeled `duplicate`. Treat
closed exact-fingerprint candidates as historical evidence. For a newly
observed actionable regression after resolution, create a new issue with exact
`[issue_type, severity]` labels and link the closed history.
Every newly created issue body must contain exactly one canonical marker using the
computed lowercase 64-hex fingerprint. For the unique open owner, preserve
existing prose, append only verified evidence absent from its body, add the
canonical fingerprint marker when missing, reconcile exact `[issue_type,
severity]` labels, and continue on that issue instead of creating another.
Do not rewrite its title, replace existing body prose, or publish a redundant
comment. When the accepted fix is terminally complete and durably effective in
the authoritative owning system, append any missing completion evidence and
close the owning issue without separate authorization. A committed active local
installation qualifies when it is the system of record; an uncommitted,
inactive, or unverified fix remains open.
After every issue creation, owner maintenance, label reconciliation, or closure,
repeat the same all-state
exact canonical marker discovery and authoritatively direct-fetch every discovered
exact-fingerprint candidate. Treat repeated search-result state only as discovery
metadata; list metadata never decides final ownership. Continue to a verified
follow-up publication receipt only when those authoritative reads establish exactly
one selected open owner remains, or after terminal closure the selected owner is
closed and no open exact owner remains. Otherwise fail closed without further
mutation. Refetch the selected issue and verify repository, expected state, unique
canonical marker, body evidence, and exact
`[issue_type, severity]` labels. Record required receipt fields `issue_url`,
`verified_state`,
`fingerprint`, `fingerprint_comparison`, `issue_type`, `severity`,
`expected_labels`, `disposition`, and `label_verification_status`. Disposition is exactly
`created`, `reused-open`, `created-after-closed`, or
`closed-complete`.
A verified GitNexus follow-up publication
receipt satisfies the same publication and must be recorded rather than repeated.

This GitHub publication is semantic receipt work owned by the root. The
deterministic state helper remains GitHub-free, and `state.json` never stores
issue URLs, labels, fingerprints, or publication receipts.

### Pull request description contract

`.github/pull_request_template.md` is the canonical public projection schema for
every implementation PR. The implementation task renders it before first
publication regardless of whether GitHub UI, CLI, or API creates the PR:

- `Contract source` names the applicable issue and the exact checkpointed
  primary specification and slice plan. Use `Closes #...` only when this PR
  completely satisfies that issue; otherwise use a non-closing reference.
- `Accepted outcome` comes from the slice plan's `Outcome`.
- `Scope / Included` comes from `Scope and Ownership`; `Scope / Non-Goals`
  comes from the slice plan's `Non-Goals`.
- `Acceptance` includes every `AC-*` assigned to the slice, preserving its
  required outcome without weakening it and mapping it to current or still
  required evidence.
- `Implementation`, `Verification`, `Risk and Recovery`, and follow-ups come
  from the actual current-head work and observed evidence, not predictions.

The Contract, Scope, and required-outcome text in Acceptance freeze when the PR
is created. Implementation and evidence cells may be updated as the head
changes. If Plan legitimately changes the accepted contract for the same open
PR, update the owning primary artifact, decisions, acceptance, and slice plan
through the normal workflow first. Then synchronize the public contract and
append a dated `Contract amendments` entry naming the approved source and
reason. Never silently rewrite frozen text. A summary-only description, unfilled
template, task-local note, or bare bundle-path reference is not a public
contract and blocks publication or review.

### Canonical implementation-PR bundle snapshot

Every implementation PR includes a deterministic, coordinator-approved snapshot of
this entire work bundle. The implementation child first publishes its ordinary code
head and open-PR identity, sends one attention callback without terminalizing, and
stops mutation. The root consumes that exact handoff, freezes one Git commit after the
applicable lifecycle update, and resumes the same bound child with the exact slice,
commit, lifecycle path, classification, and canonical manifest digest. That bounded
resume authorizes only the frozen bytes; it does not delegate semantic or lifecycle
authorship. The child publishes its terminal receipt and terminal callback only after
the exact snapshot is materialized and verified.

- An intermediate multi-slice PR carries the bundle under `in_progress/<work-id>`
  with `phase != complete` and preserves incomplete later slices, dependencies,
  acceptance, review, and merge state truthfully.
- Each subsequent trunk-based slice starts from current `main`, freezes the next
  canonical snapshot, and replaces the earlier snapshot rather than branching bundle
  truth.
- The designated final implementation PR carries the canonical `phase=complete`
  projection under `completed/<work-id>` and removes the matching `in_progress/`
  path. Its exact final slice remains `pr_open`: pre-review work and acceptance are
  complete and locked, while merge is truthfully pending. Under
  `final_pr_design_gate`, exactly the final human design acceptance remains `pending`
  and unlocked and `completion_binding` remains null; every other criterion is passed
  or waived and locked. The live post-review lifecycle records the typed human result
  before the final PR may merge or the live bundle may complete. All other slices are
  merged or superseded, blockers are empty, every other task lineage is quiescent, and the
  selected implementation task remains the sole active task with its review unbound
  until that same child publishes the snapshot-bearing terminal handoff.
- A single-slice implementation PR is its designated final PR and therefore carries
  that truthful pre-merge completed snapshot itself.

Create the final projection from the exact frozen coordinator boundary in an isolated
coordinator checkout. Invoke `transition --phase complete --premerge-final-slice
<PR-*>` there, move the projection from `in_progress/` to `completed/`, and commit it.
The live coordinator bundle remains separate and may receive later required receipt,
review, PR, or quiescence facts without changing the approved projection.

Before resuming the implementation child, persist the exact tuple in the live
coordinator bundle:

```sh
"$WORK_STATE" authorize-pr-snapshot \
  --work-bundle <live-coordinator-bundle> \
  --expect-revision <revision> \
  --id <PR-*> \
  --snapshot-commit <coordinator-approved-commit> \
  --snapshot-path <authenticated-lifecycle-path> \
  --classification <intermediate|final> \
  --manifest-digest <64-hex-digest>
```

Review attachment ingestion compares the child-carried tuple to this durable
coordinator record before review ownership begins.

Before review and before merge, invoke:

```sh
"$WORK_STATE" verify-pr-snapshot \
  --work-bundle <authenticated-lifecycle-path> \
  --snapshot-commit <coordinator-approved-commit> \
  --pr-head <exact-candidate-head> \
  --slice-id <PR-*> \
  --manifest-digest <64-hex-digest>
```

The read-only operation authenticates the immutable commit, exact lifecycle and slice
classification, and supplied digest; requires the approved commit to be an ancestor
of the candidate head and both commits to contain the exact same file/mode/content
manifest; and rejects the alternate lifecycle path. The child must merge or directly
descend from the coordinator snapshot commit rather than copy its tree into unrelated
history. It never compares against mutable post-handoff coordinator bytes. Its
`MATCH` result includes the work identity, `intermediate` or `final` classification,
manifest digest, and file count. Missing, stale, inconsistent, wrong-digest, or
dual-path snapshots block review and merge. This result is a deterministic comparison,
not a new lifecycle receipt or state transition.

Review attachment carries this authenticated snapshot separately from ordinary
ownership. `review-lease activate` requires a non-null authenticated snapshot for
every ordinary feature or full bug-fix review. The only null-authorization exception
is an explicit compact fast-fix record. The authenticated lifecycle paths are removed
from the ordinary base-to-head path set before ownership and compact-record
classification, so a canonical `BUG_FIX.md` inside a full completed snapshot cannot
misclassify that bundle as a compact fast fix.

### Per-slice RECEIPT.md

```markdown
# PR-01 Receipt

## Result

- Status: implemented | verified | pr_open | merged | blocked | superseded
- Branch, base, head, and PR: <Observed references.>
- Worktree: <Distinct worktree evidence for the attempt.>
- Plan checkpoint and digest: <Committed SHA and final plan digest.>
- Implementation task: <attempt, task ID, task worktree, bound head>.
- Review task: <attempt, task ID, task worktree, exact bound head, native result>.

## Implementation

- Routing: requested <model/effort>; effective <model/effort or unknown>;
  binding <codex_task_request | automatic | unknown>; deviations <details or none>.
- Changed paths: <Actual paths.>
- Summary: <What changed and why.>
- Task tree: <Delegated scopes, accepted receipts, and quiescence evidence.>

## Verification

- <Command or skill gate, result, and artifact.>
- Base-refresh verification: <Current origin/main or upstream-head evidence.>
- UI verification: <receipt path, task stage, worktree tree or committed head/PR, and verdict.>

## Acceptance

- AC-001: passed | failed | waived - <evidence path or explanation>

## Review and Deviations

- <Separate-task current-head gitnexus-pr-review/CI result.>
- <Owned-path or plan deviation, or none.>

## Follow-ups

- <issue URL> | fingerprint <SHA-256> | type bug/documentation/enhancement/harness |
  verified state OPEN/CLOSED | fingerprint comparison exact | severity P1/P2/P3 |
  labels verified | disposition created/reused-open/created-after-closed/closed-complete |
  labels reconciled yes/no |
  source <GitNexus receipt or root connector refetch evidence>, or none.
```

Receipts carry explanations. JSON records operational slice/task bindings and states,
criterion digests, and relative evidence paths only. The root writes receipts after
inspecting task results and system facts.
For UI slices, follow `ui-gate-ownership.md`. `DEFERRED_TO_PR_FINAL` may make an
intermediate PR merge-ready or a final PR publication/review-ready; it never closes
the final design criterion or makes the final PR merge-ready.
The slice task returns typed UI payloads while paused and never writes this bundle.
The root atomically persists the proof generation, validates it against the separate
slice worktree tree, then may accept the receipt and quiesce the task tree.
For the final design AC, the root records both the typed receipt and validator-issued
`final-design-acceptance` attestation via `record-acceptance --ui-attestation`.
Immediately before `complete`, rerun completion validation and pass its current
sidecar to `transition --ui-attestation`; generic evidence cannot satisfy either gate.
`work-state` remains stdlib- and Git-independent for generic lifecycle operations.
Only designated final-UI acceptance, merge, and completion conditionally delegate
evidence verification to `project-ui-verification`, whose typed validator resolves
commit/tree facts without rewriting the recorded attestation.

## Planning and routing receipts

Create `discovery/planning-routing.md` before the first Plan dispatch. Follow
`model-routing-and-delegation.md` for the pushed specification/source binding, the
zero-pass reservation digest, completed response capture, and parent evidence
handoff. The generic helper owns current specification/source identity, accepted
planning evidence, the typed blind-completeness requirement and any required audit,
and blockers.

Once a Pro consultation exists, the deterministic lifecycle bridge creates
`discovery/planning-continuation.json` at that exact path and is its only writer. The
sidecar begins at revision `0` with work identity, repository, the current
attempt-1 consultation reservation, an empty `passes` array, a null
`parent_cursor`, and its timestamp. Successful attempt-1 `capture-response`
advances it to revision `1` with one immutable consumed response pass and parent
cursor. Deterministic invalid-response recovery preserves that pass as
invalidated and advances revision `2` with one derived expected-successor cursor;
it creates no lease. Successful attempt-2 capture advances revision `3` with
the immutable predecessor and current consumed successor. No third attempt is
representable. The sidecar never contains prompt text, task-local receipt paths,
task/thread or heartbeat details, conversation URLs, work phase, or blocker
projections. Validate it against the planner's
`planning-continuation.schema.json`.

The sidecar owns only parent planning continuation. `state.json` remains the sole
operational authority for feature phase, revision, blockers, Delivery, slices, and
tasks. After each bounded claimed attempt, capture only the completed response
from that attempt's canonical conversation and only after a fresh typed completed-generation
result. Browser observations and root polling are not wait authority. Never use
`Answer now`, `Stop answering`, or an equivalent early-stop control without
exact user authorization for that action in the current conversation. On
capture failure, record `BLK-PRO-UNCERTAIN-SEND` in `state.json` and stop before
blind audit or Delivery. On a second deterministically invalid response, record
`BLK-PRO-INVALID-RESPONSE-EXHAUSTED` and stop.

## Parallelization assessment

Use three layers; do not claim more enforcement than each provides:

1. The root owns the semantic decision in `PLAN.md`: APIs, schemas, documents,
   fixtures, backend resources, UI acceptance, and independent proof.
2. The state helper checks declared dependencies, wave membership, execution mode,
   path-prefix overlap, branch uniqueness, and base equality atomically.
3. The root checks actual Git worktrees, base commits, changed paths, builds, tests,
   PR heads, review, and CI outside the helper.

Treat an owned path as a file or directory prefix. Do not use globs, absolute paths,
`..`, generated aliases, or an umbrella directory that hides a narrower conflict.
Include every production, test, fixture, generated, documentation, and configuration
path the task may change.

## Plan support

Discuss remains root-only. In Plan, optionally delegate a bounded, non-overlapping,
read-only question when fresh context or parallel evidence has independent value.
Give it exact sources, a stop condition, and a compact receipt; internal delegation
has no named workflow role or route. The repo-local ChatGPT Pro planner captures one
completed canonical response through its deterministic bridge. Follow
`model-routing-and-delegation.md` for checkpoints, evidence, and blocker handling.
The root alone writes semantic or lifecycle state.

## Coordinator and worktree contract

- Keep plans, discovery, decisions, acceptance, receipts, `state.json`, and
  `STATUS.md` in the spec-work coordinator worktree.
- Before the Pro primary, let the planning owner probe the capabilities required
  for that send, create the planning-routing receipt, and author the complete
  coordinator specification. For a fresh Plan,
  atomically bootstrap its complete slice, task, pair, and acceptance projection with
  `adopt-plan` and `active_response_digest: null`, then run public validation. Commit
  and push that decision-complete projection with
  `docs(spec): checkpoint <work-id> for Pro planning`. Verify the exact remote
  branch/commit and path visibility, then record that digest/commit through
  `record-specification`. This checkpoint opens no PR. Bootstrap stores no
  specification/source or Pro authority and cannot authorize Delivery or task launch.
- Submit once in ChatGPT Pro, consume current task-local typed wait results, and
  capture the completed response through the lifecycle bridge under its receipt-
  bound unreconciled revision. Record only the bridge-owned immutable response and current
  context manifest as planning authority. `record-pro-primary` validates the
  original response before annotations or repairs. Requested headings are
  readability guidance, not a protocol schema. Preambles, epilogues, extra or
  mixed-depth headings, tables, and equivalent organization are admissible when
  the root judges the immutable response useful for planning. The root may
  synthesize useful content into canonical artifacts without changing the
  source evidence. The bridge records ordinary accepted evidence from the
  original producer path and digest without publishing presentation-correction
  state. The continuation remains bound to that original response.
- The lifecycle bridge classifies before successor reservation.
  `valid` produces no lifecycle mutation, successor, send intent, send,
  resend, or stage restart. The root uses `--semantically-unusable` only for a
  content-based decision that the response cannot support planning; formatting
  never authorizes retry. Only `restart_or_fail_closed` may reserve one
  append-only attempt-2 successor. A terminal attempt-2 semantic decision is
  persisted before exhaustion is returned, so repeating classification cannot
  admit the rejected immutable response. Recovery rejects stale revisions, identity
  substitutions, reconciled-send reuse, valid-response invalidation, duplicate
  reservation, and conflicting orphan state without mutating canonical
  authority.
- When capture fails, consume the lifecycle bridge's
  `BLK-PRO-UNCERTAIN-SEND` result through `work-state set-blocker` and stop.
- Record the canonical task roster, requested routes, and slice/predicate-bounded
  replacements in the primary specification, then carry their exact structured
  values in the complete `adopt-plan` proposal. The spec-work request is standing
  authority for these plan-bounded tasks; never ask the user to approve the roster,
  writing, replacement, or route.
- Checkpoint the final plan before Delivery dispatch. Each mutating slice and its
  exact-head review use
  different harness-native user-visible project-worktree tasks;
  root/same-directory/fork/internal-
  subagent substitution and peer attachment are invalid.
- The root alone owns the bundle, accepted evidence, merge, and completion. A slice task
  owns only its checkpointed plan, branch/worktree, delegated descendants, and PR.
- Follow the running harness's task bridge (`codex-task-bridge.md` under
  Codex, `claude-task-bridge.md` under Claude Code) for route requests, direct
  serial launch, supervised
  review preparation/Git attachment/activation, parallel implementation activation,
  scope expansion, publication, branch release, replacement, and the one terminal
  callback.
- Re-observe every task, worktree, Git, PR, check, and quiescence fact before state writes.

Different features may use different coordinator worktrees concurrently. Never share
one active bundle across roots, add a repository-wide lifecycle registry, or modify
project-wide agent depth/thread settings for this workflow.

## State tool

Set the helper once when useful:

```sh
WORK_STATE=.agents/workflows/spec-work-orchestrator/scripts/work-state
WORK_GIT_BINDING=.agents/workflows/spec-work-orchestrator/scripts/task-git-binding
REVIEW_LEASE=.agents/workflows/spec-work-orchestrator/scripts/review-lease
```

Both executable helpers fail fast unless the resolved interpreter is Python 3.11+ and
use standard-library modules only. Invoke them directly; `uv` supplies no dependency,
inline environment, or pinned interpreter for these commands. The canonical state helper locks
the bundle directory around initialization, revision-checked mutations, and status
rendering, so the revision precondition is a real process-safe CAS. The adjacent JSON
Schema is a maintainer and test oracle, not a runtime package or network dependency.
Mutations validate and atomically commit canonical `state.json` before refreshing the
generated `STATUS.md` projection. A rejected JSON replace/fsync restores the prior
canonical bytes and does not touch status. After JSON commits, a status-write fault
cannot roll back or ambiguously reject the canonical mutation; the command returns the
new canonical result with a structured `status_projection_write_failed` recoverable
warning. Initialization uses the same ordering.
It stores the current specification/source binding; append-only Pro evidence and
applicability records; root-observed main SHA; structured task authority and
pair-assessment bindings; launch/probe/stop/release provenance; and acceptance
producer/head provenance. Semantic applicability, replacement, and pairwise rationale
stay in Markdown; the executable route string and replacement-predicate array stay in
structured state. The bundle-wide `observation_epoch_high_water` never decreases, so
clearing invalidated live evidence cannot make a later typed producer reuse an accepted
epoch.

Bundles use only `spec-workflow-state`. Unsupported state is inert history; the
helper rejects it with archive-or-restart guidance and never copies, migrates,
reinterprets, or rewrites it in place.

Initialize after creating or reviewing semantic artifacts:

```sh
"$WORK_STATE" init \
  --work-kind <feature|bug_fix> \
  --work-bundle <bundle-parent>/in_progress/<work-id> \
  --work-id <work-id>
```

Before the first primary, bind the exact decision-complete source:

```sh
"$WORK_STATE" record-specification \
  --work-bundle <bundle> \
  --expect-revision <revision> \
  --specification-digest <64-hex-digest> \
  --source-commit <40-hex-sha>
```

After a successful lifecycle capture, record the current primary through:

```sh
"$WORK_STATE" record-pro-primary \
  --work-bundle <bundle> \
  --expect-revision <revision> \
  --answer-artifact discovery/chatgpt-pro-primary.md \
  --response-digest <64-hex-digest> \
  --repository <owner/repository> \
  --source-branch <branch> \
  --manifest-artifact discovery/chatgpt-pro-context-manifest.json \
  --manifest-artifact-digest <exact-json-sha256> \
  --manifest-digest <provenance-digest> \
  --contract-disposition <clean|accepted_with_annotations|repaired_addendum|repaired_replacement>
```

`record-pro-primary` independently validates the context manifest, continuation pass,
conversation anchor, exact response bytes, and current specification/source binding.
Tampered or incomplete caller-authored projections cannot satisfy those bindings.
Pro evidence carries no response-heading correction fields. Every record must already
match the exact current field set; omitted, retired, or additional fields fail closed
without rewriting state. Substantive `repair_artifact` and `repair_digest` evidence
retains its current disposition and continuation behavior.
Completed Pro evidence remains append-only when the user explicitly stops
further sends. The only alternate current Plan authority is the paired
`adopt-plan --pro-primary-waiver-reason <reason>
--pro-primary-waiver-blind-receipt <json-path>` transaction. It is eligible only
when the latest current applicability receipt already authorizes a fresh primary
for a material or uncertain change. When final user-directed synthesis changes
approved semantic bytes after that terminal receipt, the proposal source commit
must be locally available, descend from the receipt's target source, and contain
the exact final approved inventory. The transaction then advances the existing
current specification/source pair and binds the immutable active response, final
specification digest, final source commit, post-adoption canonical Plan digest,
and fresh CLEAN blind receipt digest; it installs `required_user_requested` and
the receipt atomically. Any mismatch rejects with canonical state and projection
unchanged. Successful Plan-to-Delivery revalidates the final source ancestry and
bytes, then consumes the one-shot waiver; completed Pro evidence remains
unchanged.

The paired transaction also accepts
`--pro-primary-waiver-disposition captureless_pro_recovery` for a canonical
zero-pass continuation and `source_access_recovery` for a canonical bounded
invalidated-primary continuation whose specification and source commit match
the proposal. Captureless recovery requires an explicit user reason and fresh
CLEAN blind receipt. Source-access recovery requires the fresh CLEAN blind
receipt and records the canonical coordinator exception without a renewed user
reason. Both keep `active_response_digest` null and `evidence` empty, and bind
`required_local_plan_recovery` plus the exact continuation digest alongside the current
specification, committed approved-source bytes, canonical Plan, task authority,
and blind receipt. The local authority remains present through ordinary
Delivery mutations; returning to Plan clears it. No second state or inferred
source-access classification is permitted.

`record-pro-applicability`, `record-blind-requirement`, `blind-context`, and
`record-blind-completeness` retain their current evidence and quiescence gates. A
valid applicability receipt follows the existing direct path with nullable
`original_receipt`, `original_receipt_digest`, `correction_receipt`, and
`correction_receipt_digest` fields all null. If strict validation fails solely on
`toSourceCommit`, the root may supply
`--source-observation <bundle-relative-context-manifest.json>`. The manifest must
match the active evidence repository/branch, bind checked-out HEAD to one live
verified remote branch head, and exactly cover current approved source bytes.
The candidate must reproduce the receipt's changed paths, evidence anchors, and
current specification bytes. The assessment epoch remains the monotonic freshness
boundary.

Successful target reconciliation stores the immutable caller receipt in
`original_receipt`, a canonical derived effective receipt in the existing `receipt`
field, and a `spec-work-artifact-correction` binding both. Every other typed field and
the generated producer identity remain byte-for-byte or value-for-value unchanged.
Equal derived/receipt bytes are an idempotent replay; a conflicting pre-existing
artifact fails closed, and unbound orphans grant no authority. Every current
applicability record supplies all four nullable lineage fields explicitly; retired
shapes are not interpreted.

A current exact-digest Pro plan records `not_required_exact_pro_plan` and needs no blind
receipt. Every `required_*` decision requires a fresh clean receipt for the current
plan digest. The typed decision is stored together with that exact canonical plan
digest and becomes stale when the plan bytes change, even if the Pro evidence is
otherwise unchanged. New Pro evidence or material applicability invalidates both the
decision and its prior receipt.

State has one exact current `spec-workflow-state` shape. Reads never derive missing
fields, supply defaults, normalize retired representations, migrate evidence, or
rewrite canonical JSON. There is no v1/v2 identity, compatibility interpretation,
parallel state document, or malformed-state repair path.

### Public validator recovery inventory

This machine-readable inventory is the command-to-matrix completeness oracle.
Every parser command has a boundary record. Its
`currentAmplificationCost` names the maximum producer or verification stages
that a validation failure could repeat; an empty list grants no restart,
replacement, or repeat authority. Every failure without a named
`recoveryOverride` inherits `defaultFailurePolicy`. A named validator override
must supply the complete audit row: failure class, current amplification cost,
authoritative recovery sources, safe correction rule, and residual fail-closed
condition.

<!-- recovery-inventory:start -->
```json
{
  "defaultFailurePolicy": {
    "failureClass": "restart_or_fail_closed",
    "authoritativeRecoverySources": [
      "canonical command inputs",
      "bridge-owned state",
      "immutable typed artifacts",
      "required live provider observations"
    ],
    "safeCorrectionRule": null,
    "residualFailClosedCondition": "Any validation failure without an explicit validator override remains rejected and grants no new mutation, send, successor, replacement, restart, or repeat authority."
  },
  "validators": {
    "pro-applicability-to-source-commit": {
      "failureClass": "reconcile_authoritative_state",
      "currentAmplificationCost": ["pro-primary", "blind-audit"],
      "authoritativeRecoverySources": [
        "active immutable Pro evidence",
        "canonical planning state",
        "checked-out HEAD",
        "configured GitHub remote",
        "fresh live remote branch head",
        "current approved source bytes",
        "recomputed changed paths and evidence anchors"
      ],
      "safeCorrectionRule": "Change only toSourceCommit when every non-target field already validates and all authoritative sources prove one candidate; preserve the original receipt and publish a canonical derived receipt plus correction receipt.",
      "residualFailClosedCondition": "Reject absent or foreign remote binding, live-head disagreement, stale state or epoch, source-byte or diff mismatch, missing anchors, semantic-field failure, multiple candidates, and conflicting immutable publications."
    },
    "pro-response-semantic-admission": {
      "failureClass": "valid",
      "currentAmplificationCost": [],
      "authoritativeRecoverySources": [
        "immutable captured producer response",
        "root semantic usability decision",
        "current context manifest",
        "exact checkout and source commit"
      ],
      "safeCorrectionRule": "Admit root-judged useful planning content from the immutable original response artifact and digest regardless of presentation or source-access prose; verify material cited paths and repository claims against the exact checkout and source commit before synthesizing verified content into separate canonical planning artifacts, and exclude or explicitly annotate unsupported claims.",
      "residualFailClosedCondition": "Reject non-UTF-8 or empty capture, digest, manifest, repository, branch, source-commit, or provenance mismatch, or an explicit root-owned semantic_response_unusable decision; never derive unusability from headings, tables, wrappers, fences, prose organization, or source-access wording."
    }
  },
  "bridges": {
    "pro-lifecycle": {
      "bind-conversation": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "capture-response": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "claim-send": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "commit-send": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "init": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "normalize-conversation-url": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "reconcile-wake": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "recover-captured-response": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "record-wait-result": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "reserve-invalid-response-successor": {
        "currentAmplificationCost": ["pro-send-wait-capture"],
        "recoveryOverrides": ["pro-response-semantic-admission"]
      },
      "show": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "validate": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "validate-continuation": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      }
    },
    "work-state": {
      "adopt-plan": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "adopt-review-head": {
        "currentAmplificationCost": ["exact-head-review"],
        "recoveryOverrides": []
      },
      "authorize-pr-snapshot": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "blind-context": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "clear-blocker": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "init": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "observe-main": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "terminal-clean-delivery": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "ready": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "reopen-final-ui-remediation": {
        "currentAmplificationCost": ["implementation-task", "exact-head-review"],
        "recoveryOverrides": []
      },
      "reauthorize-implementation": {
        "currentAmplificationCost": ["implementation-task"],
        "recoveryOverrides": []
      },
      "reconcile-callback-chain": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "reconcile-merged-delivery": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "record-acceptance": {
        "currentAmplificationCost": ["verification"],
        "recoveryOverrides": []
      },
      "record-blind-completeness": {
        "currentAmplificationCost": ["blind-audit"],
        "recoveryOverrides": []
      },
      "record-blind-requirement": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "record-parallel-assessment": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "record-pro-applicability": {
        "currentAmplificationCost": ["pro-primary", "blind-audit"],
        "recoveryOverrides": ["pro-applicability-to-source-commit"]
      },
      "record-pro-primary": {
        "currentAmplificationCost": ["pro-send-wait-capture"],
        "recoveryOverrides": ["pro-response-semantic-admission"]
      },
      "record-specification": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "record-task-roster": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "refresh-acceptance": {
        "currentAmplificationCost": ["verification"],
        "recoveryOverrides": []
      },
      "render": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "set-blocker": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "show": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "start-wave": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "task-transition": {
        "currentAmplificationCost": ["implementation-or-review-task"],
        "recoveryOverrides": []
      },
      "transition": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "update-slice": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "validate": {
        "currentAmplificationCost": [],
        "recoveryOverrides": []
      },
      "verify-pr-snapshot": {
        "currentAmplificationCost": ["publication-verification"],
        "recoveryOverrides": []
      },
      "verify-publication": {
        "currentAmplificationCost": ["publication-verification"],
        "recoveryOverrides": []
      }
    }
  }
}
```
<!-- recovery-inventory:end -->

The executable completeness test compares both parser command sets with
`bridges`, validates every default and override audit field, rejects unknown
avoided-stage names, and requires every named validator to be referenced by at
least one command boundary. In particular, representation normalization can
avoid one Pro send/wait/capture trajectory, applicability target reconciliation
can avoid at most one fresh Pro primary plus one blind audit, and every
unoverridden category-four failure retains only its already-declared bounded
path.

At every resume, phase, wave, or receipt boundary:

```sh
"$WORK_STATE" validate --work-bundle <bundle>
"$WORK_STATE" show --work-bundle <bundle>
"$WORK_STATE" ready --work-bundle <bundle>
```

Use the returned revision for exactly one mutation:

```sh
"$WORK_STATE" transition \
  --work-bundle <bundle> --expect-revision <N> --phase plan
```

Observe the intended `origin/main` base outside the Plan adoption:

```sh
"$WORK_STATE" observe-main \
  --work-bundle <bundle> --expect-revision <N> --sha <origin-main-sha>
```

After terminal CLEAN, persist the sole unchanged-main comparison before the
external merge boundary:

```sh
"$WORK_STATE" terminal-clean-delivery \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --observed-main-sha <unchanged-origin-main-sha>
```

After interruption, run the same command without `--observed-main-sha`; it
reconstructs the pending merge precondition from authenticated clean lineage
and explicitly reports that no freshness comparison is required. If the atomic
provider returns `HEAD_DRIFT`, pass its create-only result with `--merge-result`.
That receipt must preserve the consumed base and moves the same review task to
current-head revalidation. Once review is CLEAN again, resume without either
optional argument and do not compare remote main again. Pass
`BASE_DRIFT_AFTER_CONSUMED_FRESHNESS` through the same `--merge-result`
argument; its authenticated observed base becomes the consumed base for the
existing merge-based synchronization continuation and invalidates every affected
open slice, again without another remote-main comparison. If `HEAD_DRIFT` also
reports an advanced base, the same receipt consumes both observations and enters
combined base synchronization plus current-head revalidation. Finalized provider
receipts carry the authenticated provider-command digest used by their operation.
The slice's `terminal_freshness` binding is the sole marker that the current
comparison was consumed. Historical CLEAN entries remain review lineage only,
including entries archived when a sibling PR advances the shared base. Every
accepted no-merge result is retained in `premerge_drift` by receipt path, byte
digest, and epoch; state reload reauthenticates the retained typed operation and
fails closed if its file is missing or replaced. A
`PROVIDER_OUTCOME_UNKNOWN` recovery holds an exclusive claim on the exact
reservation through reconciliation, any provider retry, and atomic final
publication, so concurrent recovery cannot overwrite the winner.
New `merged` transitions require the finalized provider receipt path and digest.
Bundle validation may read the exact legacy merge-observation shape only for an
already-persisted merged slice whose stored merge-receipt digest matches the
retained bytes; this compatibility read cannot authorize a new merge transition.

Write the complete Plan as one transient regular JSON proposal outside the bundle.
Use the same exact shape first for a fresh pre-Pro bootstrap and again after current
Pro evidence is applicable:

```json
{
  "specification_digest": "<current semantic specification digest>",
  "source_commit": "<current applicable 40-hex commit>",
  "active_response_digest": null,
  "semantic_artifacts": [
    {"path": "<approved Markdown path>", "sha256": "<exact-byte digest>"}
  ],
  "slices": [
    {
      "id": "PR-01",
      "title": "<title>",
      "plan": "pr/PR-01/PLAN.md",
      "plan_digest": "<exact-byte digest>",
      "depends_on": [],
      "wave": 1,
      "execution_mode": "serial",
      "owned_paths": ["<repo-relative path prefix>"],
      "branch": "codex/<slice>",
      "base_ref": "origin/main",
      "base_sha": "<observed origin/main sha>"
    }
  ],
  "parallel_assessments": [],
  "task_authorization": {
    "entries": [
      {
        "slice_id": "PR-01",
        "role": "implementation",
        "route": "gpt-5.6-sol high",
        "replacement_predicates": [
          "unrecoverable_task_runtime",
          "unrecoverable_worktree",
          "repository_identity_mismatch",
          "pr_identity_unrecoverable",
          "separate_deliverable_user_decision"
        ]
      },
      {
        "slice_id": "PR-01",
        "role": "review",
        "route": "gpt-5.6-sol high",
        "replacement_predicates": [
          "unrecoverable_task_runtime",
          "unrecoverable_worktree",
          "repository_identity_mismatch",
          "pr_identity_unrecoverable",
          "separate_deliverable_user_decision"
        ]
      }
    ]
  },
  "acceptance": [
    {
      "id": "AC-001",
      "criterion_digest": "<normalized criterion digest>",
      "scope": "slice",
      "producer_slice": "PR-01"
    }
  ]
}
```

Arrays use canonical path, slice, pair, task, predicate, and acceptance order.
`semantic_artifacts` is the complete approved source inventory, `slices` is complete,
and `acceptance` contains every current criterion. A parallel wave contains at least
two slices and supplies every sorted pair record with the current left/right plan
digests, canonical assessment digest, and `independent` verdict. For an unbound slice,
`branch`, `base_ref`, and `base_sha` may all be null; otherwise all three are supplied
together. Delivery-owned fields are not proposal properties.

The nullable response field has exactly two modes:

- `null` requests an initial bootstrap and is accepted only in Plan when there are no
  slices, structured task authority, pair assessments, adopted Plan digest,
  specification/source binding, Pro or blind evidence, delivery/completion identity,
  or non-empty acceptance history. It atomically installs the complete projection and
  stores only `current_plan_digest`. Exact replay is non-fresh and rejects without
  mutation.
- A 64-hex digest requests ordinary adoption and must match the latest applicable Pro
  evidence for the proposal's specification digest and source commit. This mode
  retains all existing Plan recovery and re-adoption behavior.

The bootstrap proposal's `source_commit` must still name a real commit, but it is a
well-formed proposal input rather than stored source authority. After bootstrap and
public validation, commit and push the exact decision-complete bundle and use
`record-specification` to bind that checkpoint before requesting Pro planning.

Use the same atomic command for the fresh bootstrap and the later post-Pro adoption:

```sh
"$WORK_STATE" adopt-plan \
  --work-bundle <bundle> --expect-revision <N> \
  --proposal <transient-plan-proposal.json>
```

The helper holds the bundle lock while it recomputes current semantic, acceptance,
plan, and task-authority bindings, checks the expected revision and either the fresh
bootstrap guard or applicable Pro source, merges only Plan-owned fields, and validates
the full slice/task/DAG/wave/
parallel/acceptance kernel. Existing delivery history, task/external identity, heads,
PRs, receipts, merge state, publication, completion, and locked acceptance are carried
forward or reject a conflicting proposal. A changed complete proposal requires every
current nonquiescent task to be quiescent because the proposal reasserts the full
semantic and authorization projection. Accepted unlocked criteria whose digest or
assignment changed reset through the existing pending record. The blind requirement
and completeness proof reset only on an actual adoption. Exact-identical replay returns
`UNCHANGED` for ordinary post-Pro adoption without changing canonical JSON or `STATUS.md`
bytes; a null-digest bootstrap replay rejects because bootstrap authority is single-use
and fresh-only.

Do not assemble or repair a decision-complete Plan with sequential `update-slice`,
`record-task-roster`, `record-parallel-assessment`, or `refresh-acceptance` calls.
Those narrow command surfaces remain available only for their declared non-Plan
lifecycle duties; `adopt-plan` is the public Plan assembly and recovery authority.
When this atomic Plan route applies, put the complete task authorization projection
in the proposal instead of deriving and recording it sequentially.

Bind the complete initial parallel same-wave group before `ready` or the Plan-to-
Delivery transition. `ready` advertises semantic candidates only when the exact initial
`start-wave` predicates pass: all nonterminal same-wave members are selected, pending,
parallel, path-disjoint, dependency-independent, distinctly branched, and bound to one
observed `origin/main` SHA. It does not claim a native task is prepared. Only serial
dependency slices with current-head-clean upstream reviews may appear as `stackable`.
Delivery-only isolated retry remains available beside quiescent verified/open siblings.

Reserve the attempt before calling the user-visible task API. The first mutation has no
external ID; after `create_thread`, bind its returned task/client ID to that reservation:

```sh
"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state creating \
  --launch-marker <feature/PR-01/implementation/attempt> \
  --authorization-digest <64-hex-digest> \
  --plan-checkpoint-sha <40-hex-sha> \
  --adoption-start-sha <attempt-1-start-sha>

"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state creating --client-thread-id <id>

"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state active \
  --thread-id <id> --task-worktree <absolute-path> --bound-sha <start-sha>
```

The initial attempt-1 `creating` command requires and durably binds
`implementation_task.adoption_start_sha`; replacement attempts and review tasks
must carry null. It also resolves `--plan-checkpoint-sha` as a local
commit object, compares that commit's slice-plan blob with the current
authorized `PLAN.md`, and validates the checkpointed `state.json` plus every
declared plan against the current planning snapshot before changing the
revision, attempt, generation, task identity, or any other state. A nonexistent
object, noncommit object, missing state or plan, or snapshot mismatch rejects
with byte-identical `state.json` and `STATUS.md`; the root must not call
`create_thread`.

The plan checkpoint is a full bundle authority snapshot: it contains exact
`state.json` plus every declared slice plan, not only the launched slice's plan.
Same-task reauthorization derives the prior canonical topology from those immutable
bytes.

For direct `RUN_SLICE`, a successful `creating` mutation is the admission token. A
feature blocker rejects it byte-identically, so the root must not create the task.
PREPARE_ONLY reservation remains nonmutating and may be recorded while blocked, but
`start-wave` cannot activate the cohort until all blockers clear.

Never retry an uncertain create. Bind exactly one task/worktree proven by the launch
marker and repository/slice/role; zero or multiple matches remain blocked. A serial or
stacked implementation starts from the initial `RUN_SLICE` prompt and binds directly to
`active`. A review starts once with `RUN_REVIEW` and a digest-bound immutable startup
fact block. The child maps those facts and its live checkout into task-local
`task-git-binding attach-review` and `review-lease activate` calls before semantic
review, remediation, or fan-out. It may correct a retryable startup error in the same
task. The root reconciles the published bundle-relative attachment receipt and records
`creating -> active`; it sends no activation packet and creates no preparation task.

`release`, `adopt-implementation`, `adopt-review-head`, `attach-review`,
`attach-implementation`, and `observe-pre-pr-implementation` require the canonical
absolute bundle root and publish only a no-follow, descriptor-anchored descendant
receipt. All six
publishing operations, plus read-only `verify-review`,
share one process lock for the exact Git common directory and branch across
observation, mutation or verification, receipt publication, compensation, and return.
For a newly created full-fix bundle that retains an already-open fast-fix PR,
`adopt-implementation` has two closed implementation-attempt-1 modes. Ordinary
adoption attaches a clean detached ownerless branch only when the bound head equals
the durable `implementation_task.adoption_start_sha` and descends from the required
base. Required-base-sync adoption accepts an already-attached branch only when the
exact task worktree is its sole owner and the bound head is a two-parent merge with
first parent equal to that durable adoption start, second parent equal to the exact
live required-base SHA, and the deterministic clean merge tree. Both modes prove the
exact live remote head and an open live PR whose number/base/head/branch match, and
publish typed PR/head/base/adoption-start/branch ownership facts. `work-state`
consumes that receipt atomically on `creating -> active`; without it, attempt 1
remains fresh-from-base.
The binding operation itself rejects attempt 2+ or a receipt without at least one
matching non-empty external task identity before receipt publication or checkout mutation.
Mismatched adoption proof rejects without state mutation, and terminal publication
must preserve the adopted PR number. A later owner-recorded base synchronization
retains the immutable adoption-time base in that receipt while advancing the slice's
current integrated base. Superseding an adopted attempt before terminal publication
materializes the authenticated adoption PR/head as replacement lineage before clearing
the task attachment, so the next authorized attempt cannot fall back to the slice base.
Non-success terminal quiescence performs the same retention before clearing the
attachment, preserving that recovery path through later supersession. Authenticated
terminal output advances the retained replacement head beyond the adoption-time start
when the implementation pushed additional commits.

Missed implementation callbacks are reconciled only through public
`work-state reconcile-callback-chain`. The state-first operation admits one exact
current typed terminal followed by that attempt's exact typed branch-release receipt
for every ordinary terminal. The current scope-expansion and satisfied-prerequisite
variants instead retain the branch owner and omit release. It rejects stale, incomplete, reordered, ambiguous, or
tampered chains without mutation. Exact replay of the persisted current chain is a
byte-identical no-op that creates no revision, task, heartbeat, or redispatch.
On first active admission, `--observed-main-sha` applies the current owner's
base synchronization before terminal and release authentication.

Before initial review attachment, `adopt-review-head` is the only recovery for an
ownerless remote PR head that advanced after release. It accepts exactly one
two-parent merge of the recorded head and current descendant main or clean direct
upstream base, requires its tree to equal Git's deterministic clean merge, confines
the final PR delta to frozen owned paths, authenticates the exact causal release
receipt, safely advances a remote-only local branch ref with rollback, advances the
same creating reservation through a typed coordinator receipt, and preserves its
attempt and external identity. The persisted receipt remains valid lineage when a
later material finding reauthorizes the same implementation owner.
Before granting implementation ownership, `attach-implementation` re-observes the
exact live `origin` branch head inside that locked operation and rejects stale startup
facts without switching the checkout or publishing a receipt.
The closed startup fact block carries exact stable identity, revision epoch, remote, base,
and frozen owned paths before any review skill load or fan-out. Attachment provenance
uses the greatest typed release before that attachment epoch from the actual prior
owner across implementation and review history, never an older release merely because
it came from implementation.

Review head rotations remain inside one task-local rolling lease and never mutate the
bundle. Each authorized push advances a compact ledger after deterministic ancestry,
identity, remote, cleanliness, mutation-receipt, and frozen-path checks. Head-bound
semantic, CI, mergeability, and verification evidence becomes stale on every advance.
Immediately before the single terminal callback, the review owner runs one supplied
finalize command and returns its terminal lease receipt for root consumption.
Clean finalization first requires the lease's current base to equal the live remote
base; a newer base must be merged and recorded through the existing typed `base_sync`
advance. Finalization revokes mutation authority before receipt publication; an
interrupted non-authorizing `FINALIZING_*` state may complete only through the same
finalize command. State rejects parent-driven `active -> active` rotation. Creating
and startup-failed attempts have no lease evidence. To resume a same-plan authority-
retaining block, attach the same attempt at the blocked head, transition
`blocked -> active`, and run `review-lease resume` with that fresh attachment and the
current canonical `--base-ref` before any advance or later finalization. When head
adoption preceded the lease, the coordinator retains each authenticated blocked or
clean terminal receipt as a contiguous adoption-to-current lineage before a resume or
newer-base reopen replaces the active attachment; repeated same-attempt resumes
therefore preserve the immutable startup proof across later rolling segments. If the
review task is later replaced through the typed supersession path, the causal adoption
task remains authoritative in review history and same-owner reauthorization archives
that proof with the current replacement task's terminal. A clean lease
may adopt that explicitly supplied normalized base only after remote ancestry
validation; a blocked lease cannot change base identity. A material review-driven
replan instead finalizes blocked, releases the branch, reauthorizes the same
implementation task at the next slice revision epoch, and later resumes this same
review task on the descendant revised head. A terminal review may classify an exact
missing path as `required_by_intent_scope_expansion`; only the coordinator may consume
that classification, deterministically amend plan/path authority, and send the
next-revision attachment with the exact current owned-path inventory and canonical
revision authority digest back to the same review lease. The lease derives authority
from that attachment; it accepts no independent caller-supplied reauthorization
receipt or path declaration. Unsupported active task state uses the existing typed
replacement path rather than task migration.

For the same accepted goal and open PR, `reauthorize-implementation` is the only
Plan-phase bridge into the next revision. `fresh_pro_plan` requires a fresh
exact-digest Pro plan and current typed blind requirement.
`required_by_intent_scope_expansion` instead requires an unchanged accepted
specification, one typed exact-head blocked terminal finding, its newer branch release,
and a deterministic prior-to-revised plan comparison that permits only an additive
owned-path inventory, the exact `no` to `yes` consumer-sweep declaration when needed,
an append to the cited existing acceptance entry, and appended verification bullets.
The cohesion exception consumes the authenticated blocked review terminal only after
it proves the unchanged primary acceptance criterion and owning observable seam, the
exact added owned-path inventory, and the same PR, branch, implementation task, and
review task lineage. Independent execution flows, unrelated production behavior,
another acceptance outcome, or a changed owning seam still require a split. There is
no caller-supplied cohesion bypass: ordinary planning, pre-PR scope expansion, and
direct plan validation cannot manufacture this authority. Both dispositions require
a new checkpoint and roster digest,
quiescent implementation, exact current PR/head/base facts, and a checkpoint whose
bundle-relative slice-plan blob exactly matches the current authorized plan. The
branch release must be newer than the terminal review attachment; an older release
remains causal attachment provenance only. Reauthorization archives the prior
revision and digest-bound terminal receipt, persists the disposition, specification
and acceptance digests, prior owned paths, added paths, and cited criterion, preserves
attempt/thread/client/worktree/branch/PR identity, increments `revision_epoch`, and
puts implementation in `authorized`.
The Delivery-phase exception is `reopen-final-ui-remediation`. It consumes the
canonical current-head BLOCKED final-checkpoint receipt only after the designated
final UI slice has a successful quiescent implementation owner, exact-head CLEAN
review, pending unlocked final design acceptance, and no concurrent mutation. It
archives the clean revision with disposition `final_ui_checkpoint_issue`, retains
the PR/task lineage, invalidates only slice-owned acceptance, clears terminal,
release, and attachment proof, authorizes the same implementation, and blocks the
same review in one revision-CAS mutation. The archived CLEAN rolling-lease terminal
is the causal release for the next implementation attachment; its digest and
observation epoch must match exactly. It does not replan or rotate owners.
After Delivery begins, the root sends `RESUME_IMPLEMENTATION_AFTER_REPLAN` to that same
task with the exact reauthorization facts, plan, and `attach-implementation` command.
The attachment receipt alone permits `authorized -> active`. After revised
implementation releases the new head, the same review task receives `RESUME_REVIEW`,
attaches the next revision, and runs `review-lease resume` with the current canonical
base ref and the exact causal release receipt. When the revision changes frozen
paths, it also supplies the exact
`spec-work-task-reauthorization` receipt; the lease validates the additive inventory
and binds that receipt digest into its identity before adopting the paths. If the base
advanced after the revised head was released, resume preserves
the base actually integrated into the revised head and the task immediately records
the live descendant through the existing merge-based `review-lease advance` path.
Task replacement is reserved for a typed unrecoverable-task predicate or a separate
deliverable. After stop and branch-release proof are recorded, implementation
supersession clears the former owner's live branch-attachment fields.

Before a first commit or PR, an unchanged-checkpoint
`coordinator_scope_expansion` stays in Delivery when the exact required paths
remain inside the terminal's accepted criterion and owning seam without changing
behavior, dependencies, topology, route, runtime authority, or deliverable. The
root revalidates the existing current Pro evidence, current roster, unchanged
checkpoint, exact terminal path set, and typed dirty-manifest observation, then
archives that evidence and authorizes the same task. It does not edit the plan,
enter Plan, send Pro, run a blind audit, replace the task, increment its attempt,
or attach Git again. The literal reauthorization receipt is the only authority
for the same task to activate immediately in the still-current Delivery phase.
Any changed criterion, seam, checkpoint, planning evidence, roster, or other
material boundary rejects atomically and must use the material route.

An unpublished generic `FAILED` implementation remains non-publishing and
non-resumable. A failure blocked only by a separately delivered pull request may use
the closed prerequisite variant, which adds exactly a canonical
`prerequisite_fingerprint`, optional tracking issue, positive prerequisite PR,
prerequisite head SHA, and required base ref. The fingerprint binds those four facts;
the terminal preserves the bound head and carries no implementation PR. Partial or
additional prerequisite fields reject.

After the prerequisite merges, `task-git-binding observe-pre-pr-implementation`
receives its immutable bundle-confined merge receipt. The observer validates the
atomic merge result and transport digests, exact prior-base/prerequisite-head parents,
the live remote required-base head, ancestry from both the prior base and merge commit
to that live head, the unchanged task/branch identity, and the complete current owned
dirty manifest. Issue state and a mutable PR lookup grant no authority.

`reauthorize-implementation --disposition satisfied_prerequisite` consumes that
`pre_pr_prerequisite_observation` only for the exact blocked unpublished failure owner
that retained its physical branch attachment, with no review owner or peer mutation.
This route records the quiescent terminal without branch-release proof. The checkpoint,
canonical plan digest,
specification, acceptance, roster, owned paths, terminal digest, prerequisite
fingerprint, delivery receipt/digest, merge SHA, and observation epoch must match
current authority. One CAS archives the dedicated prerequisite-resume revision,
increments `revision_epoch`, advances the required base, clears only live failure and
release fields, and returns the same task to `authorized` without another attachment.
Its only next action is `merge_required_base_and_rerun_verification`; it cannot add
scope, replace the task, change the attempt/thread/client/worktree/branch, publish a
PR, or reinterpret older failure evidence.

The separate material `pre_pr_scope_expansion` disposition does not create
retained PR lineage. It requires one current quiescent
implementation terminal whose result-dependent exact variant carries sorted
required paths, one existing acceptance ID, the unchanged owning seam, and
nonempty regression and verification obligations. It cannot claim a PR and its
output head must equal its bound head. Other terminal results keep their prior
exact field set.

After fresh Plan authority for that material disposition, the same implementation
task runs
`task-git-binding observe-pre-pr-implementation`. The helper does not assert
remote equality or mutate the checkout. It proves the persisted local bound SHA,
local base ancestry, unique same-worktree branch ownership, complete revised
owned-path containment, and a canonical NUL-safe dirty manifest with separate
index/worktree states, index mode/content digest, and worktree content digest
or explicit absence. Ignored-only rejection is scoped to the revised owned-path
inventory; unrelated ignored repository artifacts do not enter the observation.
The bundle-relative observation receipt must be a JSON path. The receipt's
precursor digest binds the current state revision, terminal, task identity,
checkpoint/plan, roster, revised paths, specification, acceptance, epoch, and
manifest. State CAS revalidates those facts, archives the terminal and
observation path/digests as the closed pre-PR revision-history variant, advances
the global evidence epoch, and binds the observation path/digest into the final
reauthorization digest. Material reauthorization preserves the typed blind
requirement against the revised plan and clears prior blind completeness; a
required decision therefore needs a fresh complete audit before Delivery.

The archived disposition is the activation-mode discriminator.
`authorized -> active` for this variant consumes no second Git attachment and
preserves task, attempt, thread/client identity, worktree, branch, and dirty
bytes. An observation receipt rejected by CAS remains immutable but
non-authorizing; any later state, task, epoch, branch, base, path, or manifest
change makes it non-replayable. All task descendants and supported writers must
remain quiescent between observation and CAS. Parallel peers must also remain
quiescent and disjoint with unchanged wave/dependency topology.

Base movement remains with the current original branch owner. Before implementation
release, the same active implementation task integrates current main and returns
terminal evidence plus `--observed-main-sha`. When the remote base advances during
review startup, the creating review activates against its recorded ancestor base and
then immediately performs the merge-based base sync through `review-lease advance`.
The root records the new base with terminal evidence, not on `creating -> active`.
The same review task owns later base sync through that lease path. These current-owner
transitions update the synchronized base while preserving the owner's task identity
and invalidating other affected quiescent slices. Stale base alone is not a
replacement predicate.

Every roster replacement predicate is exactly one of
`unrecoverable_task_runtime`, `unrecoverable_worktree`,
`repository_identity_mismatch`, `pr_identity_unrecoverable`, or
`separate_deliverable_user_decision`. Correctable startup facts, base movement, and a
same-goal material replan cannot be registered as replacement reasons.

Only a complete eligible group of at least two parallel implementation tasks uses
`prepared`, including every nonterminal member of a same-wave retry cohort, followed by
one atomic activation and one ACTIVATE message per task:

```sh
"$WORK_STATE" start-wave \
  --work-bundle <bundle> --expect-revision <N> \
  --slice PR-01 --slice PR-02
```

Use `update-slice` for later status, receipt, binding refresh, and PR facts. If
verification succeeds, record `--receipt <path> --head-sha <40-hex-sha>` with
`verified`; `verified`, `pr_open`, and `merged` require both. Binding changes are
accepted only while a slice is `pending` or `blocked`. Invalidating an attempt clears
its receipt, PR, head, and every acceptance record whose provenance names that slice,
even if the current plan omitted or misassigned it. Unrelated criteria remain. If
`ACCEPTANCE.md` gains IDs, normal validation stops until the root records one of the
new IDs as `pending`; that mutation synchronizes every newly declared ID as pending.
If an existing block changes during current Plan assembly or recovery, put the changed
criterion and its assignment in the complete proposal; `adopt-plan` atomically stores
the new digest and resets only the affected unlocked record to pending. For non-Plan
lifecycle recovery, ordinary commands remain
blocked until every affected task is outside `creating`, `prepared`, and `active` and
the root runs revision-checked `refresh-acceptance --id AC-NNN`; refresh performs that
same selected pending reset. `record-acceptance` accepts only confined regular
non-symlink evidence files and hashes their bytes before the state mutation. Every
state read rehashes evidence and fails closed on missing, replaced, or changed content.
Refresh never replaces semantic plan invalidation, a fresh typed blind decision and
any required audit, or re-verification, and rejects locked records. Attempt invalidation
preserves locked records. Use `set-blocker` and `clear-blocker` for their named facts.
Canonical reads, including `show` and `ready`, remain usable when `STATUS.md` is
missing, stale, or malformed. `validate` and `render` recreate the deterministic
projection, and every successful mutation refreshes it after committing JSON.
Completed-bundle `verify-publication` remains fail closed and requires the
projection to exist and exactly match canonical state before reporting reachability.
The root is the only caller allowed to mutate state or render status.

## Parallel and stacked execution

### Parallel wave

Before `start-wave`, prepare every user-visible implementation task in a distinct clean
worktree at the recorded `origin/main` SHA. Select at least two tasks and the entire
nonterminal initial wave or retry cohort; partial preparation may persist, but partial
activation is rejected. Send ACTIVATE only after the atomic state mutation succeeds.
A retry launches directly only when every same-wave peer is terminal or quiescent;
never call singleton `start-wave` or peel a member from a mutating retry cohort.

Let slices progress asynchronously through `in_progress`, `verifying`, `verified`,
and `pr_open`. Their state writes still fan in through the root and revision CAS.
If one slice fails, block or remediate that slice without rewinding an independent
sibling. A failed singleton may restart beside same-wave `verified`/`pr_open`
siblings; any mutating, pending, or blocked sibling keeps restart closed.

When one member merges, the helper records the newly observed main SHA and reopens
each remaining open parallel sibling in its existing review task. It preserves the
slice status, PR, exact head, implementation task, review task, and worktrees while
clearing stale acceptance, clean-review, and attachment proof. The same review task
reattaches or resumes its lease, merges the refreshed base into its existing head,
pushes the rotated head to the same PR without force, and reruns review and
verification. Stale base alone creates neither replacement lineage nor a fresh task
attempt. A fresh parallel retry cohort remains available only after an independently
typed replacement predicate makes multiple task replacements necessary.

When the highest planned wave is parallel and terminal for the bundle, hold its
lexicographically last slice as the final snapshot carrier before that implementation
publishes its ordinary `pr_open` attention handoff. Other same-wave slices may
materialize intermediate snapshots, quiesce, review, and merge while that carrier
remains active. Those merges may advance observed `origin/main` without invalidating
the held carrier. After the other members are terminal, the carrier's current owner
performs the ordinary merge-based base synchronization and only then publishes the
ordinary head/PR attention handoff. The root freezes the completed projection, the
carrier materializes it, and the same owner publishes the terminal handoff. No recovery-only
snapshot PR or replacement task is created.
Carrier identity is topology-derived and persists independently of task state. The
carrier cannot terminalize successfully before its active `pr_open` attention handoff,
so the ordinary early-terminalization path cannot erase these publication guards.

Start an ordinary serial slice only after every earlier slice is terminal. The
stacked rule below is the sole exception that permits a downstream serial lifecycle
to coexist with an open upstream PR.

### Stacked slice

Treat a pending slice as `stackable` only when exactly one direct dependency is
`pr_open` and every other direct dependency is `merged`. Start it alone with
`execution_mode: serial`, after every open upstream implementation is quiescent and its
separate review task is clean at the current head, with
`base_ref == upstream.branch` and `base_sha == upstream.head_sha`.

If upstream review/remediation resumes, first quiesce affected downstream task owners.
The state interlock rejects simultaneous dependency-edge task activity. When the
upstream merges, the helper preserves an open downstream PR and reopens its existing
review task while normalizing its required base to current `origin/main`. That review
task resumes the lease, performs the base sync on the same PR branch, and reverifies.
A dependent slice cannot become `merged` until every dependency is merged and its
base ref is `origin/main`.

## Activation and closure

Use this launch contract only with canonical executable state. Unsupported active work
must be archived or restarted manually. In-place persisted-state normalization never
migrates or revives unsupported active task state.

The designated final implementation PR's frozen projection may transition to
`complete` before its own merge only with `--premerge-final-slice <PR-*>`, exactly one
`pr_open` slice matching that argument, every other slice merged or superseded,
every other task lineage quiescent, the selected implementation task as the sole active
task with its review unbound, and no blockers. Every declared acceptance criterion is
passed or waived and locked except that a `final_pr_design_gate` projection leaves
exactly its final human design criterion pending and unlocked with null
`completion_binding`. Completion preserves those states while the final slice's
`pr_open` value remains the truthful merge-pending fact for that candidate head.

For the later live coordinator lifecycle, after all slices are merged or superseded
and every declared acceptance criterion,
including final design acceptance when applicable, is passed or explicitly
waived, the root re-observes current repository and GitHub facts and asks the helper
to transition to `complete`. The helper consumes the already recorded typed slice,
review, merge, acceptance, UI, and task-lineage evidence directly; do not create a
second model audit or completion receipt. The published lifecycle move belongs to the
isolated pre-merge projection in the designated final implementation PR. After merge,
run `verify-publication` against the fetched default-branch commit from an immutable
checkout of that projection; do not compare it with later live coordinator receipt,
review, PR, or quiescence writes. Terminal completion and wait shutdown require its
`REACHABLE` result. The helper does not
archive, commit, push, review, or merge.
The normal publication carrier is the designated final implementation PR. Every
implementation PR must pass `verify-pr-snapshot` at its exact candidate head; a
missing snapshot blocks that PR and never creates authority for a recovery
documentation-only PR.
Every on-disk read deeply revalidates passed automated UI receipts through the
repository's `project-ui-verification` contract and the proof route discovered
for the active work. Completed UI
bundles additionally rerun `validate-receipt` in `completion` mode against the stored
binding and sidecar; hand-edited, missing, fabricated, or stale proof fails closed.

Completion locks every passed/waived acceptance record in the same revision-CAS
transaction. Live complete state requires every current record locked, passed/waived,
and bound to current criterion and evidence bytes. A frozen pre-merge final-UI
projection has the single human-gate exception above; all other frozen complete
projections use the live acceptance rule. A frozen pre-merge projection also
permits exactly its designated `pr_open` final slice; merge remains a later external
fact. A bundle under `completed/<id>` remains
in `phase=complete`; never invoke `--reopen` against that completed-root path.

To reopen, the root first performs a normal Git move from
`<bundleParent>/completed/<id>` to `<bundleParent>/in_progress/<id>` while the persisted
phase remains complete. Validate the moved bundle at its active path, then invoke
`transition --phase discuss --reopen` there. The helper never moves a directory,
archives or unarchives a bundle, commits, or performs any Git mutation. If validation
or reopen fails, `state.json` and generated `STATUS.md` remain unchanged. While the
bundle is still complete, correct the precondition and retry under `in_progress`, or
move it back to `completed`; no partial phase change is accepted. Reopen preserves
locked historical status, evidence paths and digests, criterion digests, and referenced
receipt bytes. Continue reopened work only with new unlocked `AC-*` and new `PR-*` IDs.
