# Project Agent Guide

This repository uses a repository-local spec-work lifecycle. Project-specific
facts belong in this file; lifecycle mechanics belong in the routed skills and
workflow core.

## Project goal

Read `docs/PROJECT_GOAL.md` before project planning or implementation. It is the
authoritative product brief for the Workflow Step Editor take-home project.

Treat the four-hour work limit as a hard scope constraint. Prioritize the
must-have requirements and record explicit tradeoffs. Keep the application
local-only and use no external services.

The goal document records submission instructions for human reference. It does
not authorize an agent to send email, submit the project, contact people, or
publish external artifacts. Perform those actions only when the user requests
them.

## Project context discovery

The project goal declares product requirements but does not define a project
configuration schema. Discover implementation facts from the current
repository's manifests, scripts, CI, documentation, installed capabilities,
and any project instructions added to this file later.

If active work reaches a concrete implementation or proof step whose required
command, environment, or capability cannot be discovered, report that exact
blocker at that step. Do not copy project-specific facts into the shared
workflow core.

The starter convention is remote `origin` with default branch `main`. Keep that
convention unless changing the workflow's branch contract is an explicit
harness project.

## Workflow routing

- Use `.agents/skills/spec-driven-feature-orchestrator/SKILL.md` for a new or
  materially expanded capability, or for a full bug fix after debugging routes
  it into spec work.
- Use `.agents/skills/project-debugging/SKILL.md` first for runtime, data-flow,
  persistence, concurrency, performance, integration, or displayed-behavior
  defects. Diagnosis-only work remains read-only.
- A passing `fix_fast` classification stays in the current task and follows
  `project-debugging/references/fast-fix-lane.md`; it skips Pro planning and the
  spec-work bundle while retaining independent review, applicable issue
  ownership, PR publication, and merge delivery.
- Use `.agents/skills/project-ui-verification/SKILL.md` for any user-visible UI
  proof required by an active route, including the fast-fix lane and standalone
  UI verification.
- Use the installed external `gitnexus-pr-review` skill for pull-request review.
  Do not copy or fork it into this repository.
- Use `.agents/skills/improve-codebase-architecture/SKILL.md` only when the
  user explicitly asks for an architecture or deepening survey. It is a
  standalone maintenance entry point that surveys, reports, and grills
  through one picked candidate, then routes execution into the spec-work
  workflow. It never starts a bundle or edits production code; its only
  repository writes are the domain-modeling side effects (glossary terms in
  `CONTEXT.md`, an ADR that passes the three-part test) made with the user
  during the grilling loop.
- Use `.agents/skills/wayfinder/SKILL.md` only when the user explicitly asks
  to chart or work a wayfinder map: a fog-wrapped effort too big for one
  session, planned as decision tickets on GitHub. It is planning only and
  never starts a bundle. At map completion it mints one `enhancement`
  backlog issue per user-confirmed cleared capability and projects them
  onto the Spec Work board, creating the board when absent; bundle
  initialization later adopts each as its owning issue. Route each
  cleared, nameable capability into the spec-work workflow.
- Narrow standalone edits and review-only requests do not start a spec-work
  bundle merely because the workflow is available.

The shared lifecycle owns this repository only. A cross-repository predicate
forces `fix_full` planning but does not authorize mutations elsewhere; represent
the external change as a separately accepted repository owner and dependency.

Once a route is loaded, that route is the authority for its mutations, task
handoffs, planning pass, blockers, UI gate, review, delivery, and completion.
Do not combine parts of separate routes into an improvised lifecycle.

## Engineering and evidence defaults

- Work in small, reversible vertical slices. Reproduce or specify observable
  behavior, make the smallest complete correction, then run focused and
  relevant broader proof.
- Treat test names and claims truthfully. An integration, end-to-end, UI, or
  acceptance test must exercise the unchanged production path implied by its
  name. Fake nondeterministic external boundaries, not product behavior.
