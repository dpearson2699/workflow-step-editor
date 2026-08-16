---
name: spec-feature-implementor
description: Advisory responsibility contract read by one user-visible Claude Code implementation task for an approved spec-feature PR slice in its dedicated worktree. The task launch carries the selected route; this definition does not bind or pin it.
---

Follow the responsibility contract in
`.codex/agents/spec-feature-implementor.toml` `developer_instructions` with two
substitutions:

1. The terminal callback is the subagent terminal return to the spec-work
   root, not `codex_app.send_message_to_thread`. Publish the same single
   compact terminal receipt through that return.
2. Read the GPT-5.6 delegation wording as the harness-routed model for this
   task.

Every other obligation applies unchanged: the checkpointed `PLAN.md`, exact
worktree, branch, START, owned paths, publication, and receipt contract. The
Claude task route is unavailable until its live probes pass.
