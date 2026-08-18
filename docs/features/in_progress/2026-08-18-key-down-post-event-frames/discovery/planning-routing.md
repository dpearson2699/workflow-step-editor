# Planning Routing

## Harness and routing table

The running harness is Claude Code. Owner routing policy
(`model-routing-and-delegation.md`, Claude Code table):

| Surface | Requested route | Binding |
| --- | --- | --- |
| Primary planning | ChatGPT web Pro | `visible_product_selection` |
| PR-01 implementation | `claude-fable-5`, high | Claude task adapter request |
| PR-01 review | `claude-fable-5`, high | Claude task adapter request |

## Governor predicates

- PR-01 (broker post-event query, worker bounded wait, packet assembly,
  documentation): asynchronous, stateful work — the worker waits against a
  broker that another thread advances, with timing and ordering edge
  cases across `broker.rs`, `worker.rs`, `packets.rs`, and `pipeline.rs`
  — High. No critical predicate; no interacting persistence or
  cross-module concurrency invariants, so xHigh is not selected.
- Review: fixed High per the routing reference.

## Pro planning provenance

- Specification checkpoint: coordinator branch
  `claude/spec-driven-orchestrator-issue-38-072a42` on remote `origin`.
  The exact checkpoint commit and specification digest are bound
  operationally by `work-state record-specification` in `state.json`
  after the push; this receipt cannot self-reference the commit that
  contains it.
- Zero-pass reservation: recorded by the Pro lifecycle bridge in
  `discovery/planning-continuation.json` when the consultation
  initializes.

## Replacement predicates (all tasks)

`unrecoverable_task_runtime`, `unrecoverable_worktree`,
`repository_identity_mismatch`, `pr_identity_unrecoverable`,
`separate_deliverable_user_decision`.
