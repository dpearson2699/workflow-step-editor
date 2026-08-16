# Codex Task Bridge

Load this reference only when reserving, launching, observing, replacing, or closing
a spec-work slice implementation or review task. It is the Codex desktop/app adapter,
not a portable subagent contract.

This bridge creates and executes generic `spec-workflow-state` bundles only.
Unsupported state fails before render or mutation with archive-or-restart guidance;
never migrate or reinterpret it in place.

The checkout-ownership authority is the bundled executable
`.agents/workflows/spec-work-orchestrator/scripts/task-git-binding`. Only that
helper may attach or detach an orchestrated task worktree. It never fetches, creates or
moves refs, resets, cleans, commits, or pushes.

```sh
WORK_GIT_BINDING=.agents/workflows/spec-work-orchestrator/scripts/task-git-binding
REVIEW_LEASE=.agents/workflows/spec-work-orchestrator/scripts/review-lease
```

## Non-substitution and authority

Every mutating slice, including one trivial slice, runs in one user-visible Codex task
created with `codex_app.create_thread` and a new project `worktree` environment. Every
published PR is reviewed in a different user-visible project-worktree task. Do not
substitute root mutation, a same-directory task, `fork_thread`, or an internal
collaboration subagent. If the project-worktree surface or requested route is rejected,
record a blocker and stop.

Invoking this workflow is standing authority for the approved plan's registered
implementation, review, and predicate-bounded replacement tasks. Derive the canonical
roster in the primary specification, run `record-task-roster`, and never ask for per-task, writing,
replacement, or model-route approval. Authority never extends beyond the slice plan.

One top-level task lineage owns each slice worktree. It may delegate slice-bounded
research, tests, mutation, integration, verification, and publication. Do not impose a
workflow-authored agent-count, concurrency, or nesting limit or edit
`.codex/config.toml`; actual runtime limits remain external. Delegate concurrent writes
only to disjoint paths, serialize overlap, and quiesce the full lineage before handoff.

## Launch facts

- `LAUNCH` is the durable `<work-id>/<slice>/<role>/<attempt>` recovery marker.
- `START` is the sole checkout authority: an existing ref plus its exact observed SHA.
- `REQUIRED_BASE` is included only when the task must integrate a newer main or
  upstream head before planned edits. When omitted, `START` is also the required base.
- A fresh slice starts from its recorded required base. A resumed open-PR revision or
  typed replacement starts from the exact current PR head and names current main or
  approved upstream as `REQUIRED_BASE`; `replacement_lineage` preserves that PR/head
  only for revision reauthorization or a typed replacement. An ordinary base advance
  preserves the PR/head and reopens the same review task without replacement lineage.
- The committed coordinator bundle supplies the immutable `CHECKPOINT`, `PLAN`, and
  final plan digest. Never launch from uncommitted semantic artifacts.
- The root records launch markers, roster digests, queued creation IDs, task IDs,
  worktree paths, and observed SHAs. Do not copy those root-owned facts into child
  prompts unless a prompt field below explicitly requires them.

Select the saved Codex project whose Git common directory matches the coordinator
repository. `create_thread` uses this target shape, with the existing `START` ref as
`startingState`, never the desired new branch:

```text
target: {type: project, projectId: <id>, environment:
  {type: worktree, startingState: {type: branch, branchName: <START-ref>}}}
```

Request every implementation task in `create_thread` with
`model: "gpt-5.6-sol"` and the roster-selected
`thinking: "medium" | "high" | "xhigh"`. Request every exact-head review task with
`model: "gpt-5.6-sol"` and `thinking: "high"`. `codex_task_request` proves request
transport, not the effective route.

## Reservation and uncertain creation

Before `create_thread`, reserve the exact attempt through revision CAS:

```sh
"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state creating \
  --launch-marker <work-id/PR-01/implementation/attempt> \
  --authorization-digest <digest> --plan-checkpoint-sha <sha> \
  --adoption-start-sha <attempt-1-start-sha>
```

Attempt 2+ omits `--adoption-start-sha` and instead supplies one registered
replacement predicate and its current typed observation. Serial reservations are
legal only in Delivery when the slice is actually executable. Initial parallel
reservations may be made for a complete eligible wave so its tasks can prepare before
atomic activation.

The creating transition is the launch admission check. For `RUN_SLICE` and
`RUN_REVIEW`, evaluate every predicate independent of the returned task identity,
worktree, or bound SHA before `create_thread`; rejection leaves state and roster
unchanged. An accepted implementation reservation is that task's activation token.
Its later identity binding is not a second admission gate.
Review startup is supervised by the exact task-local Git-binding command and receipt
described below. If a blocker exists before
`create_thread`, do not create. If creation may have occurred, reconcile and record the
exact task identity, then stop and account for the task as required instead of erasing
the attempt. Implementation `PREPARE_ONLY` remains inert, so it may coexist with
blockers and `start-wave` still rejects until blockers clear.

A returned `threadId` or queued `clientThreadId` proves only creation. A queued ID may
be recorded on the existing `creating` attempt but is not steerable. Never retry an
uncertain create automatically. Reconcile by exact `LAUNCH`, repository, slice, and
role. Bind exactly one verified match; zero, multiple, or unverifiable matches remain
visible in `creating` for manual recovery. Never infer identity from list order, title,
or an available worktree.

Before binding an implementation `active` or `prepared`, the root re-observes that the
task worktree is distinct, clean, in the correct Git common directory, and at `START`.
Before binding a review `active`, the Git-binding helper additionally proves unique PR
branch ownership. Wrong repository, branch, start SHA, cleanliness, duplicated
task/worktree, stale required base, or a detached review fails closed. `work-state`
records supplied observations but never queries Codex or Git.

## Direct serial implementation

A serial or isolated same-wave retry launches active in its initial task prompt. There
is no preparation callback or activation message.

