# Fast Fix Lane

Load this reference only after `scripts/bug-fix-route` returns `fix_fast` for a
requested project bug fix.

## Authority and ownership

The invocation authorizes one implementation owner to keep the job from the
confirmed evidence through branch creation, focused and relevant broader tests,
one atomic commit, push, and PR creation. It also authorizes normal remediation
requested by the independent review owner and progression through clean
merge-readiness gates. Ask the user only for an unresolved consequential
decision, explicit waiver, unavailable required capability, or the applicable
final UI acceptance.
This authority is limited to the confirmed cohesive fix; it excludes secrets,
unrelated working-copy changes, and work outside the classified route.

Do not create a spec-work bundle, `state.json`, `STATUS.md`, task roster,
heartbeat, ChatGPT Pro task, blind-completeness pass, or separate implementation
task. The current thread is the implementation owner.

## Required sequence

1. Preserve the confirmed root-cause trace and completed regression abstraction
   in the implementation plan or working notes.
2. Write the failing root-cause regression test or executable acceptance check.
3. Implement the smallest production-quality correction at the owning seam.
4. Run focused verification, applicable broader tests, lint/build gates, and
   the existing UI display gate when the change is user-visible.
5. Create
   `docs/bug_fixes/completed/<YYYY-MM-DD-slug>/BUG_FIX.md` on the same branch.
   The implementation PR must contain this record; a later coordinator-only
   documentation commit is not acceptable.
6. Commit, push, and open one cohesive implementation PR. Before publication,
   read `.github/pull_request_template.md` and fill every non-comment field from
   the confirmed bug record, compact `BUG_FIX.md`, current implementation
   evidence, and applicable linked issue even when using a CLI or API. Copy the
   accepted outcome, included scope, non-goals, and required outcomes into the
   public description; a summary-only body or a pointer to working notes is not
   valid. Use a closing keyword only when the PR completely satisfies the
   linked issue; otherwise use a non-closing reference. The Contract, Scope,
   and required-outcome text freeze at PR creation. Implementation and evidence
   may track later heads, but an accepted contract change must update its
   durable source first and then append an explicit Contract amendment with the
   source and reason.
7. Follow the executable handoff below. The implementation owner releases the
   branch with the repository-owned `task-git-binding`, then invokes one
   independent GitNexus review owner with the immutable startup facts. The
   review owner attaches the exact branch and activates the repository-owned
   rolling lease before semantic review, fan-out, edit, commit, or push. Bind
   `.github/issue-label-policy.json` and its exact SHA-256 in that handoff so
   review-time follow-up publication cannot use stale or ambient policy.
   Include `docs/bug_fixes/completed/<workId>/BUG_FIX.md` in the frozen owned
   paths. One successful attachment transfers branch ownership to that
   independent reviewer for the review lifetime; routine descendant head
   rotations do not return to the parent for reattachment.
   After successful review attachment and rolling-lease activation, that
   independent review owner is explicitly authorized for the review lifetime to
   reply to review findings and resolve terminal GitHub review threads on the
   route-owned pull request without a separate user prompt.
   The review risk comes from the route receipt: standard has no child lens,
   elevated has one, and critical has at most two plus a required
   remote-provider pass.
