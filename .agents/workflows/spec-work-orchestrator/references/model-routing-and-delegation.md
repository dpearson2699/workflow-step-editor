# Model Routing and Delegation

Use explicit model routing only for a surface that the running harness can
launch as an
independently configurable user-visible task. Internal Plan delegation and audits
inherit the current task and never claim a requested or effective route.

## Authoritative routing table under Codex

| Independently configurable surface | Route | Binding |
| --- | --- | --- |
| Root coordinator task | Sol; effort chosen by the operator | Observe when available; never gate, reject, or rebind the workflow |
| Primary planning | ChatGPT web Pro | `visible_product_selection` |
| Slice implementation task | `gpt-5.6-sol`, Medium, High, or xHigh | `codex_task_request` |
| Exact-head review task | `gpt-5.6-sol`, High | `codex_task_request` |

These are the only workflow-configurable surfaces. `codex_task_request` proves
request transport, not the effective route. If a requested task route is rejected or
cannot be observed, record the exact blocker; do not silently substitute a route.
The root's operator-selected effort is context, never a conformity gate.

## Owner routing policy under Claude Code

These rows are owner routing policy, not a model-equivalence claim. Do not
state that Fable equals GPT-5.6, and do not state that matching effort labels
imply equal reasoning.

| Independently configurable surface | Route | Binding |
| --- | --- | --- |
| Root coordinator task | Fable; effort chosen by the operator | Observe when available; never gate, reject, or rebind the workflow |
| Primary planning | ChatGPT web Pro | `visible_product_selection` |
| Slice implementation task | Fable 5 (`claude-fable-5`), Medium, High, or xHigh | Claude task adapter request |
| Exact-head review task | Fable 5 (`claude-fable-5`), High | Claude task adapter request |

Select the implementation effort tier with the same governor predicates below.
For each task, record the requested provider, model, and effort, and the
effective provider, model, and effort when observable. The adapter must fail or
disclose a silent substitution or cap, and must not claim an effective value
that the UI does not expose. The Claude task route remains unavailable until
the live probes in `claude-task-bridge.md` pass.

## Planning sequence

1. The root may delegate one or more bounded, non-overlapping, read-only Plan
   questions when fresh context or parallel evidence has independent value. Give each
   delegation exact sources, a stop condition, and a compact evidence receipt. Do not
   name a workflow role or model route for internal delegation.
2. Run the repo-local `chatgpt-pro-feature-planner` primary pass against the pushed,
   digest-bound specification checkpoint. The bridge reserves revision `0`, empty
   `passes`, and a null `parent_cursor`, then claims one receipt-bound primary send intent.
3. Submit exactly once in a verified ChatGPT Pro conversation. Bind the exact
   same-send conversation and generation under the unreconciled receipt revision before
   registering the same-task heartbeat, then consume only fresh typed lifecycle
   results for the current attempt and wait generation. Yield only after
   `heartbeat_registered`; capture only after `generation_completed`. If
   registration is unavailable, report typed `manual_resume_required` without
   polling. A manual re-entry requires `manual_resume_observed` and then a
   current manual completion result. After an admitted wait, keep the persistent
   in-app Browser binding; if its tab binding is missing, stale, or closed,
   discard only that tab binding and reopen the exact receipt-bound canonical
   conversation URL. Verify the committed conversation and generation and
   continue the same generation automatically without manual recovery. This is
   exact-producer reacquisition, not conversation substitution, and tab cleanup
   alone never authorizes manual resume, uncertain-send reconciliation, resend,
   or a user recovery request.