```text
RUN_SLICE
LAUNCH: <work-id>/<PR-*>/implementation/<attempt>
PARENT_TASK_ID: <delegation source_thread_id>
CHECKPOINT: <40-hex SHA>
PLAN: <bundle-relative pr/PR-*/PLAN.md>
PLAN_SHA256: <digest>
BRANCH: <desired branch>
START: <existing-ref>@<SHA>
REQUIRED_BASE: <ref>@<SHA>  # omit when equal to START
AUTHORITY: only the checkpointed slice plan and declared owned paths
IMPLEMENTOR_CONTRACT: .codex/agents/spec-feature-implementor.toml

Use the dedicated worktree. Before any work, read `developer_instructions` from
IMPLEMENTOR_CONTRACT and treat them as the task responsibility contract. Stop if the
contract cannot be loaded. Load the contract and exact plan with git show
<CHECKPOINT>:<path>; verify the plan digest, repository, branch, START, and a clean
checkout before mutation. Execute the plan directly; do not create another plan.
Never author work-bundle lifecycle state, merge, waive acceptance, or cross an
undeclared/shared boundary. Publish the ordinary code/docs head and open-PR identity,
then send one compact attention callback to PARENT_TASK_ID and stop mutation without
publishing the terminal receipt. The parent resumes this same bound attempt with:

```text
BUNDLE_SNAPSHOT: <slice>:<classification>:<40-hex commit>:<bundle-relative lifecycle path>@<manifest digest>
```

Before that resume, the parent records the exact tuple in the live coordinator
bundle with `work-state authorize-pr-snapshot`. The later review attachment must
match that durable record exactly.

Materialize only those exact bytes and lifecycle-path deletion in a commit that keeps
the coordinator-approved snapshot commit as an ancestor, verify them against the
resulting candidate head with the supplied `work-state verify-pr-snapshot` command,
and do not synthesize, summarize, or refresh them. On scope expansion, stop
before commit or push. Only after the verified snapshot head and every terminal
receipt field are ready, send exactly one terminal direct callback to PARENT_TASK_ID
with codex_app.send_message_to_thread immediately before returning the JSON slice
receipt. Later live coordinator writes are not snapshot input.
```

For a resumed open-PR revision or typed replacement, `START` must equal the retained
lineage head even when the slice's `base_sha` has advanced to `REQUIRED_BASE`. Preserve
existing commits, integrate that base before other planned edits, update the same PR,
and never force-push. A conflict outside owned paths or a new shared API,
schema/migration, canonical document, fixture/generated file, backend resource, or UI
dependency returns `scope_expansion_required` without commit or push. The root accepts
a rotated output head only for the retained PR, then clears lineage on the new
`pr_open` observation.

When a new full-fix bundle adopts an already-open fast-fix PR as implementation
attempt 1, `START` is the existing PR head and `REQUIRED_BASE` is its accepted base.
Before mutation, the same implementation task runs the supplied ownerless-branch
adoption command:

```sh
"$WORK_GIT_BINDING" adopt-implementation \
  --work-kind bug_fix --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 --role implementation --attempt 1 \
  --revision-epoch 1 --launch-marker <same-marker> \
  --thread-id <same-task-id> --worktree <absolute-task-worktree> \
  --git-common-dir <absolute-git-common-directory> \
  --branch <existing-pr-branch> --bound-sha <exact-pr-head> \
  --observed-remote-sha <exact-pr-head> --base-ref <required-base-ref> \
  --base-sha <required-base-sha> --pr <existing-pr-number> \
  --adoption-start-sha <durable-attempt-1-adoption-start> \
  --observation-epoch <epoch> --bundle-root <canonical-bundle-root> \
  --receipt <absolute-bundle-receipt>
```

The operation has exactly two attempt-1 modes. Ordinary adoption requires a clean
detached ownerless branch checkout at the exact live remote PR head,
required-base ancestry, and `bound-sha == adoption-start-sha`. The recorded
`adoption_start_sha` is the durable adoption start. Required-base-sync
adoption requires the branch already attached to the exact task worktree and accepts
only a two-parent merge whose first parent is the durable `adoption_start_sha`, whose
second parent is the exact live required-base SHA, and whose tree equals Git's clean
deterministic merge of those parents. Both modes require a successful live
`gh pr view` observation proving that the numbered PR is open with that exact head
branch, head SHA, and base branch. They publish typed
PR/head/base/adoption-start/branch/task ownership proof. The operation rejects any
other attempt, owner, parent order, merge tree, base, head, PR, or external task
identity before receipt publication or checkout mutation. An already-attached branch
is accepted only as an idempotent replay of the exact published receipt.
A retryable attachment correction stays in the same creating task and attempt. The
root then consumes that exact receipt when recording activation:

```sh
"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state active --thread-id <id> \
  --task-worktree <absolute-path> --bound-sha <exact-pr-head> \
  --branch-attachment-receipt <bundle-relative-adoption-receipt>
```

Without this proof, attempt 1 still must start from its required base. Adoption proof
with a mismatched task, attempt, branch, PR, head, or base fails without state mutation.
The implementation terminal must preserve the adopted PR number. Its immutable
adoption-time base remains receipt lineage when the current owner later integrates and
records a newer `origin/main`. If the adopted attempt is superseded before terminal
publication, the state machine retains the receipt's authenticated PR number and exact
bound head as replacement lineage before clearing the task-local attachment fields;
the next authorized attempt must start from that retained head. The same retention
occurs before non-success terminal quiescence clears the attachment, preserving later
supersession recovery. Once authenticated terminal evidence exists, its exact output
head supersedes the adoption-time start head for retained replacement lineage.

Immediately before implementation terminal handoff, the same implementation task
fetches and compares the current PR base. If `origin/main` advanced, it performs the
permitted merge-based integration, reruns affected proof, and returns the new base SHA
with its terminal/release evidence. The root records that SHA through the same
`task-transition ... --state quiescent --observed-main-sha` mutation. A base advance
never creates implementation attempt 2.

After creation resolves and the root verifies the checkout, record direct activation:

```sh
"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state active --thread-id <id> \
  --task-worktree <absolute-path> --bound-sha <START-sha>
```

If a task started despite a failed root observation, stop it and keep the attempt
accounted until quiescent; never rewrite state as though it never ran.

## Pre-PR scope-expansion continuation

Before its first commit or PR, an implementation may return
`SCOPE_EXPANSION_REQUIRED` only through the closed terminal variant. In addition
to the unchanged terminal fields, it carries sorted nonempty
`requiredOwnedPaths`, one declared `acceptanceId`, the unchanged `owningSeam`,
and nonempty `regressionObligations` and `verificationObligations`. Other
results reject these additional fields.
The scope-expansion variant cannot claim a PR and its output head must still
equal the task's bound head.

The root records that terminal and quiesces the same implementation task. It
first decides whether the exact required paths remain inside the same accepted
criterion, owning seam, behavior, dependencies, topology, route, runtime
authority, and deliverable. That non-material case stays in Delivery, preserves
the current plan/checkpoint and existing Pro evidence, and asks the same task to
publish one observation. A material expansion returns Delivery to Plan, obtains
current Pro and typed blind authority, amends only the cited scope, acceptance,
and verification text, records the revised roster and checkpoint, then asks
that same task to publish the same observation shape:

```sh
"$WORK_GIT_BINDING" observe-pre-pr-implementation \
  --work-kind <feature|bug_fix> --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 --role implementation --attempt <same-attempt> \
  --revision-epoch <current-revision> --launch-marker <same-marker> \
  --thread-id <same-task-id> --client-thread-id <same-client-task-id> \
  --worktree <same-worktree> --git-common-dir <same-common-dir> \
  --branch <same-branch> --bound-sha <persisted-bound-sha> \
  --base-sha <persisted-base-sha> --state-revision <N> \
  --terminal-receipt <bundle-relative-terminal> \
  --terminal-receipt-digest <digest> \
  --plan-checkpoint-sha <new-checkpoint> --plan-digest <new-plan-digest> \
  --authorization-digest <current-roster-digest> \
  --specification-digest <current-specification-digest> \
  --acceptance-digest <current-acceptance-digest> \
  --observation-epoch <next-epoch> \
  --owned-path <complete-revised-owned-path> [...] \
  --bundle-root <canonical-absolute-bundle-root> \
  --receipt <absolute-bundle-observation-receipt>
```

The operation never switches, cleans, stashes, resets, commits, or copies the
checkout and makes no remote-head claim. Under the shared branch lock it proves
local `HEAD` and the local branch equal the persisted bound SHA, the persisted
base is its ancestor, and the task worktree is the unique branch owner. Its
NUL-safe porcelain-v2 manifest keeps separate index/worktree status and current
index mode/content digest plus current regular-file or symlink worktree content
digest, or explicit absence, for ordinary changes, type changes, deletions,
rename/copy pairs, and untracked paths. Conflicts, dirty submodules,
ignored-only paths within the complete revised inventory, unsupported objects,
malformed or duplicate records, noncanonical ordering, and any dirty path
outside that inventory reject without publishing. Unrelated ignored repository
artifacts are not part of the observation.

The root consumes that immutable receipt with the same revision CAS:

```sh
"$WORK_STATE" reauthorize-implementation \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --disposition coordinator_scope_expansion \
  --add-owned-path <exact-terminal-required-path> \
  --plan-checkpoint-sha <unchanged-current-checkpoint-sha> \
  --authorization-digest <current-roster-digest> \
  --pre-pr-observation-receipt <bundle-relative-observation-receipt>
```

This current-shape fast path validates the terminal path set, accepted criterion,
accepted owning seam, task identity, checkpoint, authorization, and complete
dirty manifest before one CAS archives the evidence and authorizes the same task.
It does not edit the plan, enter Plan, send Pro, run a blind audit, create a task,
increment the task attempt, or attach Git again. A changed criterion, seam,
deliverable, or other material boundary must not use this disposition.

After a material expansion completes fresh planning, consume the receipt with:

```sh
"$WORK_STATE" reauthorize-implementation \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --disposition pre_pr_scope_expansion \
  --add-owned-path <exact-terminal-required-path> \
  --plan-checkpoint-sha <new-checkpoint-sha> \
  --authorization-digest <current-roster-digest> \
  --pre-pr-observation-receipt <bundle-relative-observation-receipt>
```

The precursor digest binds current state revision, exact terminal path/digest,
complete task identity, checkpoint/plan, roster, revised paths, specification,
acceptance, observation epoch, and dirty-manifest digest. Successful CAS
archives the terminal and observation path/digests, adds the observation epoch
to the evidence ledger, increments only revision authority, and additionally
binds the observation path/digest into the final reauthorization digest. The
bundle-relative observation path must end in `.json`. CAS preserves the typed
blind requirement against the revised plan and clears prior blind completeness,
so a required decision must receive a fresh complete audit before Delivery. An
orphan observation has no authority and becomes stale after any relevant CAS.

For `coordinator_scope_expansion`, send the literal reauthorization receipt to
the same task and activate it immediately in the still-current Delivery phase.
For `pre_pr_scope_expansion`, after Plan re-enters Delivery send
`RESUME_IMPLEMENTATION_AFTER_PRE_PR_SCOPE_EXPANSION` with the literal receipt
and revised checkpoint/plan. Activation derives its mode from the latest
archived disposition and consumes no attachment command.
The same task, attempt, client task, worktree, local branch, and preserved dirty
bytes continue. A replacement task, second Git attachment, PR lineage, copied
worktree, or caller-supplied dirty-path exception is invalid.

## Pre-PR satisfied-prerequisite continuation

An unpublished implementation that cannot finish until a separate pull request
merges may return the closed prerequisite `FAILED` terminal variant. In addition to
the ordinary failure fields, it carries `prerequisiteFingerprint`, optional
`prerequisiteIssue`, positive `prerequisitePr`, exact `prerequisiteHeadSha`, and
canonical `requiredBaseRef`. The fingerprint is the canonical digest of
`trackingIssue`, `pullRequest`, `headSha`, and `requiredBaseRef`. The terminal cannot
claim an implementation PR or a changed output head. A generic `FAILED` terminal is
valid failure evidence but is not resumable through this route.

After the prerequisite is delivered, keep the same implementation task quiescent and
physically attached to its branch; this unpublished prerequisite terminal is the sole
exception to the ordinary implementation branch-release handoff below. Record the
terminal without `--branch-release-receipt`, do not run `task-git-binding release`,
and run the existing observer with the immutable merge receipt:

```sh
"$WORK_GIT_BINDING" observe-pre-pr-implementation \
  --work-kind <feature|bug_fix> --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 --role implementation --attempt <same-attempt> \
  --revision-epoch <current-revision> --launch-marker <same-marker> \
  --thread-id <same-task-id> --client-thread-id <same-client-task-id> \
  --worktree <same-worktree> --git-common-dir <same-common-dir> \
  --branch <same-branch> --bound-sha <persisted-bound-sha> \
  --base-sha <persisted-prior-base-sha> --state-revision <N> \
  --terminal-receipt <bundle-relative-terminal> \
  --terminal-receipt-digest <digest> \
  --prerequisite-delivery-receipt <bundle-relative-merge-receipt> \
  --plan-checkpoint-sha <unchanged-checkpoint> --plan-digest <plan-digest> \
  --authorization-digest <current-roster-digest> \
  --specification-digest <current-specification-digest> \
  --acceptance-digest <current-acceptance-digest> \
  --observation-epoch <next-epoch> --owned-path <current-owned-path> [...] \
  --bundle-root <canonical-absolute-bundle-root> \
  --receipt <absolute-bundle-observation-receipt>
```

