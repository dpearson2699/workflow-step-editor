# Claude Task Bridge

Load this reference only when the running harness is Claude Code and the root
is reserving, launching, observing, steering, replacing, reviewing, or closing
a spec-work slice implementation or review task. It is the Claude Code UI
adapter, not a portable subagent contract. It is a companion to
`codex-task-bridge.md`, never a conditional layer inside it.

## Route availability

This route is DISABLED-UNTIL-PROVEN. It is unavailable until every probe in
the probe list below passes in a live Claude Code UI run. Each probe reports
exactly one closed result: `PASS`, `ROUTE_UNAVAILABLE`, or
`MANUAL_RESUME_REQUIRED`. An ambiguous warning is not a result. While any
probe is unproven or failing, record the exact failing probe, mark the Claude
task route unavailable, and stop; Codex remains the usable route.

## Operator contract

- The operator starts the root workflow manually in Claude Code UI. That start
  selects this adapter for the whole workflow.
- Do not transfer an active workflow between Codex and Claude Code.
- Never start a root session through the shell as orchestration: `claude
  --bg`, `claude -p`, `codex exec`, and any wrapper that starts a new Codex or
  Claude root session are forbidden. Repository commands inside an active UI
  task remain permitted.
- The operator is not a task-message relay. The operator supplies only
  decisions that require human authority.

## Authority and shared bridge

The root remains the only coordinator, and the `work-state` program remains
the only roster writer. This adapter drives the same deterministic scripts as
the Codex bridge and adds no parallel state surface:

```sh
WORK_STATE=.agents/workflows/spec-work-orchestrator/scripts/work-state
WORK_GIT_BINDING=.agents/workflows/spec-work-orchestrator/scripts/task-git-binding
REVIEW_LEASE=.agents/workflows/spec-work-orchestrator/scripts/review-lease
```

Every mutating slice and every exact-head review runs in a distinct
user-visible native Claude Code task with its own worktree. Root mutation,
same-directory substitution, or a hidden process imitating the other harness
is invalid. Reservation, activation, terminal, release, and quiescence facts
flow through the same `work-state` and Git-binding commands the Codex bridge
uses.

## Exact START requirement

START stays the sole checkout authority: an existing ref plus its exact
observed SHA, both proven before task activation. The proof must cover these
start forms:

1. A fresh slice from the recorded required base.
2. A resumed implementation from the retained pull-request head.
3. An exact-head review from the recorded remote pull-request head.
4. A same-task replan using the retained owner.

Never weaken START to the default branch or to the root's current coordinator
HEAD. Never use a headless background session as a workaround. If the native
UI cannot meet START for the required form, the probe result is
`ROUTE_UNAVAILABLE` and the route stays unavailable.

## Serial and parallel launch

- Launch every implementation task as the native custom subagent
  `spec-feature-implementor` through the Agent tool with worktree isolation. Do not
  substitute `general-purpose`. If the named agent is unavailable, report
  `ROUTE_UNAVAILABLE`.
- A serial implementation launches direct-active in its initial task prompt.
  There is no preparation callback or activation message.
- `PREPARE_ONLY` then `ACTIVATE` apply only to an atomic parallel wave of at
  least two independent implementation tasks. Do not impose `PREPARE_ONLY` on
  a serial Claude task.
- If native Claude messaging cannot activate every prepared task in the wave,
  fail the whole wave: stop notified tasks, persist a blocker, and wait for
  observed quiescence. Do not emulate activation by starting shell sessions.
- Live probe fact (2026-08-13): the harness removes an unchanged task
  worktree across an idle gap, and a locked worktree marker does not prevent
  removal. A prepared or resumed task therefore re-creates or re-verifies its
  exact START checkout at every activation before any other action. Never
  assume worktree persistence between turns.

## Input contract

The task adapter must receive:

- Work ID, slice ID, task role, and attempt
- Launch marker
- Checkpoint SHA, plan path, and plan digest
- Desired branch
- START ref and START SHA
- Optional required-base ref and SHA
- Owned paths
- Requested model policy and requested effort policy

## Output contract

The adapter must return or expose:

- Native task identity and native task status
- Worktree path and Git common directory
- Observed branch and observed START SHA
- Requested route values, and effective route values when observable
- Attention signal and terminal signal
- Exact terminal receipt
- Descendant quiescence state

## Failure contract

Reject before activation on any of:

- Wrong repository or wrong Git common directory
- Wrong START SHA or wrong branch
- Dirty initial checkout
- Duplicate worktree ownership
- Missing plan digest
- Missing native task identity
- Unproved messaging for a parallel wave
- Unproved recovery wake
- Unavailable requested route

## Callback policy

`work-state` remains the state authority. UI notifications and hooks are wake
signals, never terminal evidence. Use `SubagentStop` for ordinary native
subagent completion. Use `TaskCompleted` only for a real agent-team task
transition; do not map every completion to it. Prove parent-to-child messaging
before any `PREPARE_ONLY` use, prove child-to-parent terminal delivery, and
re-observe exact task identities before each state mutation. A missed callback
uses the shared parent-owned recovery reconciliation, not polling.

## Probe list

Each probe reports `PASS`, `ROUTE_UNAVAILABLE`, or `MANUAL_RESUME_REQUIRED`:

- Native task creation
- User-visible task identity
- Exact worktree ownership
- Exact START ref and SHA
- Desired branch ownership
- Direct serial activation
- Parallel preparation
- Parent-to-child activation message
- Child-to-parent terminal signal
- Missed-callback recovery
- Cancellation
- Descendant quiescence
- Effective route observation
- Same-owner replan
- Exact-head review

Model and effort routing for this adapter is owner policy in
`model-routing-and-delegation.md`. Record requested provider, model, and
effort for each task, record effective values when observable, and disclose
any substitution or cap.
