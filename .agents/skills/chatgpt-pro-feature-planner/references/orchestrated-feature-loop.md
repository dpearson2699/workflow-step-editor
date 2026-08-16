# Orchestrated spec-work primary

Read this reference when `caller_mode` is `spec_workflow`. Standing authority
covers one digest-bound attempt-1 ChatGPT Pro submission and exact completed-
response capture from that canonical conversation. Only deterministic
invalid-response recovery may derive one separately claimed attempt-2
submission; no third attempt is authorized. Under Claude Code, this same
lifecycle runs through Claude Code's native in-app browser control only, and
that route is unavailable until one complete live planning pass proves the
skill's dependency-boundary capabilities.

## Deterministic bridge

~~~sh
PRO_LIFECYCLE=.agents/skills/chatgpt-pro-feature-planner/scripts/pro-lifecycle
python3 "$PRO_LIFECYCLE" --help
~~~

The receipt schema is `schemas/pro-lifecycle.schema.json`; the parent
continuation schema is `schemas/planning-continuation.schema.json`. The bridge
locks mutable directories, atomically replaces files, derives SHA-256 digests
from canonical bytes, and emits JSON only on stdout.

Exit codes:

- 2: invalid command or path;
- 3: compare-and-swap conflict;
- 4: integrity mismatch;
- 5: missing authority.

Reload durable state after any failure. Never guess a transition or edit a
receipt by hand. Every mutating command uses the current `--expected-revision`
CAS value.

| Command | Purpose |
| --- | --- |
| `init` | Create one task-local receipt and zero-pass reservation, or replace the same-locator reconciled unbound attempt-2 reservation after explicit blocker clearance. |
| `claim-send` | CAS prepared state to one receipt-bound primary send intent and allow the existing Browser send. |
| `bind-conversation` | Commit one same-send canonical conversation/generation observation under the unreconciled receipt revision before waiting. |
| `record-wait-result` | Consume one fresh typed result for the current task, attempt, bound conversation generation, and wait generation. |
| `capture-response` | Consume the unreconciled send revision, validate the already-bound conversation, and publish exact response evidence. |
| `recover-captured-response` | Publish one exact same-task native capture obtained after an admitted heartbeat was reconciled and its canonical uncertain-send blocker was recorded. |
| `reserve-invalid-response-successor` | Classify the captured attempt-1 response; return it to parent admission for proportional recovery, or invalidate only an unrecoverable candidate and prepare its one derived successor without a claimed send intent. |
| `commit-send` | Close a sent prompt when response identity cannot be captured. |
| `reconcile-wake` | Reconcile an uncertain primary send intent without Browser authority. |
| `show` | Print the authoritative receipt. |
| `validate` | Validate the receipt and optional continuation. |
| `validate-continuation` | Validate the bounded zero-pass, consumed, reserved-successor, or completed-successor continuation. |
| `normalize-conversation-url` | Stateless canonical URL syntax check. |

## State and trust boundaries

The task-local receipt owns the work/source/spec/prompt identity, revision CAS,
primary send intent, submission time and deadline, same-send conversation URL
and opaque generation binding, typed task-local wait generation and consumed-
result digests, invalidation, and captured response digest. The bundle owns phase,
blockers, and recorded planning evidence. The continuation owns the initial
reservation, the current consumed pass, or one invalidated predecessor plus its
current bounded successor.

Receipt state, revision CAS, prompt intent, deadlines, and same-task Browser
observations provide weak-form supervised execution. They do not create a
coordinator-only or model-inaccessible capability boundary.

Browser DOM, visible URL, title, prompt text, recent-chat order, clipboard,
screenshots, and copied text are observations and never authenticate their own
origin. The current Browser API exposes same-send CDP observations but no
attested receipt, signature, or provenance credential consumable by this
bridge. Commit the same-task projection of the exact successful send with
`bind-conversation`; do not call that JSON field validation a trust boundary.
Later capture observations gain authority only after `record-wait-result` and
only when they match that binding, a current completed wait result, the
unreconciled receipt revision, and all durable identities.

