# Spec-Work Orchestrator Core

Load this forced-only workflow only from `spec-driven-feature-orchestrator` or
from `project-debugging` after its classifier returns `fix_full`. It has
no frontmatter and is not a discoverable skill. Carry one descriptor-bound work
bundle through Discuss -> Plan -> Delivery -> Complete while keeping the
coordinator root as the sole semantic and lifecycle writer.

## Invocation descriptor

Require the caller to establish this exact in-memory descriptor before using
the core. Do not create a second manifest file.

```text
workKind: feature | bug_fix
bundleParent: docs/features | docs/bug_fixes
primaryArtifact: FEATURE.md | BUG_FIX.md
planningGate: feature_discovery | confirmed_root_cause_and_regression_abstraction
```

Validate the closed combinations:

- `feature` -> `docs/features` -> `FEATURE.md` -> `feature_discovery`.
- `bug_fix` -> `docs/bug_fixes` -> `BUG_FIX.md` ->
  `confirmed_root_cause_and_regression_abstraction`.

Use `<bundleParent>/in_progress/<YYYY-MM-DD-slug>/` for active work. The
basename is the `work_id`. GitHub issues do not determine bundle identity;
a `feature` bundle nonetheless carries exactly one mandatory owning issue
recorded in `FEATURE.md`'s `Source / Issue` (per
`references/github-board-sync.md`), and a `bug_fix` bundle carries its
verified defect owner when the debugging route established one.

## Authority and deterministic bridge

- Treat the primary artifact, `INTERVIEW.md`, `DECISIONS.md`,
  `ACCEPTANCE.md`, discovery notes, slice `PLAN.md`, and `RECEIPT.md` as
  semantic authority.
- Treat `state.json` as operational authority and `STATUS.md` as its
  recoverable generated projection. Canonical reads do not require the
  projection to exist or match. A mutation commits JSON first, then refreshes
  status; a status-write fault returns a structured recoverable warning without
  making the committed canonical mutation appear rejected. Never hand-edit
  either JSON or generated Markdown.
- Treat `discovery/planning-continuation.json` and the task-local Pro receipt as
  the bounded authorities described by the Pro lifecycle reference; neither
  may override state phase or blockers.
- Keep secrets, credentials, signed URLs, raw external payloads, ChatGPT URLs,
  heartbeat IDs, and task-local receipt paths out of repository artifacts.
- Keep authority descriptor-, plan-, and route-bounded. It excludes unrelated
  working-copy changes and actions outside the registered slices and routes.
- Treat wrong or contradictory commits, inaccessible required sources, invented
  material claims, substantive omissions, accepted-decision contradictions, and
  unsafe ambiguity as blockers. Annotations and internally consistent receipts
  cannot cure them.

Use the standard-library bridge for every state read and mutation:

```sh
WORK_STATE=.agents/workflows/spec-work-orchestrator/scripts/work-state
"$WORK_STATE" validate --work-bundle <bundle>
"$WORK_STATE" show --work-bundle <bundle>
```

Reload state and pass `--expect-revision` before every mutation. On a stale
revision, reload and reconsider. The bridge owns local deterministic facts and
the read-only completed-bundle/default-branch publication comparison; it does
not change Git or inspect GitHub, builds, tests, UI runtimes, APIs, tasks,
reviews, or merges.

The bridge accepts only the single `spec-workflow-state` schema identity. It
validates only the exact current document shape and never supplies omitted fields,
rewrites an older representation, or interprets retired receipts. Never introduce a
versioned schema identity, migration, compatibility path, parallel state file, or
default for missing current authority; every mismatch rejects before mutation.

After bundle initialization, and after every successful bridge mutation that
changes phase or records or clears a bundle blocker, the root syncs the
GitHub oversight projection per `references/github-board-sync.md`: the
bundle's owning issue as the board card and its lifecycle Status column.
The `complete` transition itself triggers no sync; `Complete` is set only
in the completion step, after `verify-publication` returns REACHABLE. The
projection is never authority; a sync fault is a structured recoverable
warning, and board identifiers never enter `state.json`.

Read `references/work-bundle-contract.md` before creating or resuming a bundle,
changing semantic artifacts, adding slices, assessing parallelism, starting a
wave, or closing. It owns templates, bundle invariants, and helper commands.

## Discuss

Read `references/interview-and-doc-authority.md` and
`references/domain-modeling.md` when entering/resuming Discuss, closing a
question, or when Plan exposes ambiguity.