This variant verifies the terminal fingerprint, the existing atomic merge result and
its transport digests, the merge parents `[prior-base, prerequisite-head]`, the live
remote required-base head, and local ancestry from both the prior base and merge
commit to that live head. It also performs the ordinary exact task/branch/dirty-byte
observation. It never consults issue state, changes the checkout, or grants authority
from a mutable PR lookup.

Consume the resulting `pre_pr_prerequisite_observation` with:

```sh
"$WORK_STATE" reauthorize-implementation \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --disposition satisfied_prerequisite \
  --plan-checkpoint-sha <unchanged-checkpoint> \
  --authorization-digest <current-roster-digest> \
  --pre-pr-observation-receipt <bundle-relative-observation-receipt>
```

The CAS requires an unpublished blocked slice, the exact quiescent failure owner, no
review owner, unchanged plan/specification/acceptance/roster/path authority, peer
quiescence, an advanced required base, and a fresh observation epoch. It archives a
dedicated `satisfied_prerequisite` revision and returns
`nextAction=merge_required_base_and_rerun_verification`. Send that literal receipt to
the same task and activate it without another Git attachment. Only merging the
required base and rerunning already-authorized verification are permitted; any new
scope or other obligation requires the ordinary material route.

## Same-task implementation revision

A material review finding that changes the accepted plan does not replace a healthy
implementation task. After the review task is blocked, quiescent, and has released
the branch, the root returns Delivery to Plan and reauthorizes the existing
implementation attempt. The ordinary `fresh_pro_plan` disposition requires a fresh
exact-digest Pro plan and typed blind requirement:

```sh
"$WORK_STATE" reauthorize-implementation \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --disposition fresh_pro_plan \
  --plan-checkpoint-sha <new-checkpoint-sha> \
  --authorization-digest <current-roster-digest>
```

The checkpoint must resolve locally and its bundle-relative slice-plan blob must match
the current working `PLAN.md` byte for byte. Every implementation checkpoint also
contains the bundle's exact `state.json` and every declared slice-plan blob so a later
review reauthorization can recompute the complete prior topology from that immutable
commit; rejection occurs before state mutation.
The current typed blind decision is likewise bound to the canonical plan digest.
Reauthorization also requires branch-release proof newer than the terminal review
attachment. It archives that blocked terminal receipt and digest in revision history
before live review fields are reused.

The returned `spec-work-task-reauthorization` receipt preserves task, worktree, branch,
and PR identity while incrementing `revisionEpoch`. After Plan enters Delivery, send
one message to that same implementation task:

```text
RESUME_IMPLEMENTATION_AFTER_REPLAN
REAUTHORIZATION: <literal spec-work-task-reauthorization JSON>
CHECKPOINT: <new 40-hex SHA>
PLAN: <bundle-relative pr/PR-*/PLAN.md>
PLAN_SHA256: <new digest>
ATTACHMENT_COMMAND: <literal complete task-git-binding attach-implementation command>
IMPLEMENTOR_CONTRACT: .codex/agents/spec-feature-implementor.toml

Before any work, reload `developer_instructions` from IMPLEMENTOR_CONTRACT and treat
them as the task responsibility contract. Stop if the contract cannot be loaded.
Reload the contract and exact plan from CHECKPOINT. Execute ATTACHMENT_COMMAND
unchanged before mutation. Correct a retryable startup error in this same task; do not
request or create a replacement task. Preserve the existing PR and commits, implement
only the revised authorized plan, push without force, release the branch, publish
revision-bound terminal evidence, and send one terminal callback to the original
PARENT_TASK_ID.
```

The attachment command binds the retained branch and current remote PR head under the
shared Git lock and publishes a `revisionEpoch`-bound receipt. The root records
`authorized -> active` only for that exact receipt and the same attempt/thread/worktree.
Replacement remains legal only for an existing typed unrecoverable-task predicate or
an explicitly separate deliverable.

### Review-confirmed required-by-intent scope expansion

The review owner may classify a blocked exact-head finding, but may not expand its
own lease. It finalizes the lease with
`--blocked-disposition required_by_intent_scope_expansion`, the exact
`--required-owned-path` values, one existing `--acceptance-id`, the unchanged owning
seam, and nonempty regression and verification obligations, then releases the branch.

The root may avoid another Pro planning send only when the accepted specification,
criterion inventory, slice topology, dependencies, route, and every plan section
outside Scope and Ownership, the cited Acceptance Coverage entry, and Verification
remain unchanged. The revised plan must retain every prior owned path, add exactly the
reviewer-declared paths, append only to the cited criterion, and append verification
bullets containing the reviewer obligations:

```sh
"$WORK_STATE" reauthorize-implementation \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --disposition required_by_intent_scope_expansion \
  --add-owned-path <exact-reviewer-declared-path> \
  --plan-checkpoint-sha <new-checkpoint-sha> \
  --authorization-digest <current-roster-digest>
```

The returned receipt records both plan digests, unchanged specification and
acceptance digests, the prior and revised owned-path inventories, blocked terminal
digest, cited criterion, revision epoch, and authorization digest. Any mismatch
rejects before state mutation and returns to the `fresh_pro_plan` path.

The coordinator-generated next-revision review attachment is the only adoption
authority. It carries the exact current owned-path inventory and canonical revision
authority digest already validated by `work-state`. The same review task resumes
without a caller-supplied path or reauthorization receipt:

```sh
"$REVIEW_LEASE" resume \
  --lease <task-local-lease> \
  --attachment <next-revision-review-attachment> \
  --base-ref <current-state-base-ref> \
  --release-receipt <absolute-causal-release-receipt>
```

The lease derives its frozen inventory directly from that attachment, authenticates
the causal branch release, binds the revision authority digest into its identity,
proves the revised Git delta is inside the inventory, and invalidates all prior
head-bound evidence through the existing revision lifecycle. Reordered, additional,
or unrelated caller-declared paths have no separate input surface. A product,
acceptance, architecture, dependency, topology, route, migration, external-contract,
authentication/security-policy, or separate-deliverable change is never eligible for
this disposition.

## Atomic parallel implementation

Use `PREPARE_ONLY -> ACTIVATE` only when at least two independent implementation tasks
must enter one atomic wave, including a complete same-wave retry cohort. Reserve every
cohort member first, then create each with this compact prompt:

```text
PREPARE_ONLY
LAUNCH: <work-id>/<PR-*>/implementation/<attempt>
PARENT_TASK_ID: <delegation source_thread_id>
CHECKPOINT: <SHA>
PLAN: <bundle-relative plan>
PLAN_SHA256: <digest>
BRANCH: <branch>
START: <existing-ref>@<expected START SHA>
REQUIRED_BASE: <ref>@<SHA>  # omit when equal to START
AUTHORITY: only the checkpointed slice plan and declared owned paths

Verify the contract, plan digest, repository, branch, START, optional REQUIRED_BASE,
and clean checkout. Make no tracked-file edit, commit, push, or PR. Wait for ACTIVATE.
```

The root verifies every worktree and records each task `prepared` with `thread-id`,
optional already-recorded client ID, `task-worktree`, and `bound-sha`. No preparation
receipt or wake proof exists. If any preparation fails, activate none.

For a retry cohort with open PRs, each task's START is its own retained PR head while
every cohort member shares the same refreshed REQUIRED_BASE. Do not substitute that
newer base SHA for START.

Call `start-wave` once with the complete prepared selection; it requires at least two
members and changes all to `active` atomically. Then send each task one `ACTIVATE`
message without changing route or effort. If delivery partially fails, stop notified
tasks, persist a blocker, and wait for observed quiescence. Do not fake rollback or
create duplicates. A retry is direct only when it is the sole mutating member and all
same-wave peers are terminal or quiescent; never peel one member from a retry cohort.

## Supervised exact-head review

Wait for the implementation task and every descendant to become quiescent, then
observe the exact local and remote PR head. Before recording implementation
`quiescent`, release its branch through the helper and pass the resulting bundle-
confined receipt to `work-state`. The unpublished satisfied-prerequisite `FAILED`
terminal above retains branch ownership and does not enter this review handoff:

```sh
"$WORK_GIT_BINDING" release \
  --work-kind <feature|bug_fix> --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 \
  --role implementation --attempt <attempt> --revision-epoch <revision> \
  --launch-marker <marker> \
  --thread-id <id> --client-thread-id <id> \
  --worktree <implementation-worktree> --git-common-dir <common-dir> \
  --branch <short-branch> --bound-sha <head> --observed-remote-sha <head> \
  --observation-epoch <epoch> --bundle-root <canonical-absolute-bundle-root> \
  --receipt <absolute-bundle-release-receipt>

"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role implementation --state quiescent \
  --implementation-receipt <bundle-relative-terminal-receipt> \
  --branch-release-receipt <bundle-relative-release-receipt>
```

`RELEASED` must show detached `HEAD` and `branchOwners: []`. Only then probe
`gitnexus-pr-review`. A miss creates a root blocker without reserving review. On
success, reserve the review and create one project worktree from the local PR branch at
the exact remotely observed head with `model: "gpt-5.6-sol"`, `thinking: "high"`, and
this initial goal:

| Spawn | Role | Audience | Tier | Brief mode | Receipt mode | Justification |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Exact-head review owner | INTERNAL | REVIEWER | NORMAL | JSON_RECEIPT | Checkout transfer precedes consequential remediation and push authority. |

Review-task creation and the `RUN_REVIEW` activation carry standing authority for
only the bounded blind lenses selected by the registered deterministic review-risk
route: standard authorizes zero, elevated authorizes exactly one, and critical
authorizes at most two. Do not add a blanket no-descendants instruction that cancels
this registered lens budget.

```text
RUN_REVIEW
LAUNCH: <work-id>/<PR-*>/review/<attempt>
PARENT_TASK_ID: <delegation source_thread_id>
PR: <URL>
START: <exact remote PR-head ref>@<SHA>
BRANCH: <exact PR branch>
REMOTE: <remote name and URL>
BASE: <accepted base ref>@<SHA>
OWNED_PATHS: <frozen path list>
BUNDLE_SNAPSHOT: <slice>:<classification>:<40-hex commit>:<bundle-relative lifecycle path>@<manifest digest>
IMPLEMENTATION_RELEASE: <receipt path, digest, and observation epoch>
STARTUP_FACTS_SHA256: <digest of the complete immutable startup fact block>
LEASE_PATH: <exact absolute task-local path outside the repository and bundle>
LABEL_POLICY_PATH: .github/issue-label-policy.json
LABEL_POLICY_SHA256: <SHA-256 of exact policy bytes>

Verify LAUNCH, repository, exact PR head, clean checkout, and companion availability.
Run the supplied `work-state verify-pr-snapshot` command before semantic review and
again against any remediated head before terminalization. Missing, stale, inconsistent,
or alternate-lifecycle-path bundle bytes are a material blocker.
The branch attachment carries the same exact `snapshotAuthorization` separately from
`OWNED_PATHS`. Review-lease startup accepts the already materialized snapshot only
when its commit, slice, classification, canonical path, manifest digest, file modes,
and bytes match the current head. It never adds the bundle root to ordinary ownership.
Ordinary feature and full bug-fix review activation requires that non-null
authorization. Only a caller that supplies the canonical explicit compact fast-fix
record may activate with null snapshot authorization. Authenticated lifecycle paths
are excluded from ordinary changed-path ownership and compact-record classification.
Every remediation rotation remains restricted to `OWNED_PATHS`; the reviewer may not
edit or refresh the frozen snapshot.
In a terminal parallel wave, the deterministic final carrier delays this `pr_open`
attention handoff until its peers are terminal and it has consumed the coordinator's
already observed main SHA through the current-owner path. Other members use the normal
intermediate snapshot handoff. Carrier identity is topology-derived, and a successful
implementation terminal before its active `pr_open` handoff is rejected; no later
snapshot-only task or PR is authorized.
Hash and validate the complete startup fact block, then use the existing task-local
`task-git-binding attach-review` and `review-lease activate` helpers before loading a
review skill, fan-out, edit, commit, or push. Construct their arguments only from the
validated fact block and live task-local checkout; no coordinator activation packet
exists. Correct a retryable startup error in this same task; do not request or create
another task. After startup succeeds, load [$gitnexus-pr-review] and own its
native review, remediation, verification, GitHub thread disposition, push, and
progress-based loop. This registered activation authorizes only the blind-lens count
selected by the deterministic review-risk route: standard authorizes zero, elevated
authorizes exactly one, and critical authorizes at most two. Preserve the primary
review owner as the sole repository writer. Create each required lens only after
freezing the primary local review and before remote intake. Each lens is exact-head,
read-only, remote-blind, and limited to one named risk surface, with no mutation,
planning, remediation, verification, publication, or descendants. Confirm every lens
is quiescent before remote intake and again before terminalization. Do not request
later user or coordinator reauthorization for these required lenses. An explicit
limitation in the current user request still narrows authority; return a conflict
before knowingly launching an unsatisfiable review.
Before follow-up publication, verify LABEL_POLICY_PATH exists at HEAD and its exact
bytes match LABEL_POLICY_SHA256. Return a policy mismatch to the spec-work root
without creating or relabelling an issue. Return each verified follow-up publication
receipt so the root can record it in the originating slice's RECEIPT.md.
Return product ambiguity, plan change, scope expansion, or a shared boundary to the
spec-work root. For a material replan, finalize blocked and release the branch before
the terminal callback. Do not merge, edit the work bundle except through the supplied
receipt-writing helpers, or copy implementation reasoning or expected findings. At
terminal state send exactly one direct callback to PARENT_TASK_ID with
codex_app.send_message_to_thread.
```

