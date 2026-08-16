---
name: chatgpt-pro-feature-planner
description: Use this skill when a standalone caller or the shared spec-work workflow wants to prepare a bounded ChatGPT web Pro planning request grounded in an exact pushed repository branch plus approved local-only context. In spec-work, it owns the initial authorized Browser send, at most one derived invalid-response successor, completed response capture, immutable continuation evidence, and handoff back to the root. Standalone callers stop before sending.
---

# ChatGPT Pro Spec-Work Planner

Prepare one evidence-bound planning prompt for ChatGPT Pro. In the spec-work
workflow, submit it once per claimed attempt, capture the completed response from
that attempt's canonical conversation, and publish the immutable response
lineage that the root consumes before Delivery. The bounded invalid-response
path may derive attempt 2; no other repeat submission is authorized.

## Invocation contract

Accept:

- `caller_mode: standalone | spec_workflow` (default `standalone`);
- `authorization: confirm_each_send | digest_bound_standing` (default
  `confirm_each_send`).

For `spec_workflow`, require `digest_bound_standing`, work kind, work ID,
canonical work bundle, specification digest, exact source branch and commit,
path inventory, and task-local receipt path. Read
`references/orchestrated-feature-loop.md` before preflight, claim, submission,
capture, or recovery.

Standalone callers prepare the prompt and stop before Browser submission.
Spec-work standing authority permits one exact digest-bound attempt-1
submission and, only after deterministic invalid-response recovery, its one
derived attempt-2 submission. It does not permit manual uploads, arbitrary
ChatGPT messages, caller-selected replacement conversations, follow-on repair
prompts, or a third attempt.

## Dependency boundary

Spec-work submission requires a signed-in ChatGPT session in the running
harness's own in-app Browser. Under Codex, the installed
`browser:control-in-app-browser` skill remains mandatory and exclusive: load
that skill before Browser work and probe the in-app Browser. Under Claude
Code, the planner drives the same send, bind, wait, capture, and cleanup
lifecycle through Claude Code's native in-app browser control; the operator
must not manually relay the response. Two live probe facts (2026-08-13) bind
that route: the composer's Enter key does not submit, so the task submits
exactly once through the visible `Send prompt` control; and that browser
exposes no raw CDP event cursor, so the same-send conversation and generation
binding derives from the same tab's post-send network and page observations.
Each harness may use only its own native
in-app browser. Chrome-profile automation, Computer Use, private network
calls, cookies, stored credentials, and human relay stay forbidden in both.
The Claude route is unavailable until one complete live planning pass proves
conversation binding, generation binding, prompt send, idle waiting, native
Markdown capture, digest stability, required page-realm JavaScript, clipboard
behavior, cleanup, and missed-event recovery.

The deterministic bridge is:

~~~sh
PRO_LIFECYCLE=.agents/skills/chatgpt-pro-feature-planner/scripts/pro-lifecycle
PREPARE_PROMPT=.agents/skills/chatgpt-pro-feature-planner/scripts/prepare_planning_prompt.py
python3 "$PRO_LIFECYCLE" --help
python3 "$PREPARE_PROMPT" --help
~~~

It owns receipt validation, revision CAS, the receipt-bound primary send intent,
typed same-send conversation correlation, task-local wait admission, exact
response evidence, and the bounded continuation lineage. It never accesses
Browser. Do not replace it with prose assertions or handwritten receipt edits.
The current in-app Browser API exposes same-send CDP observations but no
attested Browser receipt, signature, or provenance credential that this bridge
can authenticate. Consequently the bridge validates correlation and integrity,
not the origin of a caller-authored JSON file. Never describe `producer_kind`
or any other artifact field as a trust boundary. If authenticated Browser
provenance is required, the exact capability is unavailable and the run must
stop rather than invent an attestation mechanism.

## Source contract

The spec-work root remains the only semantic writer. This skill may publish the
exact captured Pro response under
`discovery/pro-lifecycle-evidence/`, but it must not edit the primary
specification, interview, decisions, acceptance criteria, slice plans, or
canonical work state.

Build the deterministic context manifest:

~~~sh
python3 .agents/skills/chatgpt-pro-feature-planner/scripts/build_context_manifest.py \
  --repo . \
  --bundle <work-bundle> \
  --exclude-evidence-path <prior-answer-or-repair> \
  --verify-remote \
  --pretty
~~~

Add repeated `--path` arguments for focused sources and exclude every prior Pro
artifact recorded in canonical state. Bind the prompt to the emitted source
inventory and provenance digests.

Keep the emitted context manifest local until it is recorded with the completed
planning evidence. Do not list the manifest itself as GitHub-retrievable
evidence. Prepare the exact send-ready prompt from that manifest:

~~~sh
python3 "$PREPARE_PROMPT" \
  --manifest <local-context-manifest.json> \
  --work-id <work-id> \
  --work-bundle <work-bundle> \
  --specification-digest <specification-digest> \
  --work-goal '<work-goal>' \
  --decision-and-acceptance-anchors '<accepted-decision-and-acceptance-anchors>' \
  --requested-path <initial-evidence-path> \
  --output <exact-planning-prompt>
~~~

Repeat `--requested-path` for every initial repository evidence path requested
by the caller. The preparation command validates the manifest inventory and
provenance digests, rejects any `blocked` route or requested path absent from
that inventory, renders every and only the manifest's `github` routes into the
exact-path list, and embeds each `local_inline_or_upload` path under its explicit
local-only label and manifest base commit. A nonzero result grants no send
authority and must not leave an eligible prompt file. The command's successful
JSON result and output digest identify the only prompt file eligible for the
existing lifecycle claim. Do not hand-fill, append, or rewrite that file.

Route each requested path exactly once:

1. `github`: the clean path exists at local HEAD and that exact HEAD is verified
   on the named remote branch.
2. `local_inline_or_upload`: GitHub cannot see the exact working copy. Label the
   approved content local-only and identify its base commit.
3. `blocked`: source, visibility, sensitivity, or identity is unresolved. Stop.

An untracked path matched by Git ignore rules is always `blocked`; ignored
content is not eligible for local inline or upload admission. A tracked path
retains ordinary tracked/dirty routing even when a later ignore pattern matches
its name.

This skill never pushes a branch or invents remote visibility.

When `record-pro-applicability` rejects only `toSourceCommit`, build one fresh
source observation outside the state mutation lock with the same command,
current bundle exclusions, `--branch <active-evidence-branch>`, and
`--verify-remote`. Save the exact JSON under the bundle and pass it as
`--source-observation`. The state bridge may reconcile only when the manifest's
local HEAD and live verified remote branch head equal checked-out `HEAD`, every
current approved source path/digest and the receipt's diff/anchors reproduce
that commit, and no second candidate exists. This path sends no Pro message and
does not rerun blind completeness.

## Prepare, submit, and capture

1. Re-ground on the goal, accepted decisions, acceptance criteria, and exact
   manifest identity.
2. Run the manifest-bound preparation command above. Confirm its successful
   result identifies the exact prompt file and digest; do not substitute a
   manually filled `assets/planning-prompt.md` or add paths afterward.
3. In spec-work, initialize the task-local receipt and claim only that exact
   prepared prompt:

~~~sh
python3 "$PRO_LIFECYCLE" claim-send \
  --receipt <task-local-receipt> \
  --expected-revision <revision> \
  --kind primary \
  --prompt-file <exact-planning-prompt>
~~~

4. Open a fresh visible `chatgpt.com` thread through the in-app Browser. Select
   and verify Pro. Activate `@GitHub` through its visible structured suggestion
   and verify the non-editable GitHub pill. Append only the exact prompt bytes.
5. Verify repository, branch, commit, paths, local-only labels, and output
   contract. Capture the Browser CDP event cursor, submit exactly once, then
   read the same tab's post-cursor events and current conversation state. From
   that same-send observation, write one task-local
   `pro-lifecycle-conversation-binding` result containing the exact work ID,
   consultation ID, attempt ordinal, Codex task ID, prompt intent digest,
   canonical conversation URL, opaque generation ID, and observation time. A
   URL, title, prompt match, recent-chat lookup, or later
   conversation discovery alone is not sufficient. The JSON projection is not
   an authenticated Browser receipt; the same-task Browser observation is the
   supported operational boundary.
6. Commit that result before registering or yielding any wait:

~~~sh
python3 "$PRO_LIFECYCLE" bind-conversation \
  --receipt <task-local-receipt> \
  --expected-revision <revision> \
  --binding-result <same-send-observation-result>
~~~

   The bridge canonicalizes the supported URL spelling, binds the exact
   conversation plus generation to the consultation, task, prompt intent, and
   current receipt revision, and returns its immutable binding digest. A raw
   `/c/WEB:...` reference is rejected; the same-send observation must resolve it to the
   canonical `/c/<id>` identity in the binding result. Missing, ambiguous,
   stale, replayed, or mismatched binding fails closed without resend or chat
   discovery.
7. Register one same-task 15-minute heartbeat for the bound consultation,
   attempt, conversation generation, and wait generation. Persist its fresh
   typed result only in a task-local file and include the exact
   `conversation_binding_digest` and `conversation_generation_id` returned by
   step 6 before consuming it with `record-wait-result`; its observation time
   must not predate the committed conversation binding. A successful
   `heartbeat_registered` result authorizes yielding; each later
   `generation_running` result returns to the same transition and requires a
   new current generation. A `generation_completed` result authorizes one
   capture. Heartbeat results expire after 20 minutes and are consumable once.
8. If heartbeat registration is unavailable, consume a typed
   `manual_resume_required` capability-probe result and report
   `manual_resume_required`; do not yield, poll, or capture. The same task may
   later consume `manual_resume_observed`, inspect the current conversation
   once, and then consume a current manual `generation_completed` result. Every
   manual result carries the same binding digest and conversation generation.
   This path applies only when heartbeat registration itself is unavailable;
   cleanup of a tab after an admitted heartbeat never selects manual resume.
9. Keep the persistent in-app Browser binding across wakes. If its tab binding
   is missing, stale, or closed, including when the Browser lists zero tabs,
   discard only that disposable tab binding and reopen a fresh tab at the exact
   receipt-bound canonical conversation URL. Verify that it resolves to the
   committed conversation and generation, then continue observing the same
   generation. This is exact-producer reacquisition, not conversation
   substitution, and it happens automatically without manual recovery. Never
   reconcile the send, record `manual_resume_required`, publish
   `BLK-PRO-UNCERTAIN-SEND`, resend, or ask the user to recover solely because
   Browser cleanup removed the tab handle.
10. Stay on that bound conversation until the typed result says generation is
    complete. Follow the [native Markdown capture procedure](references/native-markdown-capture.md)
    to write the assistant's exact native Markdown payload into a task-local
    UTF-8 `.md` file, then read the current canonical
    `https://chatgpt.com/c/<id>` URL. Load that reference only after a current
    typed `generation_completed` result authorizes capture. Do not navigate to a
    different conversation or send another message. Write one same-task typed
    capture result from that exact observation as described below.
11. Commit the completed response through the receipt-bound lifecycle bridge:

~~~sh
python3 "$PRO_LIFECYCLE" capture-response \
  --receipt <task-local-receipt> \
  --expected-revision <revision> \
  --capture-result <same-task-capture-result> \
  --response-file <task-local-captured-markdown> \
  --bundle-state <work-bundle>/state.json \
  --planning-continuation \
    <work-bundle>/discovery/planning-continuation.json
~~~

The bridge requires the canonical URL to equal the already committed
conversation binding and validates the unreconciled receipt revision, exact
work/source/spec/prompt identity, capture-time task, binding, generation,
response-digest and observation correlation, current completed wait result,
response bytes, continuation reservation, and bundle state. It copies the exact
response to an immutable bundle-owned artifact, closes the send intent, and
publishes one consumed continuation pass. The successful result returns
`parent_action: record_pro_primary`.

12. A heartbeat wake that reaches `generation_completed` ends only the current
    wait. In the same root trajectory, reload the receipt and canonical
    continuation cursor, capture and classify when not already consumed, and
    execute the returned `parent_action` before reserving any required
    heartbeat XML for the turn's final heartbeat envelope. A prepared
    `claim_successor_send` starts attempt 2 through these same public commands;
    `record_pro_primary` returns immediately to Plan. A fresh successor
    `heartbeat_registered` result is the next authorized yield boundary.

### Native Markdown capture