1. Inspect durable docs, current code/tests, required live external evidence,
   and relevant history before asking questions.
2. Record goal, scope, non-goals, document authority, and open decisions in the
   primary artifact. Always create `INTERVIEW.md`; it may contain zero user-
   asked questions only when the request and inspected evidence settle every
   consequential decision, recorded as closed gray-area entries. An empty
   ledger is not interview completion.
3. Ask only unresolved consequential behavior, product, acceptance, or
   architecture questions, worked as design-tree frontier rounds. Keep them
   root-only and recommendation-led.
4. Close each answer across interview, decisions, primary spec, acceptance,
   affected plans, canonical docs, and blockers in one semantic update.
5. Remain in Discuss while any blocking decision is open or the question
   frontier is nonempty. Transition to Plan only when intent and acceptance are
   understandable.

For `bug_fix`, create `discovery/root-cause.md` and
`discovery/regression-test-abstraction.md`. The debugging adapter owns their
evidence rules; this core only requires their durable, decision-complete result.

## Plan

Read `references/model-routing-and-delegation.md` before Plan delegation,
ChatGPT Pro planning, route selection, or blind completeness. Read
`references/codebase-design.md` when entering Plan; it owns the design
vocabulary for slice, seam, and interface decomposition.

1. Re-ground on semantic artifacts and validated state. Follow the repository
   code-intelligence and live-data rules.
2. For `bug_fix`, do not checkpoint the specification or create final slice
   plans until the debugging adapter has confirmed the owning seam/root cause
   and completed the canonical regression abstraction. Missing or incident-
   shaped evidence is blocking.
3. Let the root synthesize research and decompose cohesive vertical slices
   along deep-module seams. Plan descendants, when used, are bounded,
   non-overlapping, read-only, and quiescent before Delivery.
4. Author the complete slice DAG, ownership, acceptance coverage, verification,
   UI classification, parallel assessment, and plan-derived task/route roster.
   The Markdown roster is a readable semantic mirror; operational task authority
   is the proposal's structured route and replacement-predicate fields. For a
   fresh Plan with no prior planning, delivery, or acceptance authority, submit
   that complete projection through `adopt-plan` with
   `active_response_digest: null`. This atomic bootstrap installs the slice,
   task, pair, and acceptance-owner projection so public validation can pass; it
   records only the Plan digest and grants no specification, Pro, blind, task-
   launch, or Delivery authority. Partial predecessor mutations are not a
   bootstrap path.
5. Validate, then commit and push the decision-complete specification checkpoint
   and record its exact specification digest and source commit. Invoke
   `chatgpt-pro-feature-planner` with `caller_mode: spec_workflow`, the exact
   descriptor, bundle, digest, branch, commit, inventory, and task-local
   receipt. Submit once in ChatGPT Pro, remain on the canonical conversation,
   consume fresh task-local typed wait results for the current consultation,
   attempt, generation, and task, and capture only after a current
   `generation_completed` result. The bridge publishes one immutable consumed
   continuation pass. Do not use root Browser polling or press `Answer now`,
   `Stop answering`, or an equivalent early-stop control without exact current-
   conversation user authorization.