## Lifecycle

1. `init` verifies the canonical bundle and creates revision `0`, `prepared`,
   plus a matching zero-pass continuation.
2. `claim-send` reopens the exact prompt, checks its digest, records submission
   time and a 5,400-second deadline, advances the receipt revision, and returns
   `browser_send_allowed: true` without creating a bearer or sidecar.
3. The caller captures the Browser CDP cursor, submits those exact prompt bytes
   once in a fresh visible Pro conversation, then reads the same tab's
   post-cursor events and current conversation state. It writes one typed
   `pro-lifecycle-conversation-binding` result with exact work, consultation,
   attempt, task, prompt-intent, canonical conversation, opaque
   generation, and observation identities. The caller commits it with
   `bind-conversation` under the unreconciled receipt revision before any heartbeat registration,
   manual-resume state, or yield. Raw `/c/WEB:...` remains rejected unless the
   same-send observation resolves it to the canonical `/c/<id>` result.
4. The same task writes one typed wait result with work ID, consultation ID,
   attempt ordinal, wait generation, task/thread ID, producer kind, the committed
   conversation binding digest and generation ID, observation time, and result.
   The initial observation time must not predate the committed binding time.
   A fresh `heartbeat_registered` result authorizes a yield;
   `generation_running` requires another generation; `generation_completed`
   authorizes capture. Results expire after 20 minutes and are consumable once.
5. If heartbeat capability is unavailable, consume
   `manual_resume_required` and report it without yielding or claiming a future
   wake. Manual re-entry requires `manual_resume_observed` from the same task,
   followed by a current manual `generation_completed` result. This state is
   only for heartbeat-registration failure; cleanup of an already-admitted
   Browser tab does not enter manual resume.
6. Keep the persistent in-app Browser binding across waits. When the tab binding
   is missing, stale, or closed, including an empty tab list, discard only that
   tab binding and reopen a fresh tab at the exact receipt-bound canonical
   conversation URL. Verify the committed conversation and generation, then
   continue the same generation. Reopening the exact producer is not
   conversation substitution and is automatic without manual recovery. Tab
   cleanup alone never authorizes `manual_resume_required`, `reconcile-wake`,
   `BLK-PRO-UNCERTAIN-SEND`, resend, or a user recovery request.
7. After the typed completed result, the caller follows the planner skill's
   native Markdown capture procedure without navigating or sending again. From
   a fresh terminal DOM snapshot, it resolves exactly one `Copy response`
   control, then runs one page-realm async CDP `Runtime.evaluate` expression
   with `awaitPromise: true`. Its exact expression invokes the documented async
   page function with JSON-stringified state-key and timeout arguments, wraps
   `navigator.clipboard.write()` and `navigator.clipboard.writeText()`, invokes
   the unique response button's existing `HTMLElement.click()` handler, awaits
   the native payload, keeps observation active through the full bounded
   capture window for any delayed clipboard call, and owns restoration in its
   own
   unconditional `finally`. Pointer-based Playwright, coordinate, and DOM CUA
   clicks are not the capture trigger. The caller requires exactly one native
   non-empty `text/plain` payload plus verified cleanup, writes that exact
   string as UTF-8 bytes, and records SHA-256. The same task writes one typed
   `pro-lifecycle-capture-result` that binds those response bytes to the exact
   canonical conversation, committed binding digest, observed generation, task,
   attempt, and observation time. If cleanup cannot be verified, the tab is
   discarded before blocker publication or handoff.
