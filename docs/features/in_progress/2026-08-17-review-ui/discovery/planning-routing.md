# Planning Routing

## Harness and routing table

The running harness is Claude Code. Owner routing policy
(`model-routing-and-delegation.md`, Claude Code table):

| Surface | Requested route | Binding |
| --- | --- | --- |
| Primary planning | ChatGPT web Pro | `visible_product_selection` |
| PR-01 implementation | `claude-fable-5`, high | Claude task adapter request |
| PR-02 implementation | `claude-fable-5`, high | Claude task adapter request |
| PR-03 implementation | `claude-fable-5`, high | Claude task adapter request |
| PR-01/PR-02/PR-03 review | `claude-fable-5`, high | Claude task adapter request |

## Governor predicates

- PR-01 (extended summaries, reveal command, screenshot display path,
  landing page, frontend test harness, UI-proof route): multi-file
  integration across the Rust store/commands and the new React shell, plus
  new verification mechanisms — High. No critical predicate.
- PR-02 (step/rename mutation commands, full detail review view,
  saved-workflow hard deletion): stateful editing flows over persistence,
  the DEC-008 mutation lock and active-workflow guard, and one critical
  predicate (destructive deletion of user data with root-confinement
  validation) — High. The single coordinator mutex is one serialization
  invariant, not interacting concurrency domains, so the xHigh tier is
  not selected.
- PR-03 (record flow, draft save/discard): asynchronous live-channel state
  with significant ordering edge cases and the final-slice UI proof
  obligations — High. No critical predicate.
- Reviews: fixed High per the routing reference.

## Pro planning provenance

- Specification checkpoint: branch `main` on remote `origin`. The exact
  checkpoint commit and specification digest are bound operationally by
  `work-state record-specification` in `state.json` after the push; this
  receipt cannot self-reference the commit that contains it.
- Zero-pass reservation: recorded by the Pro lifecycle bridge in
  `discovery/planning-continuation.json` when the consultation initializes.

## Replacement predicates (all tasks)

`unrecoverable_task_runtime`, `unrecoverable_worktree`,
`repository_identity_mismatch`, `pr_identity_unrecoverable`,
`separate_deliverable_user_decision`.