The startup fact block contains every stable identity, branch, head, prior-release,
bundle-root, receipt, base, remote, owned-path, and lease value. The task resolves only
its own working directory and Git common directory. The attachment receipt may contain
literal null runtime thread identities; the root later reconciles those with the
single observed task created for the exact launch marker. Successful attachment
publishes the canonical `spec-work-task-git-binding` receipt.

If the live PR head advances before attachment, keep the same creating review
reservation. The task or coordinator may run `adopt-review-head` only from a clean,
detached observer in the same Git common directory after the causal branch release.
The helper accepts only an ownerless remote head whose two parents are exactly the
recorded PR head and the current descendant main or stacked base, verifies that its
tree equals Git's deterministic clean merge and that the final PR paths remain inside
the frozen owned paths, advances the ownerless local branch ref plus the detached
observer, and publishes a create-only `spec-work-review-head-adoption` receipt.
Arbitrary descendants, edited merge trees, rewritten base history, branch owners,
dirty worktrees, stale release receipts, and identity mismatches fail without a
receipt, ref, or checkout mutation.

```sh
"$WORK_GIT_BINDING" adopt-review-head \
  --work-kind <feature|bug_fix> --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 --role review --attempt <attempt> \
  --revision-epoch <revision> --launch-marker <marker> \
  [--thread-id <id>] [--client-thread-id <id>] \
  --worktree <detached-observer-worktree> --git-common-dir <common-dir> \
  --branch <short-branch> --bound-sha <recorded-head> \
  --observed-remote-sha <adopted-head> --base-ref <origin/main|upstream-branch> \
  --base-sha <recorded-base> --observed-base-sha <current-base> --pr <number> \
  --observation-epoch <epoch> --bundle-root <canonical-absolute-bundle-root> \
  --receipt <absolute-bundle-head-adoption-receipt> \
  --release-receipt <absolute-causal-release-receipt> \
  --release-receipt-digest <digest> --release-observation-epoch <epoch> \
  --owned-path <coordinator-owned-path> [...]

"$WORK_STATE" adopt-review-head \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --receipt <bundle-relative-head-adoption-receipt>
```

The root consumes the adoption receipt before recording review activation. This
atomically advances the slice head, base, and observed main while preserving the
attempt, launch marker, external task identity, authorization, PR, and release
lineage. The same task then uses the ordinary attachment sequence below with the
adopted head and base; no replacement attempt or new review lease kind is created.
For a stacked slice, the base advances to the clean direct upstream branch head and
the global observed-main binding is unchanged. A later material finding may
reauthorize the same implementation owner because the persisted adoption receipt
proves the otherwise different implementation-output and current PR heads. If that
review rotates and later resumes from a same-plan block or is reopened after a clean
result because its base advanced, coordinator state retains the authenticated blocked
or clean terminal as a contiguous lineage segment before the fresh attachment replaces
it. Repeated rolling segments remain anchored to the original adopted startup head.
If typed supersession later replaces the review task, coordinator validation resolves
the adoption from that task's retained history and combines it with the current
replacement task's terminal when reauthorizing the implementation owner.

If the final human checkpoint reports an issue after review reaches CLEAN, the root
persists the canonical BLOCKED receipt and invokes `reopen-final-ui-remediation`.
The atomic transition archives clean review proof, advances the slice revision, and
reopens the same implementation as `authorized` and review as `blocked` without
changing attempt, thread, client-thread, worktree, branch, or PR lineage. The
implementation then uses the ordinary attachment and activation sequence with the
archived CLEAN terminal digest/epoch as its causal release; coordinator validation
authenticates that exact terminal before activation. The review resumes on the
repaired descendant head through its existing lease path.

```sh
"$WORK_GIT_BINDING" attach-review \
  --work-kind <feature|bug_fix> --work-id <work-id> --work-bundle <work-id> \
  --slice-id PR-01 --role review --attempt <attempt> \
  --revision-epoch <revision> --launch-marker <marker> \
  --worktree <review-worktree> --git-common-dir <common-dir> \
  --branch <short-branch> --bound-sha <head> --observed-remote-sha <head> \
  --observation-epoch <epoch> --bundle-root <canonical-absolute-bundle-root> \
  --receipt <absolute-bundle-attachment-receipt> \
  --release-receipt <absolute-causal-release-receipt> \
  --release-receipt-digest <digest> --release-observation-epoch <epoch> \
  --owned-path <coordinator-owned-path> [...] \
  [--reauthorization-digest <coordinator-revision-authority-digest>] \
  --snapshot-slice-id PR-01 --snapshot-commit <40-hex-commit> \
  --snapshot-path <canonical-lifecycle-path> \
  --snapshot-classification <intermediate|final> \
  --snapshot-manifest-digest <64-hex-digest>

"$REVIEW_LEASE" activate \
  --attachment <absolute-bundle-attachment-receipt> \
  --lease <exact-absolute-task-local-lease> --remote <remote> \
  --base-ref <base-ref> --base-sha <base-sha>

"$WORK_STATE" task-transition \
  --work-bundle <bundle> --expect-revision <N> --id PR-01 \
  --role review --state active --thread-id <id> \
  --task-worktree <absolute-path> --bound-sha <head> \
  --branch-attachment-receipt <bundle-relative-attachment-receipt>
```

The child may begin semantic review as soon as the startup sequence succeeds. The root
concurrently reconciles the exact returned task/worktree, validates the attachment
receipt, and records review `creating -> active`; any mismatch stops and accounts the
same task. There is no preparation callback, second activation message, or disposable
review task.