6. Reconcile the captured response against the planning job.
   Record the response, exact context manifest, disposition, and any permitted
   annotations through `work-state record-pro-primary`. The state bridge
   independently validates continuation identity, exact original response
   bytes, digest, manifest binding, repository, branch, and source-commit
   identity. Requested headings and source-access prose are planning content,
   not an admission or retry protocol. Before adopting a recommendation, the
   root verifies every material cited path and repository claim against the
   exact checkout and source commit. It may translate or synthesize verified
   content into canonical planning artifacts while preserving the producer
   response and continuation; unsupported claims are excluded or explicitly
   annotated. In every harness, the root then runs the bounded cross-model
   plan consensus loop per `references/plan-consensus.md`: the counterparty
   provider adversarially reviews the candidate plan in rounds and the root
   arbitrates each finding with a logged reason, checking missing
   consequential decisions, contradictions between recommendations,
   unsupported assumptions, unverified repository claims, scope expansion
   beyond the accepted goal, missing migration or rollback boundaries,
   provider-specific blind spots, and validation claims that do not match
   their named layer. The root folds accepted corrections into canonical
   synthesis and never edits the immutable response; classification still
   describes the original response. This loop is not independent review; its
   value is provider diversity. If the verified
   remainder cannot support planning, the root uses
   the existing semantic-unusable recovery. The
   planner's public successor command classifies first. The root marks a
   response `--semantically-unusable` only for content that cannot support
   planning, never for presentation. `valid` returns to the same
   stage without a successor, lease, send, resend, or producer restart. Only
   `restart_or_fail_closed` preserves the predecessor as invalidated evidence,
   creates no lease, and commits attempt-2 authority through continuation
   revision `2`. A valid attempt-2 capture advances revision `3`; a second
   invalid response returns
   `BLK-PRO-INVALID-RESPONSE-EXHAUSTED`, which the root records before stopping.
   When the invalidated primary's limiting condition is inability to retrieve
   the required source evidence, the exact revision-`2` continuation is also
   sufficient for the coordinator to synthesize the locally verified Plan.
   Do not claim or send the reserved successor. Produce the mandatory fresh
   CLEAN blind-completeness receipt and use the existing typed
   `source_access_recovery` disposition; `adopt-plan` records the bounded
   coordinator exception without renewed user authorization.
   For later applicability, strict validation remains the default. If and only
   if the sole invalid field is `toSourceCommit`, produce one fresh live-remote
   context manifest outside the state lock and pass it through
   `record-pro-applicability --source-observation`. Reconcile only when state,
   active immutable evidence, checked-out HEAD, the exact remote branch head,
   approved-source bytes/diff, evidence anchors, and the monotonic assessment
   epoch prove one candidate. Preserve the original typed receipt, publish a
   derived effective receipt plus correction receipt, and repeat neither the
   Pro primary nor blind audit.
7. After planning descendants are quiescent, construct the complete typed Plan
   proposal from the exact current semantic-artifact bytes, applicable Pro
   evidence, slice DAG, pair assessments, structured task authority, and
   acceptance assignments. Set `active_response_digest` to the applicable Pro
   response and adopt it with `work-state adopt-plan
   --expect-revision <revision> --proposal <json-path>`. This is the sole
   decision-complete Plan assembly and Plan-recovery mutation: the helper
   rechecks the proposal under the bundle lock, merges only Plan-owned fields
   into retained delivery history, validates the full kernel, and either commits
   one revision or returns `UNCHANGED` with byte-identical state.
   If the user explicitly revokes authority for another Pro send after a
   material or uncertain applicability receipt requires a fresh primary, keep
   every completed Pro response immutable and synthesize the current Plan.
   Produce the mandatory fresh CLEAN blind-completeness receipt for that exact
   candidate, then pass both `--pro-primary-waiver-reason <user decision>` and
   `--pro-primary-waiver-blind-receipt <json-path>` in the same `adopt-plan`
   transaction. This path exists only for the exact current fresh-primary
   lineage. If user-directed synthesis changes approved semantic bytes after
   that terminal applicability record, the same transaction may advance the
   current specification and source to the proposal's exact final values only
   when the source commit exists, descends from the applicability target, and
   contains those approved bytes. It then atomically binds the final
   specification digest, source commit, canonical Plan digest, active completed
   response, and blind receipt digest. It is not a general Pro or completeness
   bypass.
   The same transaction is also the sole owner for a local Plan after Pro
   retrieval produced no selectable response. Pass
   `--pro-primary-waiver-disposition captureless_pro_recovery` only for an
   explicitly user-authorized canonical zero-pass continuation; this path
   retains its paired user reason. Pass
   `--pro-primary-waiver-disposition source_access_recovery` only for the
   canonical bounded invalidated-primary trajectory whose specification and
   source commit equal the proposal. That source-access path requires the fresh
   CLEAN blind receipt but no renewed user reason; `adopt-plan` records the
   canonical coordinator exception. Both dispositions atomically record
   `required_local_plan_recovery` and bind the exact continuation,
   specification, committed approved-source bytes, canonical Plan, task
   authority, and blind receipt without manufacturing Pro evidence. The bound
   authority remains recorded through Delivery because no active response can
   replace it; Plan or source drift still fails closed.
8. Record the typed blind-
   completeness requirement for the exact current Pro evidence and plan digest.
   A complete exact-digest plan records `not_required_exact_pro_plan`; an
   incomplete, ambiguous, multi-slice, or user-requested check records its
   matching `required_*` value and must produce a fresh clean blind receipt.
   Transition Plan to Delivery only when current Pro evidence, the typed blind
   decision and any required audit, task authorization, acceptance, UI design
   classification, and executable slice readiness all pass. If exact response
   capture fails, record `BLK-PRO-UNCERTAIN-SEND` and stop without resend or
   conversation substitution.