When a current typed `generation_completed` result authorizes capture, load
`references/native-markdown-capture.md`. Read that reference completely before
taking any capture action. Follow it exactly on the receipt-bound canonical
conversation; it owns native payload observation, byte preservation, cleanup,
and fail-closed capture behavior. Do not reload, navigate away, send, poll,
substitute another representation, or publish positive evidence unless its
capture and cleanup conditions pass.

Request these H2 headings for readability: `Evidence map`, `Assumptions`,
`Architecture`, `Reviewable delivery slices`,
`Risks and missing evidence`, and `Recommendation`. They are guidance for
generative prose, not an admission protocol. Preambles, epilogues, extra or
mixed-depth headings, tables, and equivalent organization are presentation
choices. The root reads the immutable response and decides whether its
substance can support planning; it may extract, translate, normalize, or
synthesize that content into canonical repository artifacts without changing
the captured source evidence.

`work-state record-pro-primary` independently validates the original bytes,
digest, provenance, manifest binding, repository, branch, and source-commit
identity before admission. It records ordinary accepted evidence from the
original producer artifact and digest, without a derived acceptance view or
presentation-correction state. Source-access prose and additional repository
claims remain planning input for the root to verify against the exact checkout
and source commit before adopting a recommendation.

During the wait, Browser is not a polling mechanism. Never repeatedly inspect
the DOM, URL, clipboard, or conversation. Never press `Answer now`,
`Stop answering`, or any equivalent early-stop control unless the user
explicitly authorizes that exact action for the current conversation. General
workflow standing authority does not authorize interruption.

After capture, run the public recovery classifier before acquiring successor
authority:

~~~sh
python3 "$PRO_LIFECYCLE" reserve-invalid-response-successor \
  --receipt <attempt-1-receipt> \
  --expected-revision <revision> \
  --conversation-url <captured-canonical-url> \
  --prompt-file <exact-planning-prompt> \
  --bundle-state <work-bundle>/state.json \
  --planning-continuation \
    <work-bundle>/discovery/planning-continuation.json \
  --manifest-artifact <bundle-relative-context-manifest> \
  --manifest-artifact-digest <exact-json-sha256> \
  --manifest-digest <provenance-digest>
~~~

For `valid`, the command returns
`parent_action: record_pro_primary` without mutating the receipt or
continuation and without creating a successor, send intent, Browser send,
resend, or stage restart. `record-pro-primary` independently reruns semantic admission
against the original response bytes.

Pass `--semantically-unusable` only after the root has read the immutable
response, verified material cited paths and repository claims against the exact
checkout and source commit, and determined that the supported remainder cannot
support planning. Examples include missing substantive analysis or a response
that does not address the planning job. Exclude or explicitly annotate
unsupported claims in canonical planning artifacts. Omitting the flag classifies
presentation and source-access prose as `valid`. Capture uncertainty remains
`BLK-PRO-UNCERTAIN-SEND`; byte, digest, provenance, manifest, repository,
branch, and source-commit uncertainty remain automatic fail-closed evidence.

Only `restart_or_fail_closed` may reserve the one successor. Its receipt locator
and attempt-2 consultation ID are derived. The command preserves and
invalidates attempt 1, advances the continuation as the commit point, and
returns a prepared attempt-2 receipt with no claimed send intent. Claim, submit, wait,
and capture attempt 2 through the same public commands. If attempt 2 is invalid,
the lifecycle persists that terminal semantic rejection before returning
`BLK-PRO-INVALID-RESPONSE-EXHAUSTED`. Repeated classification of the same
immutable response remains exhausted even when the flag is omitted. Stop the
bundle; never create a third attempt.

## Capture failure

If the response is incomplete, the canonical current conversation URL is
unavailable, or exact Markdown capture cannot be verified, publish no positive
evidence. Reconcile the current send intent with:

~~~sh
python3 "$PRO_LIFECYCLE" commit-send \
  --receipt <task-local-receipt> \
  --expected-revision <revision> \
  --identity-unavailable
~~~

When a later trajectory resumes without the exact expected revision, use
`reconcile-wake` with the canonical bundle state and continuation. Both paths
return `BLK-PRO-UNCERTAIN-SEND`; the parent records the
blocker and stops Browser activity. If that exact attempt had already committed
its conversation and admitted heartbeat, and the same task later captures the
completed native response from the receipt-bound conversation and generation,
recover it without resend:

~~~sh
python3 "$PRO_LIFECYCLE" recover-captured-response \
  --receipt <task-local-receipt> \
  --expected-revision <revision> \
  --capture-result <same-task-capture-result> \
  --response-file <task-local-captured-markdown> \
  --bundle-state <work-bundle>/state.json \
  --planning-continuation \
    <work-bundle>/discovery/planning-continuation.json
~~~

This recovery requires the canonical blocker, a reconciled receipt, unchanged
zero-pass reservation, an admitted or already completed wait, exact
task/conversation/generation/binding/response identity, and
a capture later than reconciliation and blocker authority but within the
original send deadline. An authorized fresh-primary replan may retain its prior
stale active response in canonical evidence. The bridge derives the ordinary
typed manual completion record only when completion was not already recorded;
otherwise it preserves that completed wait. It clears only that
blocker and publishes the ordinary consumed response pass with
`parent_action: record_pro_primary`; it grants no Browser, send, resend,
successor, or new-attempt authority. Any mismatch remains fail closed.

Clearing that blocker for a fresh retry is otherwise a root-owned
workflow recovery decision, never an automatic planner action. After that
explicit clearance, `init` may replace a reserved attempt-2 receipt only at its
existing locator and only when reconciliation left an unbound send intent, no
committed submission reference, bound conversation or generation, wait, or response.
If a Browser submission may have occurred, the parent keeps the blocker and no
replacement authority exists.
The replacement remains attempt 2 and retargets only its successor linkage;
attempt-1 response evidence and pass identity remain unchanged. A live,
bound, committed, captured, or uncleared successor still fails closed and never
authorizes attempt 3.

## Parent handoff

On successful capture or a terminal heartbeat wake, the root reloads canonical
state and follows the persisted continuation cursor in the same root trajectory.
It:

1. reads the immutable response and classifies its contract disposition;
2. executes the returned `parent_action`: claim, submit, wait for, and capture a
   prepared invalid-response successor, or record the existing context manifest
   and valid response through `work-state record-pro-primary`;
3. continues Plan through adoption and the existing blind completeness gate;
   and
4. enters Delivery only after both current evidence gates pass.

Wait-generation completion is not whole-workflow completion. The final
heartbeat envelope is emitted only after a prepared successor records a fresh
`heartbeat_registered` result that authorizes yielding, or after this
continuation reaches a genuine user, blocker, manual-resume, or
workflow-terminal boundary. Stale or duplicate terminal evidence remains an
idempotent no-op for evidence consumption, but the awakened root still executes
any pending canonical `parent_action` or continuation cursor and may not describe
nonterminal canonical state as complete.

If the root invalidates a captured primary specifically because Pro could not
retrieve required source evidence, the exact revision-`2` continuation also
permits the root coordinator to use the existing typed
`source_access_recovery` Plan-adoption exception. The root verifies the exact
local source inventory, specification, and Plan, supplies a fresh CLEAN blind
receipt, and does not claim or send the reserved successor. `adopt-plan`
records the canonical coordinator reason; renewed user authorization is not
required. This grants no synthesis, resend, or state-mutation authority to this
planner.

The planner does not synthesize evidence, edit specifications, implement slices,
or create task topology.

## Gotchas

- GitHub sees the remote branch, not the local worktree.
- Selecting the visible `@GitHub` suggestion creates the app invocation; raw
  text does not.
- Receipt state, revision CAS, prompt intent, deadlines, and same-task Browser
  observations provide weak-form supervised execution. They do not create a
  coordinator-only or model-inaccessible capability boundary.
- URL normalization proves syntax only. The same-send observation plus its
  receipt-revision CAS supplies correlation before waiting; it is not an
  authenticated Browser provenance boundary.
- A completed response is captured once from the submitted conversation. Never
  resend, substitute another conversation, or hand-edit continuation evidence.
- Browser observations never replace a current typed wait result.
- Every original candidate remains immutable evidence. Qualified representation
  is admitted from that original response without changing its continuation;
  only `restart_or_fail_closed` may invalidate it and reserve a successor.
