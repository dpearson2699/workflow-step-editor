# UI Gate Ownership

Read this reference when classifying UI acceptance, planning or delivering a
UI-affecting slice, or closing final UI acceptance. Load
`project-ui-verification` before acquiring or validating proof.

## Proof route boundary

This workflow owns proof identity and acceptance sequencing; it does not assume
a browser, device, emulator, simulator, desktop framework, test harness, or
authenticated environment. At the UI proof gate, discover a truthful
repository-native route that can build or launch the real product path, reach
the proof target through unchanged production composition and rendering, emit
the typed artifact required by `project-ui-verification`, and release its
session safely. The workflow prescribes no project configuration field.

- Non-UI work uses `UI gate: not_applicable` and requires no UI receipt.
- UI work without an available and runnable proof route is blocked. Do not replace
  runtime proof with source inspection, a screenshot alone, or a test-only
  product path.
- The stable lifecycle token `snapshot_required_human_deferred` and receipt
  field `snapshotEvidence` retain the word `snapshot`; their typed proof may
  describe any project-defined user-visible surface.

## Spec-work and slice contract

Every UI-affecting bundle declares `Policy: final_pr_design_gate` and exactly
one `Final UI slice: PR-*` in its primary specification. A single UI PR is the
final UI slice. Order it after every other UI-affecting slice.

Every slice plan declares one UI gate classification:

- `not_applicable` for a non-UI slice;
- `snapshot_required_human_deferred` for an intermediate UI slice; or
- `final_human_required` for the designated final UI slice.

Slices sharing final UI acceptance are coupled even when their paths are
disjoint. Every UI slice owns one automated acceptance `AC-*` and one stable
`UI proof target`. The typed receipt and proof artifact repeat both exactly.
Non-UI slices use `none`. Automated UI acceptance IDs are unique.

Represent bundle-level design acceptance with a separate stable `AC-*` owned by
the final UI slice. Its plan declares `Final design acceptance: AC-*`; every
other plan uses `none`. Intermediate proof never passes or waives that
criterion.

## Implementation proof

`DEFERRED_TO_PR_FINAL` is a `gatePhase: implementation_proof` result. Both
intermediate and designated final UI slices may use it only after the discovered
project proof route proves the changed target from the exact implementation
worktree, applicable build and test gates pass, and the proof session is
released.

The proof route emits a `project-ui-proof` artifact with
`source: project_ui_adapter`, the exact task stage and slice, the plan-owned
automated acceptance and proof target, matching tree identity, and exactly one
observation: a finite scalar `displayed_value` or `control_present: true`.
The `project-ui-verification` receipt binds that artifact by repository-relative
path, SHA-256, identity kind, and identity; it also records the requested and
actual adapter, proof session, build/test/proof results, and any supporting
screenshots. Screenshots do not replace the typed observation.

The root persists worker-supplied proof through
`project-ui-verification/scripts/persist-proof` and validates it with
`validate-receipt --mode implementation-proof` against the separate slice
worktree. Missing, stale, cross-slice, mismatched, hand-authored, or unavailable
proof fails closed.

An intermediate UI PR may publish, review, and merge on this deferral. The
designated final UI PR may publish and enter review but may not merge.
`DEFERRED_TO_PR_FINAL` never satisfies final design acceptance or completion.
The final pre-review bundle projection therefore leaves only the human design
`AC-*` pending and unlocked with null `completion_binding`; all other acceptance
is passed or waived and locked.

After persistence, record the slice's automated UI `AC-*` as `passed` only with
the exact typed implementation receipt. `waived` and generic evidence are
invalid. Revalidate that receipt against the current slice head or tree before
each UI merge.

## Final UI checkpoint

Request the human checkpoint only after ordinary review, CI, verification,
mergeability, base synchronization, and current-head automated UI proof are
clean. Run the discovered project proof route on the exact committed head and
emit `gatePhase: final_human_checkpoint`. Keep that route's live proof session
on the changed target until the user confirms, explicitly waives, or reports
an issue; then release it.

Confirmation or waiver closes the final design `AC-*` only after
`validate-receipt --mode final-design-acceptance` succeeds for the exact
bundle, slice, final-slice, head, PR, plan-derived acceptance ID, proof target,
and clean review binding. A waiver changes only the human verdict and still
requires current automated proof.

A reported issue produces a canonical `BLOCKED` receipt. The root consumes it
through `work-state reopen-final-ui-remediation --ui-receipt <receipt>` with the
current slice revision and implementation roster digest. That single transition
retains PR and task lineage, invalidates final-slice acceptance, reauthorizes
the same implementation owner, blocks the same review owner, and returns the
slice to remediation. Invalid or stale inputs leave state unchanged. Any new
head or final-slice change invalidates prior final UI evidence.

Store typed receipts and proof under `pr/<PR-id>/evidence/`. For the final
design acceptance, record both the typed receipt and validator-created
`project-ui-attestation`. Revalidate with `--mode completion` immediately
before `complete`. Store each attestation sidecar at a fresh
mode/revision/receipt-digest-keyed path; never overwrite a receipt, proof,
lifecycle, acceptance-evidence, or completion-binding path.

## Standalone boundary

Outside an active spec-work bundle, UI work uses the standalone mode and
immediate human confirmation through `project-ui-verification`. A missing
proof route is still a blocker at this gate. Work correctly classified as
`not_applicable` requires no UI proof. Never infer the deferred exception from
PR order or prose in a task receipt.