## Shared wait-mode policy

Every wait generation selects
its primary wake, heartbeat role, cadence, and transition action from the current
phase and reliable callback capability.
Probe the task-messaging or event capability and the same-task heartbeat
capability at each use site; configured prose or an assumed callback is not an
observation. An implementation or review wait is callback-backed only after a
successful current-generation task-messaging capability observation and exact
attempt/thread binding. A PR/check wait is callback-backed only after successful
registration for the exact PR/head and its registration receipt. Callback-free
active polling after failed task messaging or event registration requires a
successfully created and observed parent-owned polling heartbeat for that exact
wait generation. If heartbeat capability or creation fails, preserve the exact
attempt/thread or PR/head/registration facts, do not yield or claim
active polling or a future wake, and report `manual_resume_required` or blocked.
Claim callback registration, heartbeat creation, update, or deletion only after
the corresponding deterministic tool call succeeds.

Pro planning is stricter than the shared task/PR wait policy. Its current
task-local lifecycle receipt is the wait-generation authority. After the one
Browser send, bind its exact conversation and generation before registering a
wait. A fresh typed
`heartbeat_registered` result authorizes yielding, a typed
`generation_running` result requires a next generation, and a typed
`generation_completed` result authorizes capture. Each result binds the work,
consultation, attempt, wait generation, bound conversation generation, binding
digest, current harness task/thread, producer, and observation time; it expires
after 20 minutes and is consumed once. A typed
`manual_resume_required` result authorizes neither yield nor capture. The same
task must later consume `manual_resume_observed` and then a current manual
`generation_completed` result before one Browser inspection and capture.
Heartbeat IDs and receipt locators remain task-local.

The in-app Browser binding persists independently of its tab handles. If the
Pro tab binding is missing, stale, or closed after a wait, including when no
tabs remain, discard only that tab binding and reopen a fresh tab at the exact
receipt-bound canonical conversation URL. Verify the committed conversation
and generation, then continue the same generation. Reopening that exact
producer is not conversation substitution and must happen automatically
without manual recovery. Tab cleanup alone never selects manual resume,
`reconcile-wake`, `BLK-PRO-UNCERTAIN-SEND`, resend, or a user recovery request.

A completed Pro wait generation ends waiting, not the spec-work workflow. In
the same root trajectory awakened by that heartbeat, reload `state.json`, the
task-local lifecycle receipt, and the canonical continuation cursor before
acting. Consume the exact-current terminal result at most once, capture and
classify the response when it is not already consumed, then execute the
bridge-returned `parent_action` and persisted cursor before ending the turn:
`claim_successor_send` claims and begins the prepared attempt 2 through the
ordinary send/wait/capture path; `record_pro_primary` records current evidence
and continues Plan toward adoption, blind completeness, and Delivery. Continue
until the successor records a fresh `heartbeat_registered` result that authorizes
yielding, or until a genuine user gate, blocker, manual-resume boundary, or
workflow terminal is reached. Any required heartbeat XML is reserved for the
final heartbeat envelope of that root turn; it never makes wait completion a
reason to abandon active work. Its quiet text must reflect canonical state and
must not claim completion while the workflow remains nonterminal. Exact-current
duplicate terminal deliveries are idempotent no-ops only for evidence consumption:
after reloading canonical state, the root still executes any pending persisted
`parent_action` or continuation cursor.

| Waiting mode | Primary wake | Heartbeat role | Cadence |
| --- | --- | --- | --- |
| Implementation or review task | Reliable attention/terminal task callback | One parent-owned missed-callback recovery | Roughly 400 minutes |
| Registered PR/check wait | Observably registered callback/event | Parent-owned recovery fallback | 240 minutes |
| Other callback-free wait | No reliable callback | Active polling | Explicit phase-appropriate cadence |

The root is the sole wait-generation writer and owns at most one current
heartbeat for a generation. Parent-owned callback/recovery heartbeat payloads
bind the exact current task attempt/thread or PR/head/registration receipt.

Before any callback or recovery action, reload the system of record and require
that exact generation identity to remain current, then check the existing parent
task or phase transition for prior consumption. Persist a current implementation
callback through public `work-state reconcile-callback-chain` before considering
redispatch. Its only accepted chain is the exact typed implementation terminal
followed by that attempt's exact typed branch release for every ordinary terminal.
The current scope-expansion and satisfied-prerequisite variants instead retain the
branch owner and omit release. The operation rejects stale, incomplete, reordered, ambiguous, or
tampered chains without mutation. Exact replay of the already-persisted current
chain is byte-identical: it creates no revision, task, heartbeat, or redispatch.
On first active admission, `--observed-main-sha` applies the current owner's
base synchronization before terminal and release authentication.
Terminality alone does not prove that the parent consumed it; only the successful
durable transition does. A callback continues the parent immediately; it never
waits for the recovery heartbeat. A recovery wake only asks the root to reconcile
durable and system-of-record facts after a missed callback; it is not terminal
evidence by itself.

