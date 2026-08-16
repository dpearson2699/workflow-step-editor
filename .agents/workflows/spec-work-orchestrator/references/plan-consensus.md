# Cross-Model Plan Consensus

A bounded adversarial loop between the root's provider and the rival
provider over the candidate plan, run during Plan after
`record-pro-primary` and cited-path verification, before `adopt-plan`. The
counterparty attacks; the root arbitrates; the argument converges or
deadlocks visibly. Adapted from chaseai-yt/crucible's Phase 2; every
command below was live-verified on 2026-08-14 (codex-cli 0.147.0, Claude
Code CLI, this machine).

This is the single carve-out from the project rule against shell-started
sessions: the audit call is read-only, time-capped, owns no task, and is
never a root session or a task route. The blind-completeness pass is
unchanged — it checks completeness; this loop checks correctness.

## Counterparty by harness

| Root harness | Counterparty invocation |
|---|---|
| Claude Code | `codex exec -m gpt-5.6-sol -c model_reasoning_effort="xhigh" -s read-only` |
| Codex | `claude -p --safe-mode --model claude-fable-5 --effort xhigh --tools "Read,Grep,Glob" --session-id <preallocated-uuid>` |

The Claude counterparty is read-only through two flags together: `--tools`
restricts the model's tool names (verified live 2026-08-15: a write attempt
under the allowlist is blocked), and `--safe-mode` starts without project
hooks, plugins, and customizations, so no hook can write on the model's
behalf. Pass both on every initial and `--resume` call. Preallocate the
session id with `--session-id <uuid>` (generate it with `uuidgen`) so the
id is known before the call and a timeout can still be resumed; Claude
prints its JSON result only at completion, so never rely on capturing the
id from output.

Model pinning is verified working on this machine's `codex-lb` provider
(`-m gpt-5.6-sol` starts cleanly; the upstream 400-on-pin caveat applied to
`-codex` variants on plain ChatGPT auth). Echo the resolved counterparty
model before round 1 so the operator can veto.

## Verified mechanics (do not "improve" these)

- Run every `codex exec` from the repository root: outside a trusted
  directory it refuses with "Not inside a trusted directory" (observed
  live).
- `< /dev/null` on every `codex exec` and `codex exec resume`: the CLI
  reads stdin in addition to the prompt argument and hangs silently under
  any non-TTY driver without it.
- Pass `--json` on every `codex exec` and `codex exec resume`: the
  `"thread.started"` line that carries `thread_id` appears only in JSONL
  mode. Capture the verdict from the `-o <file>` last-message output; never
  parse the JSONL stream for content — read it only for `thread_id` on
  round 1. Persist the counterparty type and its exact id (`codex thread
  <thread_id>` or `claude session <uuid>`) in the consensus log header the
  moment it is known, so a resumed root can continue the same session.
- `codex exec resume <thread_id>` REJECTS `-s`. Force
  `-c sandbox_mode="read-only"` on every resume — this machine's
  `config.toml` defaults to `danger-full-access`, so an unforced resume
  could write files mid-review. This is the most important safety line in
  the loop.
- Never resume with `--last`; a missing or wrong thread id can silently
  resume the wrong session and look successful. Echo the id into the
  command visibly.
- Claude side: pass a preallocated `--session-id <uuid>` on round 1 and
  resume with `claude -p --resume <uuid> ...` (verified: resumed sessions
  retain prior-round memory).
