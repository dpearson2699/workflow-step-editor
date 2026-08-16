# Regression Test Abstraction Gate

Use this reference before naming or writing regression coverage or editing
production code for a bug fix. Complete the record below in task notes, the
implementation plan, or the test-worker receipt. A missing or incident-shaped
record is blocking.

The observed incident is evidence. The test specification is the generalized
behavioral invariant that the evidence shows was violated.

## Required Record

```yaml
test_abstraction_record:
  incident_evidence: "Observed evidence only; never the test specification"
  invariant: "General behavioral rule protected by the test"
  owning_seam: "Observable public or service boundary responsible for the rule"
  equivalence_classes:
    - "Material behavioral dimensions, partitions, and boundary conditions"
  fixture_strategy: "Coherent domain fixture/builder strategy, or why no reuse is clearer"
  essential_inputs:
    - "Causally explanatory values kept visible in the test"
  observable_outcomes:
    - "Stable externally meaningful state, result, or effect"
  generalized_test_name: "Behavioral name independent of the triggering incident"
  identity_specific_details:
    details: []
    justification: "Why identity changes behavior, or that none is load-bearing"
  display_contract: "User-visible proof obligation, or not_applicable with reason"
```

Every field is required. An empty list is valid only for
`identity_specific_details.details`; its justification must then state that no
identity-specific detail is load-bearing. `display_contract` may be
`not_applicable`, but must explain why the behavior has no user-visible proof
obligation.

## Decision Rules

- Keep `incident_evidence` concrete enough to reproduce and diagnose the
  failure, but do not reuse its labels as the invariant or test name unless
  identity changes the behavior.
- State `invariant` as a rule that holds for every behaviorally equivalent
  input. Place the test at the `owning_seam` where callers can observe that
  rule, not at a private implementation detail.
- List only materially distinct equivalence classes and boundary conditions.
  Parameterize cases when they share one behavioral explanation and one
  oracle. Split cases when their explanations or expected outcomes differ.
- A focused single-case test is complete when that case fully represents the
  invariant. Do not manufacture variation merely to use parameterization.
- Reuse a fixture or builder only when it represents a coherent domain concept.
  Prefer small focused helpers; avoid broad default-heavy builders that conceal
  why the scenario passes.
- Keep essential causal inputs visible at the test site. Named values should
  communicate domain meaning; helper defaults must not hide the condition that
  triggers the behavior.
- Assert stable observable outcomes. Do not assert private helper calls,
  incidental call counts, interaction choreography, or collection order unless
  that interaction or order is itself part of the public contract.
- Preserve identity-specific details only when changing the identity would
  change the behavior under test. Record that load-bearing reason explicitly.
- Prefer clarity over reuse. Duplication is acceptable when extracting it would
  obscure the invariant, inputs, or oracle.
- For user-visible behavior, record the durable model, display, automation, or
  snapshot proof that demonstrates the visible contract.

## Blockers

Stop before naming or writing regression coverage or editing production code
when any condition is true:

- The record is incomplete, or incident evidence is being used as the test
  specification.
- The invariant or generalized test name depends on an identity that has no
  load-bearing behavioral justification.
- Material equivalence classes or boundary conditions have not been considered.
- A parameterized test combines cases with different behavioral explanations
  or oracles, or introduces irrelevant variation.
- A fixture or builder mixes unrelated concepts, hides essential inputs, or
  relies on opaque defaults.
- Assertions prove implementation structure rather than the owning seam's
  observable contract.
- Assertions depend on private helpers, unspecified ordering, incidental call
  counts, or non-contractual choreography.
- A user-visible regression has no applicable display contract or justified
  `not_applicable` decision.

## Generalized Evaluation Properties

A compliant test design:

- Separates observed evidence from the invariant and generalized name.
- Selects meaningful behavioral partitions without parameterizing irrelevant
  differences.
- Shares fixtures only for coherent domain concepts while leaving causal inputs
  visible.
- Asserts stable observable outcomes at the owning seam.
- Accepts one complete focused case and splits cases with different oracles.
- Retains identity-specific data only with a load-bearing justification.

Review the completed record and the proposed test together. The test suite,
name, fixture, inputs, and assertions must protect the recorded invariant rather
than preserve only the incident that revealed it.