For a callback-backed implementation or review handoff, finish the exact current
task/thread/attempt binding, create and observe exactly one parent-owned recovery
heartbeat scheduled roughly 400 minutes later, then end the parent turn. Healthy
child execution produces no routine parent progress commentary and no
`wait_threads`, `read_thread`, sleep, or equivalent polling. An attention or terminal
callback is the ordinary wake. After the exact callback is durably consumed, delete
the recovery heartbeat in that same continuation. If the heartbeat fires first, run
one exact missed-callback/liveness reconciliation and do not convert that recovery
into a polling loop. A stale, duplicate, already-consumed, or wrong-task callback or
heartbeat is an idempotent no-op. These rules do not alter the distinct ChatGPT Pro
or callback-free active-polling policies above.

The route carries standing bounded recovery authority for the accepted
deliverable. Receipt-driven repair, retry, resumption, callback reconciliation,
branch-ownership correction, and merge-based synchronization continue without
another permission prompt. Fail closed only for scope expansion, a destructive
action, security or privacy risk, identity ambiguity, changed user intent, or a
separate deliverable.

Any callback-free wait outside Pro must name its own phase-appropriate
active-polling cadence instead of inheriting the 240-minute recovery cadence.

## Delivery

Read the task-bridge reference for the running harness before reserving,
launching, observing, steering, replacing, reviewing, or closing a task:
`references/codex-task-bridge.md` under Codex,
`references/claude-task-bridge.md` under Claude Code.

- Run every mutating slice and every exact-head review in separate user-visible
  harness-native project-worktree tasks. Root, same-directory, fork, or
  internal-agent substitution is invalid.
- Consume the checkpointed slice plan directly. Scope expansion returns to
  Plan; do not edit outside owned paths or let a task edit the coordinator
  bundle.
- When implementation verifies an actionable defect outside the slice's
  accepted scope, keep it outside branch mutation and return its failed
  invariant, typed affected surfaces, reproduction/evidence, impact, and
  discovery context to the root. The root must immediately apply
  `.github/issue-label-policy.json`: compute the exact fingerprint, inventory
  and authoritatively fetch all exact candidates, reuse or create exactly one
  owner, verify its labels/body/state, and record the publication receipt in
  the slice handoff. Continue the slice unless the defect is a real blocking
  prerequisite. Never leave the finding only in task prose or silently absorb
  it into owned paths.
- Before an implementation `creating` reservation consumes a revision or
  attempt, resolve its checkpoint as a local commit, require the exact current
  slice-plan bytes, and validate the checkpoint's `state.json` plus every
  declared plan against the current planning snapshot. Rejection leaves task
  state byte-identical and grants no external task authority.
- Before opening a PR, render `.github/pull_request_template.md` from the
  checkpointed primary specification, slice plan, assigned acceptance criteria,
  and applicable linked issue. This requirement applies equally to GitHub UI,
  CLI, and API publication. The Contract, Scope, and required-outcome text
  freeze at PR creation; later accepted changes update their semantic source
  first and appear as an explicit Contract amendment. A summary-only PR body,
  an unfilled template, or a pointer to private working notes is not a
  publishable contract.
- Verify focused gates before broader gates. Record typed implementation,
  branch, review, merge, acceptance, and quiescence evidence through the bridge.
- Give each review task one initial exact attachment and one task-local rolling
  lease. Remediation and clean base-sync pushes advance that lease without
  parent reattachment; the root consumes one terminal lease receipt.
- If an ownerless PR receives an exact clean base-sync merge after release but
  before initial review attachment, consume the typed head-adoption receipt into
  the same creating reservation before activation. Authenticate the causal
  release, deterministic clean-merge tree, current main or stacked base, and
  remote-only branch advance before changing the local ref. Do not replace the
  task or route this pre-attachment recovery through the rolling lease.
  Preserve that adoption lineage before a later clean base-sync reopen or
  typed review-task replacement discards the live terminal attachment.