`release`, `adopt-implementation`, `adopt-review-head`, `attach-review`,
`attach-implementation`, and `verify-review` serialize on one process-shared lock
for the exact Git common directory and branch. Publishing commands confine create-only
receipt I/O beneath the explicit canonical bundle root using no-follow, descriptor-
anchored paths. `verify-review` is read-only and repeats the clean/head/common-directory/unique-owner
checks against the attachment digest. Startup failure returns `ACTIVATION_ABORTED`;
the root stops and accounts the same attempt, records release proof and a blocker, and
permits no review fan-out or remediation. Review retains the branch only while active
or during an authority-retaining blocked result.

After each authorized non-force remediation push, the review owner writes one
task-local `spec-work-review-mutation` receipt and runs `review-lease advance`. The
same operation handles a clean merge-based base sync. It proves descendant ancestry,
unchanged task/worktree/common-directory/branch/remote identity, clean checkout,
exact remote heads, the authorized mutation receipt, and both mutation and final PR
paths within the frozen owned paths. It appends one compact rotation to the task-local
ledger and invalidates semantic-review, CI, mergeability, and terminal-verification
evidence. It never writes the coordinator bundle or calls the parent. Force pushes,
non-descendant heads, dirty worktrees, remote or identity changes, absent mutation
evidence, and owned-path escape fail closed.

If the remote base advances by descendant commits during review startup, activate the
lease against the recorded base SHA. Activation verifies the live remote base still
descends from that anchor. The same review task then immediately merges the live base,
pushes without force, and records the rotation through `review-lease advance` before
semantic review. The root records the new base only with the resulting terminal
evidence through `--observed-main-sha`; it does not rewrite the creating task's base
before activation. The same leased merge path owns any later base advance. Supported
input spellings `origin/main`,
`refs/remotes/origin/main`, and `refs/heads/main` canonicalize to
`refs/heads/main` at the lease boundary. Stale base alone is never a task-replacement
predicate.

An authority-retaining blocked result may keep the lease and branch attached for
same-plan remediation. A material replan instead finalizes the lease blocked and
releases the branch. After the same implementation task publishes and releases the
next slice revision, send the same review task one `RESUME_REVIEW` message containing
the complete fresh `attach-review` command and a `review-lease resume` command carrying
`--base-ref <current-state-base-ref>` and the same authenticated
`--release-receipt <absolute-causal-release-receipt>`. The review owner runs
both unchanged before new remediation, advance, or terminal finalization.
`resume` requires the same
attempt/task/worktree/branch/remote identity, the next `revisionEpoch`, the exact
coordinator-validated attachment inventory and revision authority digest, and a
descendant revised head; it resets the rolling segment at that head while preserving
the task identity. A clean lease may adopt a different normalized canonical base ref
only after its remote descendant relationship is verified. A blocked lease cannot
change base identity. If that base advances after revised implementation releases,
resume retains the base actually integrated into the revised head; the review task
then merges the live descendant base and records it through `review-lease advance`.
The prior blocked terminal receipt remains digest-bound in revision history as the
completed prior-revision segment.

After a rotation, review only the exact delta and affected or uncertain surfaces.
Run a full re-review only after intent expansion, critical-surface expansion,
uncertain impact, conflicted base integration, or an attempted lease escape. Ordinary
findings stay with the review; root-owned ambiguity or scope changes return to Plan.
Demonstrated non-progress returns a root-owned blocker and is not a replacement
predicate.

Every review attachment cites the greatest causal typed branch release before its
attachment epoch from the actual prior owner. That owner is the implementation task
for initial or revised review, or the superseded review task for a typed replacement;
an older release is stale once a later owner released the branch.
`attach-implementation` also re-observes the live `origin` branch head inside the
locked attachment operation and rejects if it no longer equals the bound startup SHA.

Dependency-edge downstream launch waits for current-head-clean, quiescent upstream
review. Restarting upstream review is forbidden while dependent tasks are active.

## Callback-backed parent waits

For implementation and review waits, an attention or terminal callback is the primary
wake and the parent owns exactly one roughly 400-minute missed-callback recovery
heartbeat only after a successful
current-generation task-messaging capability observation and exact attempt/thread
binding. For a PR/check wait, the same mode requires successful registration for the
exact PR/head and its registration receipt. Probe the same-task heartbeat capability
at each use site too. If task messaging or event registration fails, select
callback-free active polling only after successfully creating and observing one
parent-owned polling heartbeat for the exact attempt/thread or PR/head. If heartbeat
capability or creation fails, preserve those exact generation facts, do not yield or
claim active polling or a future wake, and report `manual_resume_required` with the
exact parent plus task attempt/thread or PR/head identity, or report blocked. Do not
describe either fallback as callback-backed.

The parent recovery payload binds the exact task attempt/thread or
PR/head/registration receipt for that generation. Before any callback or recovery
action, reload the system of record and require that identity to remain current and
check the existing parent task or phase transition for prior consumption.
Exact-current terminal producer evidence is consumable once; terminal evidence is
not itself proof of parent consumption. The parent first passes the ordered typed
chain to public `work-state reconcile-callback-chain`: the implementation terminal,
then the exact branch-release receipt for every ordinary terminal. The current
scope-expansion and satisfied-prerequisite variants instead retain the branch owner
and omit release. Only that successful
durable transition proves consumption. Stale, incomplete, reordered, ambiguous, or
tampered chains reject without mutation. Replaying the exact already-persisted
current chain is byte-identical and creates no revision, task, heartbeat, or
redispatch.
When the current implementation owner merged an advanced required base before
handoff, pass its exact `--observed-main-sha`; reconciliation applies that owner
base synchronization before terminal and release authentication.

Before yielding to a reachable current non-Pro task, review, or PR/check generation,
the root creates and observes that generation's distinct wait owner only after the
required callback capability or registration succeeds, and keeps only that one
recovery owner.
Terminal callbacks continue the parent immediately; they never wait for the recovery
heartbeat. After the exact callback is durably consumed, the parent deletes that
generation's recovery heartbeat in the same continuation. A roughly 400-minute task
wake performs one exact missed-callback/liveness reconciliation; it does not assert
terminal state, delay a callback that arrives first, or begin polling. Stale,
duplicate, already-consumed, or wrong-task callbacks and recovery wakes are idempotent
no-ops. After exact task/thread binding and successful recovery-heartbeat creation, the
parent ends its turn with no routine progress commentary and no `wait_threads`,
`read_thread`, sleep, or equivalent polling. Callback-free active polling and ChatGPT
Pro retain their distinct policies. Children send exactly one terminal callback and
never poll, create, update, or own recovery heartbeats.