8. Let that review owner complete each lightweight review cycle in this order:
   collect and validate the findings for the reviewed head, batch the valid
   blockers into the smallest cohesive correction, implement with one writer,
   and verify. Push the cohesive remediation commit and advance the same rolling
   lease with the authorized mutation receipt before reviewing the new head.
   Reply to every finding from that cycle with the outcome and the fixing
   commit plus verification when applicable; reply to an invalid finding with
   concise evidence instead of creating a commit. Resolve every terminal thread.
   Review the new head and repeat until the current-head review returns no new
   blockers and no finding from any completed cycle remains unanswered or
   unresolved.
   The installed GitNexus review owner owns the provider rereview decision.
   Every remediation head still receives its local affected-delta review; the
   owner applies its conditional risk predicates to decide whether that head also
   needs a provider pass. Do not turn the remote provider into an approval gate or
   duplicate those predicates in this caller.
   The same owner publishes or reuses any verified actionable out-of-scope
   review finding under the bound label policy, then returns the policy's exact
   publication receipt fields. A policy-path or digest mismatch fails closed
   before issue mutation. Do not repeat a verified publication in the parent.
   If a finding arrives after a newer head is already published, validate it
   against the reviewed head and then check the current head. When the current
   head already fixes it, reply with the existing fixing SHA and verification,
   resolve the terminal thread, and continue the current-head review without an
   unnecessary commit.
   If authoritative current-head evidence arrives after the rolling lease was
   finalized clean, the same review owner publishes a fresh exact attachment at
   a new create-only receipt path under the existing task identity and runs
   `review-lease resume --reason late_evidence` before any reply, resolution,
   edit, commit, or push. This recovery is valid only when revision, head, base,
   identity, authority, worktree, branch, and remote remain exact; ordinary
   newer-base recovery keeps the default `base_sync` reason. Preserve every
   earlier attachment and terminal receipt, and finalize the reopened lease at
   another new create-only terminal path. This is a task-local FAST-FIX
   recovery, not a spec-work coordinator state transition.
   If clean lease finalization reports a compact-record path mismatch, treat it
   as a fixable PR-causal documentation blocker: update the record in the same
   PR, push the cohesive correction, advance the same lease, and rerun
   invalidated evidence before finalizing again.
   The initial atomic implementation commit may remain distinct; meaningful
   review-remediation commits may remain distinct when they preserve useful
   causal or engineering history. Atomicity describes each commit's cohesion,
   not the final PR integration strategy.
9. Do not merge while any returned review finding remains unanswered or any
   terminal review thread remains unresolved. Merge only after the latest-head
   review is clean and deterministic CI, mergeability, base-sync, publication,
   changed-path, and applicable UI gates pass. Finalize the same rolling lease
   clean through the repository-owned executable; clean finalization validates
   final changed-path containment and exact compact-record `## Changed Paths`
   equality. Confirm the compact bug record is reachable from `main` with the
   implementation.
   After terminal CLEAN, perform exactly one freshness check: compare
   authoritative remote main with the base bound by the clean receipt. If it is
   unchanged, merge immediately with a merge commit. If authoritative remote
   main advanced, use the existing merge-based base synchronization and
   current-head revalidation, then merge automatically with a merge commit when
   terminal CLEAN again. Do not ask the user to choose a merge method. Do not
   request separate merge consent. Stop only for a named critical blocker or an
   external condition that prevents merge. Do not repeat head, CI,
   mergeability, thread, provider, terminal-verification, attachment, or
   lease-finalization gates merely because merge execution follows a published
   clean result. Atomic-commit guidance never selects squash or rebase.
   After the merge is durably reachable from authoritative main, apply
   `.github/issue-label-policy.json` to the verified in-scope issue owner.
   Preserve its existing prose, append only missing verified completion evidence,
   add the canonical fingerprint marker when absent, reconcile exact labels, and
   close it without requesting separate user authorization. If the PR closing
   keyword already closed it, perform the same body-evidence verification and
   refetch the closed owner rather than skipping maintenance. Do not close for a
   fix not durably effective on authoritative main, and do not create a
   replacement or duplicate issue.
   Receipt-driven repair, retry, resumption, callback reconciliation,
   branch-ownership correction, and merge-based synchronization needed to
   finish this same accepted fix have standing authority. Continue without a
   repeated permission prompt. Fail closed only for scope expansion, a
   destructive action, security risk, identity ambiguity, changed intent, or a
   separate deliverable.

If scope or evidence crosses a fast predicate, stop before further mutation and
restart as `fix_full` through the canonical bundle. Unsupported partial fast-lane
state is not migrated into spec-work automatically.