- Keep one implementation task and one review task for the lifetime of each
  open-PR slice. A material review finding returns the bundle to Plan,
  increments the slice revision epoch, and reauthorizes the same implementation
  task. Use a fresh Pro plan and blind requirement for semantic, acceptance,
  architecture, dependency, topology, route, migration, policy, or
  separate-deliverable changes. A blocked exact-head review may instead use the
  closed `required_by_intent_scope_expansion` disposition when its added paths
  and obligations are inseparable from one unchanged accepted criterion and
  the deterministic plan comparison passes. The same implementation and review
  tasks then sequentially reattach the revised PR branch. Replace a task only
  for an existing typed unrecoverable-task predicate or an explicitly separate
  deliverable.
- Before a first commit or PR, a healthy quiescent implementation that returns
  the closed `SCOPE_EXPANSION_REQUIRED` terminal variant may remain the owner.
  That terminal must remain unpublished and preserve the task's exact bound
  head. When the coordinator confirms that the terminal's exact paths remain
  inside the same accepted criterion, owning seam, behavior, dependencies,
  topology, route, runtime authority, and deliverable, keep Delivery active and
  consume its observation-only Git receipt through
  `coordinator_scope_expansion`. Preserve the existing checkpoint, Pro
  evidence, task/attempt/client/worktree/branch identity, and dirty bytes;
  resume immediately without plan editing, a blind audit, user approval, or a
  second Git attachment. A changed criterion or seam rejects atomically.
  Material expansion returns to Plan. After fresh planning, require the
  observation-only Git receipt to bind the
  exact terminal, revised authority, local branch/base identity, and every
  authorized dirty byte. Consume that receipt through
  `pre_pr_scope_expansion`, preserve task/attempt/client/worktree/branch
  identity and the current typed blind requirement, invalidate prior blind
  completeness against the revised plan, and resume without a second Git
  attachment. Any stale, unrelated, conflicting, or out-of-inventory fact
  rejects before state mutation.
- An unpublished implementation blocked by a separately delivered prerequisite may
  remain the same task only when its closed `FAILED` terminal binds the prerequisite
  PR, head, required base ref, optional tracking issue, and canonical prerequisite
  fingerprint. After that PR's immutable merge receipt exists, the task publishes a
  `pre_pr_prerequisite_observation` that binds the unchanged checkpoint, plan,
  specification, acceptance, roster, owned paths, dirty manifest, terminal digest,
  prerequisite merge receipt/digest, merge commit, and live required-base ancestry.
  Consume it with `satisfied_prerequisite`; archive a dedicated revision, advance the
  required base, and reauthorize the same task/attempt/thread/client/worktree/branch.
  Keep this unpublished failed task physically attached to its branch while quiescent;
  do not publish branch-release proof before the prerequisite observation.
  Permit only the required-base merge and rerun of existing verification. Do not use
  issue state as authority, add scope, replace the task, attach Git again, publish a
  PR, or reinterpret a generic `FAILED` terminal as resumable.
- Keep base synchronization with the task that currently owns the PR branch:
  implementation before release, review during startup or under its rolling
  lease afterward. Record the new observed base with that owner's typed
  evidence. Stale base alone never replaces a task. Resolve a conflicting
  synchronization or delivery merge per
  `references/resolving-merge-conflicts.md`, within the owning task's
  authority.
- Use serial delivery unless the complete-wave independence gate proves no
  dependency, owned-path, semantic-resource, or coupled-verification boundary.
- The root alone records lifecycle changes, authorizes merges under the
  approved bundle policy, and declares completion.
- After a pull request reaches terminal CLEAN, the root performs exactly one
  freshness check: compare authoritative remote main with the base bound by the
  clean receipt. If unchanged, merge immediately with a merge commit. If
  authoritative remote main advanced, use the existing merge-based base
  synchronization and current-head revalidation, then merge automatically with
  a merge commit when terminal CLEAN again. Do not ask the user to choose a
  merge method. Do not request separate merge consent. Stop only for a named
  critical blocker or an external condition that prevents merge. Do not repeat
  head, CI, mergeability, thread, provider, terminal-verification, attachment,
  or lease-finalization gates merely because merge execution follows a
  published clean result.