8. `capture-response` requires the current unreconciled revision, completed
   wait authority, and a typed capture result whose canonical URL, binding
   digest, opaque generation ID, response digest, and task equal the committed
   binding and captured Markdown. That same-task result proves correlation, not
   authenticated Browser provenance. It
   validates all identities, writes immutable response evidence, changes the
   receipt to `response_captured`, and advances the continuation to revision
   `1` with one consumed pass and `resolve_material_findings` cursor.
   The command takes the captured identity through `--capture-result`.
   If `reconcile-wake` instead reconciled the send intent from `wait_admitted` or
   `response_ready` and the parent recorded `BLK-PRO-UNCERTAIN-SEND`, a later
   exact native capture from
   that same task, conversation binding, and generation may use
   `recover-captured-response`. It requires the unchanged zero-pass reservation,
   exact current source/specification identity, no response on this receipt,
   and a capture
   observation later than both reconciliation and blocker authority but within
   the original send deadline. An authorized fresh-primary replan may retain
   the prior stale active response in bundle evidence. When completion was not
   already recorded, the bridge derives the ordinary typed manual completion
   record from that exact capture identity; otherwise it preserves the existing
   completed wait. It clears
   only that blocker through the canonical work-state owner, publishes the same
   immutable consumed pass and parent cursor as `capture-response`, and returns
   `parent_action: record_pro_primary`. It has no Browser, send, resend,
   successor, or new-attempt authority.
9. The root calls `reserve-invalid-response-successor` as the public recovery
   classifier, then records the response and manifest through
   `record-pro-primary` when the command returns
   `parent_action: record_pro_primary`. `work-state` independently revalidates
   the continuation identity and exact original response bytes before accepting
   evidence or Delivery. Requested headings are readability guidance, not an
   admission schema. Preambles, epilogues, extra or mixed-depth headings,
   tables, and equivalent organization remain semantically `valid` when the
   root judges the immutable response useful for planning; they record ordinary
   evidence from the original response artifact and digest without correction
   state. Source-access prose remains planning input. The root verifies
   material claims against the exact checkout before Plan adoption.
10. A semantically `valid` response cannot mutate the
   receipt or continuation, reserve a successor or send intent, send or resend, or
   restart a stage. The root passes `--semantically-unusable` only for a
   content-based determination that the captured response cannot support
   planning; presentation differences never authorize it. Only
   `restart_or_fail_closed` preserves attempt 1 as
   invalidated evidence, derives and prepares attempt 2 without a claimed send intent, and
   advances the continuation to revision `2` as the commit point. Attempt-2
   capture advances it to revision `3` with both immutable passes. A second
   invalid response persists terminal invalidation authority, returns
   `BLK-PRO-INVALID-RESPONSE-EXHAUSTED`, and creates no third attempt.
   Repeating classification cannot admit that same immutable response.
11. A `generation_completed` heartbeat ends waiting, not the spec-work
    workflow. In the same root trajectory, reload canonical state and the
    continuation cursor, consume current terminal evidence at most once, and
    execute the public command's `parent_action`. `claim_successor_send` claims
    and begins the prepared attempt 2; `record_pro_primary` records valid
    evidence and continues Plan through adoption, blind completeness, and
    Delivery. Reserve any required heartbeat XML for the final heartbeat
    envelope only after a prepared successor records a fresh
    `heartbeat_registered` result that authorizes yielding, or after reaching a
    genuine user, blocker, manual-resume, or workflow-terminal boundary. Never
    describe nonterminal canonical state as complete. Stale and duplicate
    terminal evidence are idempotent no-ops only for evidence consumption; the
    root still executes any pending canonical `parent_action` or continuation
    cursor after reloading state.
    When retrieval leaves no selectable response, this planner records only the
    canonical continuation. For a captureless revision-`0` trajectory, the root
    may use the existing paired `adopt-plan` waiver with typed
    `captureless_pro_recovery` only after explicit user authorization. For the
    exact revision-`2` invalidated-primary trajectory bound to the proposal
    source and specification, the root may instead select
    `source_access_recovery` under standing coordinator authority. That path
    requires a fresh CLEAN blind receipt, records the canonical coordinator
    exception without a renewed user reason, records
    `required_local_plan_recovery`, and records no active response or evidence.
    Neither path authorizes this planner to resend, reinterpret the
    trajectory, or create another state owner.
