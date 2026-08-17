# Planning Routing

## Harness and routing table

The running harness is Claude Code. Owner routing policy
(`model-routing-and-delegation.md`, Claude Code table):

| Surface | Requested route | Binding |
| --- | --- | --- |
| Primary planning | ChatGPT web Pro | `visible_product_selection` |
| PR-01 implementation | `claude-fable-5`, medium | Claude task adapter request |
| PR-02 implementation | `claude-fable-5`, high | Claude task adapter request |
| PR-03 implementation | `claude-fable-5`, xhigh | Claude task adapter request |
| PR-01/PR-02/PR-03 review | `claude-fable-5`, high | Claude task adapter request |

## Governor predicates

- PR-01 (scaffold, signing, permission commands): localized known seams and
  decision-complete behavior — Medium. No critical predicate.
- PR-02 (KeySemantics, parser, schema v1 store): multi-file integration and
  significant edge cases (key-character mapping, chord titles, JSONL
  append-only layout) — High. No critical predicate.
- PR-03 (capture pipeline, recording lifecycle, dev trigger): interacting
  concurrency and state invariants (dedicated CFRunLoop tap thread, stream
  frame buffer, bounded async shot queue, single-active-recording) plus
  cross-module coordination (tap -> buffer -> AX -> parser -> store ->
  channel) — xHigh.
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
