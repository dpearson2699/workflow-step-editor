# Debugging Capability Routing

Use this reference when the symptom crosses layers or the next evidence source
is unclear. Select capabilities by the failed boundary, then discover the
repository's existing evidence route from instructions, manifests, scripts,
CI, documentation, installed tools, and live system access already in scope. If
a required capability is unavailable at its use site, stop before mutation and
name that concrete blocker.

| Symptom or boundary | Evidence capability | Discover at use time |
| --- | --- | --- |
| Crash, exception, hang, malformed runtime value | Runtime/process | Debugger, trace, logs, and reproduction command |
| Stale display, interaction, navigation, state propagation | UI/state | Launch path, automation/proof route, state observation |
| Missing, duplicate, corrupt, or stale saved records | Persistence | Store inspector, migration/schema authority, integration checks |
| Request failure, timeout, auth/context error | Backend/network | Request and server logs, local/emulated run path, contract docs |
| Provider mismatch or changing payload | External provider | Live read-only query, authoritative docs, response capture seam |
| CPU, latency, I/O, or energy regression | Performance | Profiler and controlled representative flow |
| Growth, retention, or ownership symptom | Memory | Heap/ownership capture and matched lifetime experiment |
| Unknown file/symbol, shared pattern, possible duplicate seam | Code intelligence | Semantic query, callers/impact, exact-text boundary check |

## Routing rules

1. Begin at the first boundary where observed behavior diverges from the
   expected contract; do not begin and end at the visible symptom by default.
2. For cross-layer behavior, trace input through every unchanged production
   seam until the value or state first becomes wrong.
3. Use live external data during research when a provider's current shape or
   state affects the diagnosis. Capture a safe, minimal fixture only after the
   actual contract is observed.
4. Use framework and service documentation to interpret evidence, not to
   replace it.
5. Profile only a bounded, repeatable interval and compare like-for-like runs.
6. For memory symptoms, define the expected lifetime before inspecting owners.
7. After locating a symbol owner, inspect upstream and downstream blast radius
   before editing.
8. Put the regression at the public/service boundary that owns the invariant.

## Fail-closed conditions

Do not edit when the runtime or provider needed to prove the route is
unavailable, no truthful evidence route can be discovered for the relevant
boundary, the code-intelligence result is stale or explicitly partial without
targeted verification, or the observed evidence identifies multiple unresolved
owning seams. Record the blocker or choose the closest executable trace only
when project rules explicitly allow that fallback.