4. After `generation_completed`, capture the full response Markdown from the
   canonical current conversation through the planner skill's native producer-
   payload procedure. It uses a fresh terminal DOM snapshot, exactly one
   `Copy response` control, and one page-realm async CDP `Runtime.evaluate`
   expression with `awaitPromise: true`. Its exact expression invokes the
   documented async page function with JSON-stringified state-key and timeout
   arguments, wraps
   `navigator.clipboard.write()` and `navigator.clipboard.writeText()`, invokes
   the unique response button's existing `HTMLElement.click()` handler, awaits
   the payload while keeping observation active through the full bounded
   capture window for delayed clipboard calls, and owns descriptor/state
   restoration in its own unconditional
   `finally`; pointer-based Playwright, coordinate, or DOM CUA clicks are not
   the capture trigger. Require exactly one non-empty native `text/plain`
   string plus verified cleanup, write its exact UTF-8 bytes, and record
   SHA-256 before calling `capture-response` with the unreconciled receipt revision. If
   cleanup cannot be verified, discard the tab before blocker publication or
   handoff.
   Missing or ambiguous capture or restoration returns
   `BLK-PRO-UNCERTAIN-SEND`; never substitute `innerText`, `textContent`,
   HTML-to-Markdown conversion, `tab.clipboard`, `pbpaste`, the macOS clipboard,
   Chrome, Computer Use, or a human relay. The bridge publishes one immutable
   consumed pass and parent cursor. Classify the response through
   `reserve-invalid-response-successor`, then record proportional results through
   `work-state record-pro-primary`. Requested headings are readability
   guidance, not a generative-prose protocol. Preambles, epilogues, extra or
   mixed-depth headings, tables, and equivalent organization record ordinary
   evidence from the original response and digest while the continuation
   remains producer authority. The root may synthesize that useful content into
   canonical planning artifacts without changing the source evidence. Pass
   `--semantically-unusable` only for a content-based determination that the
   verified response cannot support planning. Before adopting recommendations,
   verify material cited paths and repository claims against the exact checkout
   and source commit; exclude or explicitly annotate unsupported claims.
   Source-access prose does not create retry authority. Non-UTF-8 or empty
   capture and digest, manifest, repository, branch, or source-commit uncertainty
   remain fail-closed.
5. `valid` creates no lifecycle mutation, successor, send intent, send,
   resend, or stage restart. Only `restart_or_fail_closed` reserves one derived
   attempt-2 receipt without a claimed send intent, preserves the invalidated predecessor, and
   uses continuation revision `2` as the commit point. Attempt-2 capture advances
   revision `3`; a second invalid result returns
   `BLK-PRO-INVALID-RESPONSE-EXHAUSTED` and stops.
6. After descendants are quiescent, record the typed blind-completeness requirement
   for the current Pro evidence and plan digest. Enter Delivery directly for
   `not_required_exact_pro_plan`; for any `required_*` value, first record one fresh
   clean audit. On capture failure, record `BLK-PRO-UNCERTAIN-SEND` and stop without
   resend or conversation substitution.

The root never polls the Browser during Pro generation. `Answer now`,
`Stop answering`, and equivalent early-stop controls require exact user
authorization for that action in the current conversation; standing workflow
authority is insufficient.

### Blind-completeness contract

`record-blind-requirement` accepts exactly one current typed decision:
`not_required_exact_pro_plan`, `required_missing_plan_fields`,
`required_unresolved_pro_ambiguity`, `required_multi_slice_reconciliation`, or
`required_user_requested`. Local Plan recovery records
`required_local_plan_recovery` atomically inside `adopt-plan`; callers do not
select it as a substitute for the recovery disposition. The decision is
evidence, not a route or task spawn.
`blind-context` is available only for `required_*` after current captured Pro evidence
validates. `record-blind-completeness` then accepts one current CLEAN audit after
descendant quiescence. Neither operation can bypass missing, stale, or mismatched Pro
evidence. A new Pro response or material applicability change clears the decision and
audit. `record-pro-applicability` remains bound to the active evidence and exact
specification/source transition. Its ordinary typed receipt remains authoritative
when valid. A sole `toSourceCommit` failure may consume one fresh live-remote context
manifest through `--source-observation`; state, immutable active evidence, local HEAD,
remote head, current approved bytes, the recomputed diff, evidence anchors, and the
assessment epoch must prove one candidate. The derived effective receipt changes no
semantic field, preserves the immutable original through correction lineage, and
creates no fresh Pro primary or blind audit.

When the user explicitly stops further Pro sends after current material or
uncertain applicability requires a fresh primary, `adopt-plan` is the only
waiver owner. The coordinator supplies a non-empty user-decision reason and a
fresh CLEAN blind receipt in that atomic transaction. The resulting current
waiver binds the immutable active response plus the exact specification digest,
source commit, canonical Plan digest, and blind receipt digest. Synthesis after
the terminal applicability record may advance that specification/source pair
only inside the paired transaction, to a locally available descendant source
commit that exactly contains the final approved semantic bytes. Missing lineage
or pairs, unavailable or non-descendant sources, ordinary current Pro evidence,
stale candidates, stale receipts, and later Plan drift fail closed; no separate
Boolean, follow-up mutation, or old receipt shape can authorize Delivery.