- Persist both unchanged-main and advanced-main branches as terminal-clean
  delivery continuations in existing queue state before invoking the merge
  provider or yielding across review or callback boundaries. On unchanged
  main, `work-state terminal-clean-delivery --observed-main-sha` authenticates
  and archives the current clean terminal, then returns the deterministic
  pending-merge precondition. Resume that continuation without the observation
  argument; the root must not run the comparison again. On advanced main, the
  restored review terminal records the consumed SHA in
  `state.observed_main_sha` and the slice base, preserves the original clean
  terminal in `review_task.head_adoption_terminal_lineage`, and binds the
  rolling review-lease rotation whose `kind` is `base_sync`, the synchronized
  head, `review_task.head_adoption_receipt`, and the restored clean receipt.
  Require that contiguous lineage before merge. It is the shared route's
  persisted terminal-clean delivery continuation, not a new primitive or
  delivery attempt, and the root must not run the freshness comparison again.
  Store the current comparison in the slice's dedicated terminal-freshness
  binding; historical CLEAN lineage created by sibling base synchronization is
  authenticated history, not evidence that this slice consumed its comparison.
- If the atomic merge result is `HEAD_DRIFT`, ingest that typed receipt through
  `work-state terminal-clean-delivery --merge-result`. It must match the
  persisted PR and policy precondition. If its observed base is unchanged, the
  same review task revalidates the observed current head over the original
  consumed base. If its observed base also advanced, consume both observations,
  invalidate every affected open slice, and run merge-based synchronization plus
  current-head revalidation in the same review lineage. The no-observation resume
  returns the resulting head/base-bound precondition. Never perform a second
  remote-main comparison.
- Bind merge execution atomically to the reviewed head and the base SHA consumed
  by that continuation. If the deterministic merge bridge reports
  `base_drift_after_consumed_freshness`, do not merge; consume its observed base
  SHA, invalidating every affected open slice, into the same base-sync/current-
  head-revalidation continuation without a separate remote-main read. Final
  provider receipts authenticate the exact provider executable digest used by
  the operation.
  Persist every accepted typed drift receipt by path, digest, and observation
  epoch in slice state and authenticate those retained bytes on every reload.
  Unknown-outcome recovery claims one exclusive reservation owner before
  reconciliation and holds that ownership through final receipt publication;
  concurrent recovery may observe the winner but may not invoke the provider or
  replace its result.

Read `references/ui-gate-ownership.md` when classifying or delivering UI work.
Every UI-affecting spec-work bundle uses `final_pr_design_gate`, exactly one
final UI slice, current automated proof from a repository-native UI route
discovered at use time, and the final human checkpoint after review, CI,
verification, mergeability, and base synchronization are otherwise clean. Standalone work
outside a bundle retains immediate human confirmation.
If that final checkpoint reports an issue after exact-head CLEAN review, consume its
canonical BLOCKED receipt through `reopen-final-ui-remediation`; this atomically
retains PR/task lineage, invalidates final-slice proof, and reopens implementation
and review for the next revision without returning to planning.

Every implementation PR carries the exact coordinator-approved canonical bundle
snapshot for that PR. After the implementation child has published its ordinary code
head and open-PR identity, it sends an attention callback without terminalizing. The
root consumes those exact facts, freezes the deterministic snapshot in an isolated
coordinator projection, records that exact tuple in the live bundle with
`work-state authorize-pr-snapshot`, and resumes the same bound child with only the
commit, lifecycle path, slice, classification, and manifest digest. The child materializes
only those bytes in a commit that retains the approved snapshot commit as an ancestor,
verifies the resulting head, and then publishes its terminal receipt and callback; it
never authors lifecycle state. The ancestry requirement keeps every persisted snapshot
authorization reachable through the PR history for later validation and reopen.

For a terminal highest parallel wave, the lexicographically last slice is the
deterministic final snapshot carrier. It remains active before `pr_open` while its
same-wave peers publish intermediate snapshots and merge, then consumes the already
observed current main through its existing owner path before publishing its ordinary
head/PR attention handoff. The normal freeze, materialization, and terminal handoff
then proceed on that synchronized carrier. Its topology-derived carrier identity does
not disappear if an old early-terminalization path is attempted; such terminalization
is rejected until the active `pr_open` snapshot handoff.

Intermediate snapshots remain under `in_progress/` with `phase != complete`. The
designated final implementation PR carries a `phase=complete` projection under
`completed/`, with every pre-review work and acceptance obligation complete and locked
while its own exact slice remains `pr_open` to record that merge is still pending. For
a `final_pr_design_gate` projection, the sole final human design acceptance instead
remains `pending` and unlocked and `completion_binding` remains null; the post-review
live lifecycle records that typed human result before merge and live completion. Use
`transition --phase complete --premerge-final-slice <PR-*>` only in that isolated
projection. The mutable coordinator bundle remains the post-handoff orchestration
source for later receipt, review, and quiescence writes; those live writes are not an
input to snapshot verification. In that frozen projection, the designated final
implementation task remains the sole active task and its review task remains unbound:
the same child must still materialize the approved bytes and publish its terminal
receipt before review can start.