- Time-cap every audit call at 10 minutes, using the root harness's own
  process controls. Claude Code root: `timeout: 600000` on the Bash tool
  call, and `run_in_background: true` for large plans (observed
  2026-08-15: an xhigh review of a ~2,500-line diff exceeds 10 minutes in
  the foreground, and a foreground kill loses the round's output). Codex
  root: before round 1, probe for a timeout tool (`command -v timeout ||
  command -v gtimeout`); if neither exists (true of stock macOS — verified
  2026-08-15 on this machine), fall back to a shell watchdog that cannot
  outlive its target or hit a reused pid: start the counterparty in its own
  process group (`set -m; <command> & pid=$!`), start the watchdog
  (`( sleep 600; kill -TERM -- -$pid 2>/dev/null; sleep 10; kill -KILL --
  -$pid 2>/dev/null ) & wd=$!`), then `wait $pid; kill $wd 2>/dev/null` —
  cancelling the watchdog the moment the counterparty exits, and escalating
  TERM to KILL on the whole group only when the ceiling actually trips. Run
  it with stdout redirected to the round's output file, in the background
  when the plan is large, and read that file when the process exits.
  Output capture differs by counterparty:
  Codex writes its last message to `-o <file>`; Claude has no `-o`, so
  redirect its stdout (`> <file>`). A tripped ceiling is a failed round:
  resume the SAME thread or session once with a "you were cut off —
  continue and finish" prompt (both retain what they already read); if that
  also trips, stop and surface it. Never restart cold on a timeout — that
  pays the full read again.
- Stderr noise (MCP auth chatter) is cosmetic on both CLIs; judge success
  by the output file plus the thread/session line.

## The loop

Tunables: `MAX_ROUNDS` (default 5). The loop ALWAYS terminates at the cap.

1. The root synthesizes the candidate plan into the bundle's canonical
   planning artifacts as usual, then initializes
   `review/plan-consensus-log.md` in the bundle:
   `# Plan Consensus Log` + counterparty model + `MAX_ROUNDS` + the
   counterparty session id once known. The log
   lives under `review/`, not `discovery/`, deliberately: the bridge
   inventories every `discovery/*.md` as an approved specification source,
   so a log there would change the specification digest on every round and
   invalidate the recorded Pro binding. `review/` is outside that
   inventory; the log is evidence, never specification.
2. Round 1 sends the review prompt below in a fresh counterparty session;
   capture the thread/session id and the verdict file.
3. Each round, append the full critique to the log as
   `## Round <n> — <counterparty>`. Then:
   - `VERDICT: APPROVED` → converged; proceed toward `adopt-plan`.
   - `VERDICT: REVISE` → the root is final arbiter within its authority:
     it accepts what is actually right and rejects the rest, each
     rejection with a logged reason under `### Root arbitration`. Do not
     cave to everything (defeats the cross-model check); do not ignore it
     (defeats the point). The root's arbitration authority covers
     plan-internal corrections only — sequencing, missing verification,
     unverified repository claims, rollback boundaries, simpler equivalent
     designs. A finding that would change accepted intent, scope,
     acceptance, or architecture is not the root's to accept: record it as
     a decision-shaped `Q-*` through the interview reference and let the
     user decide before any artifact changes. Revise the candidate
     planning artifacts, then resume the SAME counterparty session: "The
     plan was revised. Re-review — check whether your prior findings are
     addressed and flag anything new. End with VERDICT: APPROVED or
     VERDICT: REVISE."
   - `VERDICT: REVISE` on round `MAX_ROUNDS` → deadlock; do not start
     another round. Do NOT fake convergence: record each unresolved
     finding plus the root's counter-position in the log, and route each
     one through the interview reference as a decision-shaped question
     for the user. The user breaks the tie.
4. Plan revisions made during the loop precede `adopt-plan`. Plan-only
   revisions carry no specification change and adopt normally. Any
   revision that touches the primary specification, `INTERVIEW.md`,
   `DECISIONS.md`, or `ACCEPTANCE.md` changes the specification digest
   and takes the existing applicability route — including a fresh Pro
   primary when that route classifies it material — before adoption; the
   loop grants no shortcut around it. The log is a semantic evidence
   artifact; nothing about this loop enters `state.json`.

## The review prompt (round 1)

> You are an adversarial reviewer for an implementation plan. Be skeptical
> and specific — your job is to find what breaks, not to be agreeable.
> First read the repository's agent instructions — `AGENTS.md`,
> `CLAUDE.md`, and every lifecycle reference they name for the phase under
> review — so you judge the plan against the contracts it must satisfy
> (your session loads none of them automatically). Then read the candidate
> plan and its bundle artifacts at
> `<bundle>/<primary artifact, slice plans, ACCEPTANCE.md>` and any
> repository files you need; you are read-only. Identify concrete flaws:
> security holes, race conditions, missing edge cases, schema conflicts,
> wrong assumptions, unverified repository claims, scope expansion beyond
> the accepted goal, missing migration or rollback boundaries,
> observability gaps, simpler alternatives. For each, give a one-line
> fix. Do NOT modify any files. End your reply with EXACTLY one line:
> `VERDICT: APPROVED` if the plan is sound enough to implement, or
> `VERDICT: REVISE` if it still has material problems.

## Accepted residual risk: Codex counterparty MCP surface

`-s read-only` sandboxes the filesystem only. Live probes on 2026-08-15
showed the Codex counterparty still holds the account's ChatGPT connector
tools — including GitHub-mutating ones such as create/merge pull request,
delete file, and update ref — and that these are account-level
authorization, not disableable through `-c mcp_servers.*` or
`-c plugins.*` overrides (per-server `enabled=false` removes only
user-configured `mcp_servers`). The operator accepted this residual risk
on 2026-08-15 (option: accept, with the prompt-level read-only instruction
as the guard). Consequences: the review prompt's "Do NOT modify any files"
line and its read-only framing are load-bearing and must never be dropped;
pass `-c mcp_servers.<name>.enabled=false` for every user-configured
server anyway to minimize the surface; and revisit this decision if the
Codex CLI gains a connector-disable flag or a dedicated no-connector
profile becomes available.

## Hard rules

- The counterparty is read-only every round, in both directions. It never
  writes a file and never owns a task.
- The root never edits the immutable Pro response; arbitration edits only
  the root's own canonical planning artifacts.
- The log is append-only and keeps the whole argument — critiques,
  acceptances, and rejections with reasons. A flagged deadlock beats a
  false "approved."
- Audit calls are point invocations under this reference only; the
  project-level ban on shell-started root sessions stays in force for
  everything else.