## Terminal handoff and callback

Implementation may commit, push, and open/update only its assigned PR. It does not
review or merge it. Its compact terminal report includes checkpoint/plan digest,
requested/effective route when observable, changed paths, verification, branch/start/
base/head, PR, delegated-task quiescence, blockers, deviations, and residual risk.

Before first publication, the implementation owner reads
`.github/pull_request_template.md` and renders every non-comment field from the
checkpointed primary specification, exact slice plan, assigned `AC-*` blocks, current
implementation evidence, and the applicable linked issue. GitHub UI, CLI, and API
creation have the same contract. Copy the slice's accepted outcome, included scope,
non-goals, and required outcomes into the public body; do not substitute a summary,
checkpoint list, bundle path, or task-local working notes. Use a closing keyword only
when this PR completely satisfies the linked issue; otherwise use a non-closing
reference. The Contract, Scope, and required-outcome text freeze at PR creation.
Implementation and evidence may track later heads. An accepted contract change must
first update its semantic source through the owning workflow, then update the same PR
and append an explicit Contract amendment with the source and reason. Silent contract
drift is a review blocker.

After the implementation owner publishes its ordinary head and sends its attention
callback, the root consumes the exact PR/head and acceptance handoff, freezes the
coordinator-owned canonical work bundle in an isolated projection, and resumes that
same bound owner with its slice, commit, lifecycle path, classification, and manifest
digest. Intermediate PRs carry `in_progress/` snapshots with `phase != complete`; the
designated final PR carries a `phase=complete` `completed/` projection whose own slice
remains `pr_open`, explicitly distinguishing completed work/acceptance from pending
merge. Run `work-state verify-pr-snapshot --snapshot-commit <commit> --pr-head <head>
--slice-id <PR-*> --manifest-digest <digest>` before handing the PR to review. The
operation compares immutable approved bytes to the exact head and does not compare
later live coordinator receipt, PR, or quiescence writes. A mismatch blocks
publication; never compensate with a later documentation-only PR.

The root re-observes these facts, writes `spec-work-implementation-terminal`, and
records implementation `quiescent`. The typed receipt binds work, slice, worktree,
branch/base/start, attempt/thread, output head/PR, verification and changed-path
digests, result, and descendant-quiescence digest. Result is `SUCCEEDED`, `FAILED`,
`SCOPE_EXPANSION_REQUIRED`, or `ACTIVATION_ABORTED`; only a verified `SUCCEEDED` result
may publish. Every result uses its exact current receipt shape.
`SCOPE_EXPANSION_REQUIRED` alone adds sorted nonempty required owned paths, one
declared acceptance ID, the unchanged owning seam, and nonempty regression and
verification obligations. It is valid only before publication and before the
bound head changes. These fields are evidence for planning and never authorize
work by themselves. The separately resumable prerequisite `FAILED` variant alone adds
the canonical prerequisite fingerprint, optional issue, prerequisite PR/head, and
required base ref described above; partial or additional fields reject.

Immediately before the one terminal callback, the review owner runs exactly one
supplied `review-lease finalize` command against its task-local lease and final
live checkout. Finalization revokes rolling mutation authority before publishing
the bundle-confined `spec-work-review-lease-terminal` receipt. Clean finalization
requires the lease's current base to equal the live remote base; if it advanced,
the owner first merges it and records the existing typed `base_sync` advance. An
interrupted `FINALIZING_*` state is non-authorizing and may complete only by
rerunning the same finalize command. The root consumes that receipt, then creates
`spec-work-review-clean` when the result is clean. One revision-CAS transition
validates the initial activation digest, rotation ledger digest, final head, final
changed-path boundary, terminal result, and clean receipt; there is no active-head
state rotation or parent reattachment. `spec-work-review-clean` remains bound to work,
slice, review worktree, branch/PR/current head, attempt/thread, capability probe, and
the terminal lease digest plus disposition/task/dependency-quiescence artifacts and
their byte digests. Merge
evidence binds that review worktree, reviewed head, PR, and resulting main SHA.

Only after finalize returns terminal evidence does the child call
`codex_app.send_message_to_thread` exactly once with
the literal `PARENT_TASK_ID`, even when task listings omit the parent. Do not use local
collaboration messaging to `/root`, emit routine progress callbacks, poll, sleep, add a
heartbeat, or create callback-proof artifacts. Claim delivery only after the direct
tool call succeeds. If it is unavailable or fails, report
`manual_resume_required` with the exact parent and child task IDs; the terminal work
result remains valid and the user/root may resume it manually. Successful notification
delivery is not proof that the parent resumed.

## Revision, replacement, supersession, and closure

Keep the same implementation and review task identities when a material finding
replans the same accepted goal on the same open PR. Increment the slice revision epoch,
archive prior-revision terminal evidence, reauthorize with the new exact checkpoint,
and require fresh revision-bound attachments and receipts. Change route only between
attempts. Before a typed supersession or transfer, quiesce the branch owner and record
typed stop and branch-release proof. An active resumed review cannot use the release
that causally preceded its current attachment; supersession requires a release with
a newer observation epoch. Preserve tombstones and never reuse superseded task IDs,
worktrees, launch markers, or evidence epochs. Active-owner supersession clears the
former owner's live branch-attachment fields after stop and the newer release proof
are recorded. There is no attempt cap.
The only registered replacement predicates are `unrecoverable_task_runtime`,
`unrecoverable_worktree`, `repository_identity_mismatch`,
`pr_identity_unrecoverable`, and `separate_deliverable_user_decision`; ordinary stale
base, correctable startup metadata, or material replanning of the same accepted goal
cannot satisfy them.

A malformed but task-authentic terminal, release, attachment, or review receipt is a
same-task `bounded_correction` request when the producer result, task identity, owned
paths, and exact head remain valid. A uniquely provable state/provider reference may
use `reconcile_authoritative_state`. Neither class authorizes task replacement,
implementation restart, repeated review, or repeated verification; only one of the
registered predicates above can cross that boundary.

Before replan, acceptance refresh, base refresh, merge, or completion, every affected
task must be outside `creating`, `prepared`, and `active`. A sibling/upstream merge
records current main, invalidates stale acceptance and review proof, and reopens each
affected open PR in its existing review task. That task refreshes or resumes its lease,
performs the merge-based base sync, rotates the same PR head, and reruns affected
verification. A base advance alone never supersedes a task or creates replacement
lineage. The spec-work root alone writes bundle state, approves merges, and declares
completion.
