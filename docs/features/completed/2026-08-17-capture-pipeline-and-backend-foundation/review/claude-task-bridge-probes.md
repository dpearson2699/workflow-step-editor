# Claude Task Bridge — Live Probe Record

Staged proving per interview Q-004 (user decision, 2026-08-17). Probe run:
one scratch `spec-feature-implementor` task with worktree isolation plus one
cancellation target, live in this Claude Code UI session on 2026-08-17.

| Probe | Result | Evidence |
| --- | --- | --- |
| Native task creation | PASS | `spec-feature-implementor` agent launched via the Agent tool with worktree isolation |
| User-visible task identity | PASS | task identity carried in launch result and completion notification |
| Exact worktree ownership | PASS | isolated worktree under `.claude/worktrees/`, common dir `.git` of this repository |
| Exact START ref and SHA | PASS | worktree HEAD equals required START SHA `30ef40f33bfc7bd1c46f3968d4b117da4a498720` |
| Desired branch ownership | PASS | branch created, listed, and deleted inside the task worktree |
| Direct serial activation | PASS | work started from the initial prompt with no activation message |
| Child-to-parent terminal signal | PASS | typed terminal report delivered in the completion notification |
| Descendant quiescence | PASS | task terminal with zero children |
| Effective route observation | PASS | child reported model `claude-fable-5` |
| Cancellation | PASS | second target task stopped via TaskStop; kill notification received |
| Parallel preparation | DEFERRED | proven fail-closed at first parallel wave (none planned in this bundle) |
| Parent-to-child activation message | DEFERRED | proven fail-closed at first parallel wave (none planned in this bundle) |
| Missed-callback recovery | DEFERRED | proven fail-closed at first real recovery site |
| Same-owner replan | DEFERRED | proven fail-closed at first material-finding replan |
| Exact-head review | DEFERRED | proven fail-closed at PR-01 review attachment |

Clean-checkout rejection precondition observed: `git status --porcelain`
empty in the probe worktree. Deferred probes block their own use sites if
they fail; none authorizes silent degradation.