When retrieval is captureless or a captured primary is explicitly unusable for
source access, `adopt-plan` remains the only alternate authority owner. The
coordinator selects `captureless_pro_recovery` for the exact zero-pass
continuation or `source_access_recovery` for the exact bounded revision-two
invalidated-primary trajectory. Captureless recovery retains the paired user
reason and CLEAN blind receipt. Source-access recovery requires the same CLEAN
blind receipt, while `adopt-plan` records the canonical coordinator exception
without renewed user authorization. The transaction admits no response digest or evidence; it
instead binds the canonical continuation digest, current specification and
committed approved source pack, Plan, task authority, and blind receipt. This is
an explicit local-Plan decision, not automatic resend or semantic inference.

## Sol implementation governor

Choose the least effort whose named predicates cover the slice:

- **Medium:** localized known seams, decision-complete behavior, and focused tests.
- **High:** stateful or asynchronous work, multi-file integration, significant edge
  cases, broader verification, one critical predicate, or verified Medium
  insufficiency.
- **xHigh:** interacting persistence, concurrency, or state invariants; cross-module
  coordination or named missed-verification history; multiple critical predicates;
  or verified High insufficiency.

Critical predicates are security or authentication, destructive financial or
live-data actions, data-loss migrations, and broad cross-module concurrency. Route
each independently configurable implementation task with `model: "gpt-5.6-sol"`
and `thinking: "medium" | "high" | "xhigh"`.

Never route by diff size, spare capacity, novelty, a vague complexity label, or a
desire to consume a stronger model. If the plan is contradictory or incomplete,
return to Plan instead of increasing effort. A verified insufficiency escalation must
name the failed lower-effort attempt and the missing capability or evidence.

## Exact-head review

Launch every independently configurable GitNexus review task with
`model: "gpt-5.6-sol"` and `thinking: "high"`. The review task runs the installed
`gitnexus-pr-review` skill from the exact remote PR head. Internal review workers are
owned by that skill and do not add routes to this table.

## Planning receipts

Create `discovery/planning-routing.md` before the first Plan dispatch. Record pushed
source provenance and the zero-pass reservation digest. Keep task/thread IDs,
heartbeat details, and task-local lifecycle receipt paths out of the bundle. After
capture, record the consumed continuation pass, immutable response evidence, context
manifest, typed blind decision, and any required audit receipt through their owning
deterministic bridges.

The generic state helper owns current specification/source bindings, accepted Pro
evidence, the typed blind requirement and any required completeness audit, and
blockers.

The root carries executable implementation and review authority in the atomic Plan
proposal as structured `slice_id`, `role`, `route`, and
`replacement_predicates` values. `route` is one exact nonempty string and may contain
newlines; it is never reconstructed from Markdown continuation lines. The predicate
array is complete and uses only `unrecoverable_task_runtime`,
`unrecoverable_worktree`, `repository_identity_mismatch`,
`pr_identity_unrecoverable`, and `separate_deliverable_user_decision`. The state
helper canonicalizes that array and derives the entry identity. The primary
specification's roster remains a human-readable semantic mirror, not a second
operational parser authority.

The lifecycle bridge is the only writer of
`discovery/planning-continuation.json`. Before send, the sidecar contains revision
zero, work identity, repository, current consultation reservation, empty pass array,
null parent cursor, and timestamp. Successful attempt-1 capture advances it to
revision one with one consumed response pass and parent cursor. Proportional
response normalization leaves that continuation unchanged and binds its derived
acceptance view in `state.json`. Only `restart_or_fail_closed` advances revision
two with one invalidated predecessor and the derived expected-successor cursor.
Successful attempt-2 capture advances revision three with both immutable passes
and the current consumed cursor. It never contains a conversation URL, task-local
receipt path, task/thread or heartbeat data, prompt text, work phase, or blocker
projections.

`state.json` remains the sole operational phase and blocker authority. Continuation
evidence is accepted only after `work-state` revalidates its canonical identity and
byte-equal response artifact.
