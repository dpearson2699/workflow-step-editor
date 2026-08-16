# Root-Cause Orchestration

Use this reference when a debugging symptom is non-trivial, intermittent,
cross-layer, performance- or memory-related, stateful, or potentially another
instance of a previously patched pattern.

## Hub-spike map

Write a compact map before production edits.

- Hub: exact symptom, trigger, expected contract, current state, and observed
  evidence.
- Spokes: plausible owning seams; shared code patterns; call, render, state,
  data, persistence, and network paths; external systems; relevant current
  docs; and every supported route that may share the invariant.
- For each spoke: evidence needed, owning capability/skill, blast-radius check,
  result, and potential regression seam.
- Research receipt: docs and community sources consulted, project skills
  loaded, live-system checks performed, and whether each source confirmed,
  refined, or merely suggested a hypothesis.

Do not treat the visible failing page, command, endpoint, or record as the owner
unless evidence shows that boundary owns the invariant.

## Execution topology

- Use one thread for one likely causal path or dependent evidence sequence.
- Sequence checks when each result gates the next, such as executable trace,
  then graph impact, then regression abstraction.
- Fan out only three or more genuinely independent, read-only lenses with no
  shared mutable state. Children return evidence; the coordinator synthesizes
  and remains the sole writer.

Urgency does not justify parallel writers or skipping a dependent evidence
gate.

## Evidence sources

- Runtime/process: debugger, stack, frame state, logs, or a controlled trace.
- UI/state: production launch/navigation path, state observation, and the
  repository-native proof route discovered at use time.
- Code intelligence: semantic discovery followed by callers, impact, process,
  or bounded graph inspection; verify any declared epistemic boundary.
- Persistence: store inspection, schema/migration state, transaction trace, and
  unchanged production save/load path.
- Backend/network: client request, server trace, response, and error boundary.
- External provider: current read-only API/service query and authoritative
  contract documentation.
- Performance: bounded representative profile with a like-for-like baseline.
- Memory: defined expected lifetime plus ownership capture or matched graphs.
- Tests: failing behavioral proof at the owning observable seam. Source-text
  guards are supplemental only.

Community fixes and documentation can prioritize hypotheses but cannot prove
this repository's owning seam by themselves.

## Implementation gate

Do not edit production code until evidence identifies the owning seam and blast
radius. If the primary mechanism cannot apply, state why and use the closest
executable trace permitted by project rules. If evidence still leaves multiple
owners, route to `fix_full` or remain diagnosis-only.

A local patch is valid only when the local boundary owns the invariant.
Otherwise correct the shared state machine, mapping, loader, service, data
shape, persistence boundary, architectural pattern, or guard responsible for
every reachable instance.

Verification must include the owning-seam regression plus the relevant broader
check for the blast radius: integration, UI proof, backend/service check,
live-provider invariant check, performance comparison, or graph impact check.