## Executable review handoff

`review-lease` is not an installed command and is not owned by the GitNexus
review skill. Use these repository executables from the repository root:

```sh
WORK_GIT_BINDING=.agents/workflows/spec-work-orchestrator/scripts/task-git-binding
REVIEW_LEASE=.agents/workflows/spec-work-orchestrator/scripts/review-lease
```

Create one ephemeral authority root outside every Git worktree. Its basename
must equal `<workId>` because `task-git-binding` validates that identity. This
directory contains only runtime release, attachment, and terminal receipts; it
is not a spec-work bundle and does not contain `state.json`, `STATUS.md`, a task
roster, or planning artifacts. Set the helper's required `--work-bundle` value
to the same `<workId>`.

```sh
FAST_FIX_TEMP_ROOT="${TMPDIR:-/tmp}"
FAST_FIX_TEMP_ROOT="${FAST_FIX_TEMP_ROOT%/}"
FAST_FIX_RUNTIME_PARENT="$(
  mktemp -d "$FAST_FIX_TEMP_ROOT/project-fast-fix.XXXXXX"
)"
FAST_FIX_AUTHORITY_ROOT_RAW="$FAST_FIX_RUNTIME_PARENT/<workId>"
mkdir "$FAST_FIX_AUTHORITY_ROOT_RAW"
FAST_FIX_AUTHORITY_ROOT="$(
  cd "$FAST_FIX_AUTHORITY_ROOT_RAW" && pwd -P
)"
```

After the implementation commit is pushed and local/remote heads agree, the
implementation owner releases the branch:

```sh
"$WORK_GIT_BINDING" release \
  --work-kind bug_fix \
  --work-id <workId> \
  --work-bundle <workId> \
  --slice-id PR-01 \
  --role implementation \
  --attempt 1 \
  --revision-epoch 1 \
  --launch-marker <workId>/PR-01/implementation/1 \
  --worktree <absolute-implementation-worktree> \
  --git-common-dir <absolute-git-common-directory> \
  --branch <short-pr-branch> \
  --bound-sha <exact-remote-pr-head> \
  --observed-remote-sha <exact-remote-pr-head> \
  --observation-epoch 1 \
  --bundle-root "$FAST_FIX_AUTHORITY_ROOT" \
  --receipt \
  "$FAST_FIX_AUTHORITY_ROOT/implementation-release.json"
```

Hash the exact release-receipt bytes:

```sh
shasum -a 256 "$FAST_FIX_AUTHORITY_ROOT/implementation-release.json"
```

Invoke the independent review owner with this complete immutable startup fact
block:

```text
RUN_FAST_FIX_REVIEW
WORK_ID: <workId>
PR: <pull-request-url>
BRANCH: <short-pr-branch>
REMOTE: <remote-name> <remote-url>
BASE: <accepted-base-ref>@<accepted-base-sha>
START: <short-pr-branch>@<exact-remote-pr-head>
OWNED_PATHS: <complete frozen repository-relative path list>
COMPACT_RECORD: docs/bug_fixes/completed/<workId>/BUG_FIX.md
AUTHORITY_ROOT: <absolute-ephemeral-authority-root>/<workId>
IMPLEMENTATION_RELEASE: <absolute-release-path>@<sha256>@1
ATTACHMENT_PATH: <absolute-authority-root>/review-attachment.json
LEASE_PATH: <absolute-task-local-path-outside-repository-and-authority-root>
LABEL_POLICY_PATH: .github/issue-label-policy.json
LABEL_POLICY_SHA256: <sha256-of-exact-policy-bytes>
```

The block contains:

- work ID, PR URL, branch, remote name and URL;
- exact base ref/SHA and exact remotely observed PR head;
- frozen owned paths, including the canonical compact record;
- canonical implementation-release path, SHA-256 digest, and observation epoch;
- canonical ephemeral authority root and a not-yet-existing attachment path;
- an exact absolute task-local lease path outside the repository and authority
  root; and
