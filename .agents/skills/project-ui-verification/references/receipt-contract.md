# Project UI Receipt Contract

Use these shapes when implementing the project adapter. Replace every example
identity, path, digest, acceptance ID, target, and session with values observed
from the active plan, state, Git tree, and adapter run. The validator rejects
missing and extra fields.

## Implementation proof receipt

```json
{
  "schema": "project-ui-verification",
  "gatePhase": "implementation_proof",
  "taskStage": "intermediate_orchestrated_pr",
  "workKind": "feature",
  "workBundle": "docs/features/in_progress/2026-08-11-example",
  "sliceId": "PR-01",
  "finalUiSliceId": "PR-02",
  "prNumber": null,
  "headSha": null,
  "headTree": null,
  "worktreeTree": "1111111111111111111111111111111111111111",
  "verdict": "DEFERRED_TO_PR_FINAL",
  "changeKind": "ui",
  "acceptanceCriterion": "AC-001",
  "proofTarget": "dashboard.export_button",
  "evidenceSource": "fixture",
  "adapterRequested": "project-ui-adapter",
  "adapterActual": "project-ui-adapter",
  "proofSessionId": "proof-session-01",
  "builtFromCurrentWorktree": true,
  "snapshotEvidence": [
    {
      "artifact": "docs/features/in_progress/2026-08-11-example/pr/PR-01/evidence/ui-proof/1111111111111111111111111111111111111111/snapshot.json",
      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "identityKind": "worktree_tree",
      "identity": "1111111111111111111111111111111111111111"
    }
  ],
  "screenshotPaths": [],
  "testsReviewed": ["project adapter acceptance probe"],
  "automatedGates": {
    "build": "PASS",
    "tests": "PASS",
    "snapshot": "PASS"
  },
  "humanConfirmation": "deferred_to_final_ui_gate",
  "sessionReleased": true,
  "blocker": null
}
```

The final UI slice also uses `taskStage: final_orchestrated_pr` during its
implementation proof. `persist-proof` copies the sibling payloads into the
canonical artifact path shown above and checks their exact bytes and digest.

## Typed proof artifact

```json
{
  "schema": "project-ui-proof",
  "source": "project_ui_adapter",
  "taskStage": "intermediate_orchestrated_pr",
  "sliceId": "PR-01",
  "identityKind": "worktree_tree",
  "identity": "1111111111111111111111111111111111111111",
  "automatedAcceptanceId": "AC-001",
  "proofTarget": "dashboard.export_button",
  "observation": {
    "kind": "control_present",
    "value": true
  }
}
```

`observation.kind` is `control_present` with literal `true`, or
`displayed_value` with a non-empty string, boolean, integer, or finite number.

## Final checkpoint differences

For `final-design-acceptance`, use:

- `gatePhase: final_human_checkpoint`;
- `taskStage: final_orchestrated_pr`;
- the final slice for both slice identifiers;
- positive `prNumber`, exact 40-hex `headSha`, exact Git `headTree`, and null
  `worktreeTree`;
- proof identity kind `head_tree` and identity equal to `headTree`;
- `PASS` plus `confirmed`, or `WAIVED` plus `waived`;
- `sessionReleased: true` after the human decision; and
- a null blocker.

A human-reported issue after clean automated proof uses `BLOCKED`, `blocked`,
and a non-empty blocker, with all three automated gates still `PASS`. Adapter
absence is an earlier workflow blocker and does not produce a typed receipt.

Evidence sources are exactly `fixture`, `authenticated`, `live`, `local`, or
`none`; an admitted proof uses a source other than `none`. Change kinds are
exactly `ui`, `displayed-data`, `navigation-visible-value`,
`model-fed-ui-output`, or `accessibility`.
