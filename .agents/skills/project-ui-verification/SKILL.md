---
name: project-ui-verification
description: >-
  Validate and bind proof for user-visible UI, displayed values, navigation,
  model-fed screen output, or accessibility changes in this repository. Use it
  at standalone, intermediate implementation-proof, final human checkpoint,
  and completion gates. Non-UI work is explicitly not_applicable; when UI proof
  is required, discover and run a truthful repository-native proof route or
  fail closed at that gate.
---

# Project UI Verification

Own the generic proof receipt and identity binding. Do not invent a browser,
device, emulator, simulator, test framework, screenshot tool, or authenticated
environment. At use time, discover a proof route from repository instructions,
manifests, scripts, CI, installed capabilities, and current product tooling. The
starter prescribes no `AGENTS.md` field or adapter name.

## Applicability

- Use `UI gate: not_applicable` in a slice plan when no user-visible UI,
  displayed value, navigation-visible value, model-fed screen output, or
  accessibility contract changes. No receipt is required.
- Use `snapshot_required_human_deferred` only for an intermediate UI slice in
  a spec-work bundle with a designated final UI slice.
- Use `final_human_required` for the designated final UI slice.
- If UI proof applies but no truthful proof route is available or runnable, stop
  with the exact workflow blocker before emitting a typed receipt. Never
  substitute a source-text assertion or a test-only product path.

The stable lifecycle token retains the word `snapshot`; the proof artifact may
represent a browser, desktop, terminal, device, rendered image, accessibility
tree, or other project-defined UI observation.

## Proof route contract

A proof route used by this skill must:

1. Build or launch from the exact worktree or committed head being proved.
2. Reach the acceptance target through unchanged production composition,
   navigation, state, and rendering paths. Deterministic data may enter only at
   a declared production-owned seam.
3. Emit a typed `project-ui-proof` JSON artifact whose `source` is
   `project_ui_adapter`, identity matches the receipt, and observation is a
   scalar `displayed_value` or a true `control_present` result.
4. Record the build/test/proof checks actually performed and any screenshots
   as supporting evidence. Screenshots alone are not the typed observation.
5. Keep the live proof session available when immediate human confirmation is
   required, then record confirmation or waiver and release it.

The workflow requires no advance declaration of this route. If the repository
has no truthful route when UI proof is required, the missing capability is a
blocker for that gate only.

## Receipt contract

Emit a JSON object with schema `project-ui-verification`. The validator
requires exact fields and rejects extras. Use:

- identity: `worktreeTree` for implementation proof; `headSha` and `headTree`
  for final committed-head proof;
- adapter: `adapterRequested`, `adapterActual`, and `proofSessionId`;
- source: `fixture`, `authenticated`, `live`, `local`, or `none`;
- evidence: `snapshotEvidence`, `screenshotPaths`, `testsReviewed`, and
  `automatedGates` (`build`, `tests`, `snapshot`);
- outcome: `verdict`, `humanConfirmation`, `sessionReleased`, and `blocker`.

Every admitted receipt must bind at least one typed artifact by repository-relative
path, SHA-256 digest, identity kind, and identity. `WAIVED` still requires
current automated proof. `BLOCKED` is reserved for a human-reported issue after
clean automated proof; it must name the issue and cannot masquerade as
acceptance.

For orchestrated proof, bind `workKind`, `workBundle`, `sliceId`,
`finalUiSliceId`, acceptance criterion, stable proof target, PR/head identity,
and task stage. Use the shared workflow's plan and state as authority; do not
invent identity values.

Read `references/receipt-contract.md` when implementing or changing a proof
route. It contains the exact implementation receipt and typed proof shapes
plus the final-checkpoint differences.

## Gate sequence

### Implementation proof

Run the proof route against the implementation worktree, persist the
worker-supplied receipt and proof through:

```sh
.agents/skills/project-ui-verification/scripts/persist-proof \
  --bundle <bundle> \
  --slice <PR-NN> \
  --receipt-payload <receipt.json> \
  --snapshot-payload <proof.json> \
  --implementation-worktree-root <absolute-worktree>
```

Validate with mode `implementation-proof`. A passing intermediate receipt uses
`DEFERRED_TO_PR_FINAL`, records the automated observation, releases the
implementation proof session, and does not claim final human acceptance.

### Final checkpoint and completion

After the designated final PR is otherwise clean, run the proof route on the
exact committed head and request the required human decision while its session
is active. Validate with `final-design-acceptance`; write the create-only
attestation through `--attestation-output`. After merge, validate the recorded
receipt again with `completion` and `--verify-attestation` as directed by the
shared core.

Use `final-checkpoint-issue` only to bind a truthful blocked checkpoint for
remediation. Never turn an unavailable proof route into PASS or WAIVED.

Run `scripts/validate-receipt --help` for every mode and anti-stale assertion.

## Boundaries

- The repository-native proof route acquires evidence; this skill validates and
  persists it.
- The root coordinator owns bundle state and acceptance transitions.
- An implementation or review task may return proof payloads but must not write
  coordinator state.
- UI proof does not replace functional tests, semantic review, CI,
  mergeability, issue disposition, or final delivery gates.
