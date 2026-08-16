@AGENTS.md

## Claude Code

Harness selection happens when the operator starts the root workflow in one
interactive UI, and the workflow stays in that harness. Under Claude Code,
Delivery uses
`.agents/workflows/spec-work-orchestrator/references/claude-task-bridge.md` as
the task adapter. The Claude task route and the Claude Pro-browser route are
gated: each is unavailable until its live probes pass, and every probe reports
exactly one of `PASS`, `ROUTE_UNAVAILABLE`, or `MANUAL_RESUME_REQUIRED`. Never
start a root session through the shell as orchestration: `claude --bg`,
`claude -p`, `codex exec`, and equivalent wrappers are forbidden as root
sessions, task execution, or delivery orchestration. The single exception is
the bounded read-only cross-model audit call defined by
`.agents/workflows/spec-work-orchestrator/references/plan-consensus.md`:
read-only sandbox, time-capped, no task ownership, and never a substitute for
a harness-native task route. Codex remains usable when a Claude probe fails.