12. If a later typed applicability receipt fails strictly and only on
    `toSourceCommit`, produce a fresh
    `chatgpt-pro-feature-planner/context-manifest` outside the state lock with
    live remote verification. `record-pro-applicability --source-observation`
    preserves the original receipt, changes only that target in an immutable
    derived receipt, and binds both through a canonical
    `reconcile_authoritative_state` correction receipt. Local HEAD, the live
    remote branch head, active evidence identity, current approved bytes,
    recomputed changed paths, evidence anchors, and the assessment epoch must
    prove one commit. Any disagreement remains fail-closed without a fresh Pro
    primary or blind audit.

## Failure behavior

Malformed, missing, ambiguous, stale, replayed, or
mismatched conversation bindings; unresolved `WEB:` references; reconciled or
stale send revisions or wait results; wrong
task/attempt/generation/binding; substituted conversation/prompt/bundle/spec/
source identity; empty or non-UTF-8 response files; conflicting
orphan receipts, conflicting immutable evidence, duplicate successor
reservations, and noncanonical continuation paths fail without publishing a
consumable pass.

When exact response identity is unavailable after send, `commit-send
--identity-unavailable` or `reconcile-wake` reconciles the current send intent
and returns `BLK-PRO-UNCERTAIN-SEND`. No response evidence is
published. The parent may clear that blocker only as an explicit workflow
recovery decision before starting a fresh Pro attempt. If the closed receipt is
the reserved attempt 2 and has no committed submission, conversation, wait, or
response identity, `init` may replace it at the same receipt locator with one
fresh prepared attempt 2 and retarget the predecessor's successor pointer. The
parent must keep the blocker whenever a Browser submission may have occurred.
The predecessor's response evidence, pass identity, invalidation reason, and
validation digest remain unchanged. An unreconciled send intent, uncleared blocker, bound or
committed conversation, wait or response evidence, completed successor, or
terminal invalid successor remains fail-closed; none can authorize attempt 3.

The sole committed-submission exception is `recover-captured-response`: the
same task may supply the exact typed native capture from the receipt-bound
conversation and generation after `reconcile-wake` closed an admitted or
completed heartbeat wait. The command rejects an unreconciled send intent, missing canonical
blocker, substituted,
pre-authority, or post-deadline capture, changed planning identity, prior
response, or non-zero-pass continuation without mutation. It never discovers a
conversation, sends a prompt, or creates a replacement attempt.

Native capture also returns `BLK-PRO-UNCERTAIN-SEND` when CDP or cleanup is
unavailable, `Copy response` is missing or ambiguous, the clipboard call is
missing or ambiguous, or exact non-empty `text/plain` bytes cannot be obtained.
It never substitutes `innerText`, `textContent`, HTML-to-Markdown conversion,
`tab.clipboard`, `pbpaste`, the macOS clipboard, Chrome, Computer Use, or a
human relay. An unverified-cleanup failure discards the tab before publication
or handoff, so a live instrumented realm never survives the attempt.

Browser must remain idle between typed wait results. Do not poll the conversation
from the root. Do not press `Answer now`, `Stop answering`, or an equivalent
generation-terminating control unless the user explicitly authorizes that exact
action for the current conversation.

## Safety summary

- ChatGPT Pro remains mandatory.
- Submit at most once per claimed receipt.
- Bind only the same-task observation from that exact successful send, before
  waiting.
- Yield only after a current same-task `heartbeat_registered` result.
- Capture only the completed response from that submitted conversation.
- Treat receipt revision CAS as supervised workflow correlation, not a
  coordinator-only or model-inaccessible security boundary.
- Validate the response again at the parent state boundary.
- Never substitute a conversation, resend automatically, or handwrite evidence.
- Preserve every original response; reserve at most one derived successor and
  only for `restart_or_fail_closed`.
