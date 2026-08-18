# Prompts and human control points

The brief invites a `/prompts` directory with example prompts. This project
was not built with chat one-liners, so the honest content of this folder is
a map of where the human steered and where the verbatim record lives.

## Mode of work

I operated above the loop. The agents (Claude Code as orchestrator and
implementer, ChatGPT Pro as planner, Codex as consensus counterparty,
GitNexus as reviewer) ran the repository-local harness under
[`.agents/`](../.agents/). My inputs were:

1. Charting the wayfinder map and answering its grilling tickets.
2. Answering interview and decision questions inside each capability.
3. Reading plans and pull requests.
4. Running the real-recording acceptance gates on the signed build and
   giving a verdict.
5. Making the product and architecture calls when the agents presented
   options.

Four hours of that interaction was the budget I held myself to. The agents
ran for much longer, across three calendar days.

## The verbatim record

| Control point | Where the exchange is recorded |
| --- | --- |
| Map charter and grilling answers | Issues [#1](https://github.com/dpearson2699/workflow-step-editor/issues/1), [#6](https://github.com/dpearson2699/workflow-step-editor/issues/6), [#7](https://github.com/dpearson2699/workflow-step-editor/issues/7), [#9](https://github.com/dpearson2699/workflow-step-editor/issues/9), [#10](https://github.com/dpearson2699/workflow-step-editor/issues/10), [#11](https://github.com/dpearson2699/workflow-step-editor/issues/11) |
| Research questions and findings | Issues [#2](https://github.com/dpearson2699/workflow-step-editor/issues/2)–[#5](https://github.com/dpearson2699/workflow-step-editor/issues/5) |
| UI prototype pick (variant D) | Issue [#8](https://github.com/dpearson2699/workflow-step-editor/issues/8), branch `prototype/map-1-8` |
| Capability interviews (questions asked, answers given) | `docs/features/completed/*/INTERVIEW.md` |
| Decisions and rejected alternatives | `docs/features/completed/*/DECISIONS.md`, [`docs/adr/`](../docs/adr/) |
| ChatGPT Pro planning prompts and responses, verbatim | `docs/features/completed/*/discovery/pro-lifecycle-evidence/*.md` |
| Codex consensus rounds and root arbitration | `docs/features/completed/*/review/plan-consensus-log.md` |
| Real-recording gate runs and verdicts | `docs/features/completed/2026-08-17-capture-pipeline-and-backend-foundation/review/proven-gate-run.md`; `review/timing-gate-run.md` on branch `claude/spec-driven-orchestrator-issue-38-072a42` |
| Review findings turned into issues | Every open [bug](https://github.com/dpearson2699/workflow-step-editor/issues?q=is%3Aissue+label%3Abug) issue carries its PR of origin, head SHA, and finding id |

## Representative human inputs

The entry prompt for a capability is one line:

```
/spec-driven-feature-orchestrator issue 38
```

Everything after that line is questions from the harness and answers from
me. Examples from the key-down timing capability (issue #38), in order:

- Q-001 (how a key-down step should pick its frame): "Bounded wait, then
  pre-event frame."
- Q-002 (what proves acceptance): "Yes, user-run recording blocks merge."
- Gate run 1, verdict: "precapture for the first keyboard input is still
  capturing pre image rather than post image ... the other keyboard actions
  seem to capture the correct screenshot."
- Q-003 (replacement rule): "Newest in-window frame after a 100 ms settle."
- Q-004 (whether to re-run ChatGPT Pro for the change): "Waive the fresh Pro
  primary."
- Pace steer: "I need this to finish up. This is being pedantic, and it's
  taking way too long. It's just an MVP. Once this next completeness audit
  comes back, let's just implement what it says and move on. No more other
  rounds."
- Gate run 2, verdict: "Check the saved workflow. I saved it as 'Typing
  Still Bugged'. I was typing 'Hello World'. The screenshot for the first
  button press works, but then it seems like the second button press (e)
  captured the text 'Hel'."
- Q-005 (second replacement rule): "Content-aware: first frame that changes
  the focused element."
- Gate run 3, verdict: "Almost fixed ... basically the first character
  doesn't show, but if that's a tradeoff to make every other character be
  in sync then I authorize that tradeoff."

Three of those inputs are gate verdicts from a real recording; each one
changed the shipped rule. That is where the human time went.