Before review and again before merge, run `work-state verify-pr-snapshot` against the
authenticated frozen commit, exact slice, manifest digest, and exact candidate head.
The verifier compares the immutable commit tree to the candidate head tree, binds the
truthful lifecycle classification, and rejects missing, stale, alternate-path, or
inconsistent bytes. It also requires the frozen commit to be an ancestor of the
candidate head. Review attachment and lease startup carry the same authenticated
tuple separately from ordinary `ownedPaths`; only that exact materialization is
admitted outside ordinary ownership, while review remediation stays ordinary-owned.
Every ordinary feature or full bug-fix review activation requires this non-null
authenticated snapshot. A null snapshot authorization is accepted only for an
explicit compact fast-fix record, and authenticated lifecycle paths never participate
in ordinary changed-path ownership or compact fast-fix classification.

## Complete

For the live post-review/post-merge coordinator state, re-observe system-of-record
facts and ask the deterministic bridge to transition only when:

- every slice is merged or superseded and every task lineage is quiescent;
- every acceptance criterion is passed or explicitly waived;
- review, CI, verification, base, head, and merge evidence are current;
- final UI design acceptance is confirmed or explicitly waived when applicable;
- no blocker remains; and
- for `bug_fix`, at least one planned fix PR is observed merged.

Do not launch a separate completion auditor or synthesize another receipt from
these facts. The bridge deterministically consumes the canonical slice, review,
merge, acceptance, UI, and task-lineage evidence and fails closed when any of it
is missing, stale, contradictory, or non-quiescent.

This later live completion records merge as an external fact and does not redefine or
invalidate the already published pre-merge final snapshot. After the bridge accepts
`complete`, move the root-owned bundle from
`<bundleParent>/in_progress/` to `<bundleParent>/completed/` with a normal Git
change in the coordinator checkout. Run the bridge's `verify-publication` operation
against the fetched default-branch commit and the immutable completed snapshot carried
by the designated final implementation PR. Do not require equality with post-handoff
live coordinator mutations. Do not report terminal completion, stop the completion
wait, or archive the task until that operation proves the frozen snapshot tree is
reachable from the default branch. A missing implementation-PR snapshot is a
review/merge blocker, not authority for a recovery documentation-only PR. Never infer completion from prose, an
open PR, a build, a task receipt, stale evidence, or a local-only docs commit.

For a user-supplied or route-selected owning issue, successful default-branch
publication also closes the issue lifecycle, branched by `workKind`. For
`feature`, the owning issue is the `enhancement` issue named in
`FEATURE.md`'s `Source / Issue`: fetch the issue, append a completion
comment linking the merged PRs only when no existing comment already
carries that exact merged-PR evidence, then close it if still open — the
step is idempotent under retry or resume. Its closure involves no defect
fingerprint, severity, or marker reconciliation. For `bug_fix`, apply
`.github/issue-label-policy.json`: authoritatively refetch the owner and exact
fingerprint inventory, preserve existing prose, append only missing verified
completion evidence, add the canonical marker when absent, reconcile exact
labels, and close the owner without requesting separate user authorization. If
a PR closing keyword already closed it, verify and maintain that same closed
owner rather than skipping the body update or creating another issue. A fix not
durably effective on authoritative main remains open.

## Canonical format and cost boundary

Bundles emit only the stable unversioned `spec-workflow-state` kind and generic work
identity. Workflow updates replace this canonical contract in place; never introduce
numeric schema versions, migrations, defaults, or parallel compatibility paths.
Every state and receipt must already match the current exact contract; unsupported
schema identities, missing fields, malformed state, and retired receipt formats fail
clearly and are never rewritten or reinterpreted. Feature-named script paths are thin
aliases into the canonical tools and own no separate behavior.

Diagnosis-only and `fix_fast` debugging must not load this file, create a
bundle, call Pro, create a heartbeat/task, or gain publication authority. A
bundled run requires one accepted exact digest-bound ChatGPT Pro planning pass,
with at most one preserved invalid predecessor, immutable response capture,
current parent evidence, and the existing task/review route governor.
Response-capture failure never authorizes resend, conversation
substitution, or invented evidence.