- the repository-relative issue-label-policy path and SHA-256 of its exact
  bytes at the reviewed head.

The independent review task resolves its own absolute worktree, Git common
directory, and non-null immutable task/thread identifier. Before attaching, it
must independently verify the handed release bytes and their binding. Split
`IMPLEMENTATION_RELEASE` into its path, expected SHA-256 digest, and positive
observation epoch, then run this fail-closed preflight with the immutable
startup facts:

```sh
python3 - \
  <implementation-release-path> \
  <implementation-release-sha256> \
  <workId> \
  <short-pr-branch> \
  <exact-remote-pr-head> \
  <absolute-git-common-directory> \
  <release-observation-epoch> <<'PY'
import hashlib
import json
from pathlib import Path
import sys

path, expected_digest, work_id, branch, head, common_dir, epoch = sys.argv[1:]
encoded = Path(path).read_bytes()
if hashlib.sha256(encoded).hexdigest() != expected_digest:
    raise SystemExit("implementation release digest mismatch")
payload = json.loads(encoded)
expected = {
    "schema": "spec-work-task-git-binding",
    "kind": "branch_release",
    "workKind": "bug_fix",
    "workId": work_id,
    "workBundle": work_id,
    "sliceId": "PR-01",
    "role": "implementation",
    "gitCommonDir": common_dir,
    "branch": branch,
    "branchRef": f"refs/heads/{branch}",
    "boundSha": head,
    "worktreeHeadSha": head,
    "localBranchSha": head,
    "observedRemoteHeadSha": head,
    "symbolicRef": None,
    "branchOwners": [],
    "clean": True,
    "result": "RELEASED",
    "observationEpoch": int(epoch),
}
for key, value in expected.items():
    if payload.get(key) != value:
        raise SystemExit(f"implementation release {key} mismatch")
PY
```

Only after that preflight succeeds may the review task run:

```sh
"$WORK_GIT_BINDING" attach-review \
  --work-kind bug_fix \
  --work-id <workId> \
  --work-bundle <workId> \
  --slice-id PR-01 \
  --role review \
  --attempt 1 \
  --revision-epoch 1 \
  --launch-marker <workId>/PR-01/review/1 \
  --thread-id <literal-current-review-task-id> \
  --worktree <absolute-review-worktree> \
  --git-common-dir <absolute-git-common-directory> \
  --branch <short-pr-branch> \
  --bound-sha <exact-remote-pr-head> \
  --observed-remote-sha <exact-remote-pr-head> \
  --observation-epoch 2 \
  --bundle-root <AUTHORITY_ROOT> \
  --receipt <ATTACHMENT_PATH> \
  --release-receipt-digest <implementation-release-sha256> \
  --release-observation-epoch 1 \
  --owned-path <repository-relative-frozen-path>
```

Repeat `--owned-path` for every frozen authorized path. The task then activates
the lease from that exact attachment:

```sh
"$REVIEW_LEASE" activate \
  --attachment <ATTACHMENT_PATH> \
  --lease <LEASE_PATH> \
  --remote <remote-name> \
  --base-ref <accepted-base-ref> \
  --base-sha <accepted-base-sha> \
  --compact-bug-fix-record \
  docs/bug_fixes/completed/<workId>/BUG_FIX.md
```

Any missing release provenance, dirty checkout, common-directory mismatch,
head mismatch, remote mismatch, null reviewer task identity, or non-unique
branch ownership fails before review authority starts. The reviewer must retain
the same task identity, attachment, worktree, branch, and lease for the review
lifetime.