- Inspect existing project patterns and dependencies before adding an
  abstraction, state machine, retry, compatibility path, queue, or new package.
- Keep secrets, credentials, signed URLs, tokens, and raw sensitive payloads
  out of commits, work bundles, issues, and receipts.
- Do not hand-edit workflow `state.json`, generated `STATUS.md`, or typed
  receipts. Use the owning deterministic scripts.
- Preserve unrelated user changes in a dirty worktree and never use destructive
  Git operations to make the checkout look clean.

## Durable GitHub issue lifecycle

When an actionable defect is verified outside the current task's accepted
scope, do not leave it only in chat or silently work around it:

1. Read `.github/issue-label-policy.json`, inspect the applicable issue template,
   and compute the canonical fingerprint from `failed_invariant` plus typed
   `affected_surfaces` exactly as its `fingerprint_contract` specifies.
2. Search open and closed issues for the exact canonical marker. Search results
   are discovery evidence only; authoritatively fetch every exact candidate
   before selecting or mutating an owner.
3. Reuse exactly one verified open, non-duplicate exact owner. A user-supplied
   or route-selected issue is the owner only after repository, state, defect
   identity, and collision-free exact-fingerprint verification. A different
   unique open exact owner wins; closed exact matches are history.
4. For the open owner, preserve existing prose, append only missing verified
   evidence, add the canonical marker when absent, and reconcile exactly one
   creation issue-type label plus one severity label. Do not rewrite its title,
   replace its body, or publish a redundant comment.
5. When no open owner exists, create one issue containing reproduction or
   evidence, actual and expected behavior, impact, discovery context, and
   exactly one canonical marker. Link closed history when the observation is a
   new regression after a completed fix.
6. After every mutation, repeat all-state marker discovery and authoritative
   direct fetches. Record the policy's required receipt fields in the task or
   slice handoff.
7. Once the accepted correction is durably effective in the authoritative
   owning system, append missing completion evidence, reconcile labels, and
   close the verified owner without asking for separate closure consent. A
   local, uncommitted, inactive, or unverified correction is not completion.

Create new issues proactively without a separate approval prompt, then continue
the current task unless the defect blocks it. Do not create issues for
speculation, transient environment failures, exact duplicates, secrets or
unsafe public security disclosures, or defects already being fixed inside the
current accepted scope. Multiple open exact owners, a selected duplicate, or
failed authoritative verification fail closed. This authority does not permit
unrelated title changes, comments, reopening, or mutation of a non-owner.

Use `bug` for defects in product behavior and `harness` for defects in the
agent harness, agent instructions, skills, or workflow infrastructure. Choose
the remaining types and P1/P2/P3 severity from the repository policy.

## Pull-request delivery

- Render every non-comment field in `.github/pull_request_template.md` before
  opening a pull request. Use a closing keyword only when that PR completely
  satisfies the linked issue; otherwise use a non-closing reference.
- Implementation and exact-head review are distinct harness-native worktree
  tasks. The implementation owner does not review or merge its own pull
  request.
- The review task receives the exact label-policy path and digest, runs the
  external `gitnexus-pr-review` owner, publishes or reuses required out-of-scope
  issues under that policy, and returns verified follow-up publication
  evidence. The spec-work root records that evidence rather than repeating the
  publication.
- After an exact-head CLEAN result, the root follows the workflow's single
  default-branch freshness check, merge-based synchronization when needed, and
  merge-commit delivery path. Do not ask for a second merge decision already
  granted by the active route.
- Completion requires authoritative default-branch publication, current review
  and verification evidence, quiescent task lineages, satisfied or explicitly
  waived acceptance, and any applicable UI result. An open PR, local commit,
  build, or prose claim is not completion.

Bounded repair, retry, resumption, callback reconciliation, branch-ownership
correction, and base synchronization needed to finish the same accepted
deliverable remain within route authority. Stop for changed user intent, scope
expansion, destructive action, security/privacy risk, identity ambiguity, or a
separate deliverable.