Before loading `gitnexus-pr-review`, verify `LABEL_POLICY_PATH` exists at the
exact reviewed head and its bytes match `LABEL_POLICY_SHA256`. Before every
follow-up issue mutation, recheck that binding and apply the policy's exact
fingerprint, all-state discovery, authoritative-fetch, owner-reuse, label,
evidence, and post-mutation verification contracts. Return `issue_url`,
`verified_state`, `fingerprint`, `fingerprint_comparison`, `issue_type`,
`severity`, `expected_labels`, `disposition`, and
`label_verification_status` for each published or reused follow-up. If the
binding is missing or mismatched, return the blocker without creating,
relabelling, or closing an issue.

If authoritative evidence arrives after a clean terminal on the unchanged
head/base, never replace or reuse the startup attachment or prior terminal
receipt. Choose the next positive observation epochs and new authority-root
paths, rerun the exact original `attach-review` command with only
`--observation-epoch` and `--receipt` changed, then resume:

```sh
LATE_ATTACHMENT_PATH="<AUTHORITY_ROOT>/review-attachment-late-evidence-<next-epoch>.json"
LATE_TERMINAL_PATH="<AUTHORITY_ROOT>/review-lease-clean-late-evidence-<later-epoch>.json"

"$REVIEW_LEASE" resume \
  --lease <LEASE_PATH> \
  --attachment "$LATE_ATTACHMENT_PATH" \
  --release-receipt <absolute-causal-release-receipt> \
  --reason late_evidence
```

The fresh attachment must preserve every other original attachment argument.
After the reopened review is clean, use `LATE_TERMINAL_PATH` in the finalize
command below. Receipt publication remains create-only; an existing path is an
authority error, not permission to unlink or overwrite prior evidence.

After each authorized remediation push or merge-based base synchronization,
write the exact task-local `spec-work-review-mutation` receipt for that
mutation and run:

```sh
"$REVIEW_LEASE" advance \
  --lease <LEASE_PATH> \
  --mutation-receipt <absolute-task-local-mutation-receipt.json>
```

The receipt must bind the active lease and prior lease digest, unchanged
worktree/common-directory/branch/remote identity, prior and new head/base SHAs,
deterministic changed paths, `authorization: AUTHORIZED`, and `result: PUSHED`.
Advancement rejects rewritten history, remote disagreement, dirty state, stale
authority, and paths outside the frozen set, then invalidates head-bound
semantic-review, CI, mergeability, and terminal-verification evidence.

At final clean, after all current-head evidence passes, run:

```sh
"$REVIEW_LEASE" finalize \
  --lease <LEASE_PATH> \
  --result clean \
  --bundle-root <AUTHORITY_ROOT> \
  --receipt <unique-current-terminal-receipt-path> \
  --observation-epoch <next-positive-epoch>
```

## Compact BUG_FIX.md contract

Use this exact heading set and replace every placeholder:

```markdown
# <Bug title>

## Symptom

<Observed behavior and trigger.>

## Root Cause

<Confirmed owning seam and executable evidence.>

## Generalized Invariant

<Behavior that must hold across the blast radius.>

## Fix

<Smallest complete correction.>

## Changed Paths

- `<repository-relative path>`

## Regression Proof

<Owning-seam test or executable acceptance and result.>

## Verification

<Focused and broader checks with results.>

## UI Result

<PASS, WAIVED, BLOCKED, DEFERRED_TO_PR_FINAL, or not applicable with reason.>

## Tracking

- Issue: #<number>, or `not applicable - fixed entirely in this accepted scope`
- Pull request: #<number>
```

Do not expand this record into an operational ledger. It is the durable bug
explanation and proof summary, not a second workflow state machine.

`## Changed Paths` is an exact final-file contract. List every
repository-relative file returned by
`git diff --name-only --no-renames <final-base> <final-head>`, including this
`BUG_FIX.md` file itself, as one backtick-wrapped bullet. Do not use directory
summaries, absolute paths, deleted-history labels, duplicates, or files outside
the final base-to-head delta. Bullet order is not significant. Missing, stale,
or extra entries fail clean rolling-lease finalization before merge.
