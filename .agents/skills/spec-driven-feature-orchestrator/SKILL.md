---
name: spec-driven-feature-orchestrator
description: >-
  Use this skill when the user wants a repository feature or full bug fix
  researched, interviewed, specified, planned, implemented through
  harness-native user-visible worktree tasks, independently reviewed, and
  completed through merged GitHub pull requests. Route diagnosis-only defects to project-debugging. Do not
  use for narrow standalone edits, diagnosis-only reporting, or pull-request
  review alone.
---

# Spec-Driven Feature Orchestrator

This discoverable entrypoint classifies the request, constructs one closed
descriptor, and invokes the forced-only shared lifecycle. It does not restate
Discuss, Plan, Delivery, completion, Pro, task, or UI rules.

## Classify before loading the core

1. If the requested outcome is an ordinary defect, malformed value, stale UI,
   runtime, data-flow, persistence, concurrency, performance, or integration
   failure, load `../project-debugging/SKILL.md`.
   - Diagnosis, explanation, evidence collection, or reporting only stays in that
     skill's read-only diagnosis path. Do not load the shared core or create docs.
   - Fix, implement, resolve, ship, or PR-ready intent uses that skill's fix/ship
     path. It confirms root cause and the regression abstraction, constructs the bug
     descriptor, and then invokes the shared core.
2. If the requested outcome is a new or materially expanded product capability,
   continue here with the feature descriptor.
3. If neither classification is supportable from the request and inspected evidence,
   ask one concise outcome question. Do not create a bundle on speculation.

Explicit invocation of this skill does not force feature-shaped documentation for a
bug. The user's requested outcome controls routing; a fix/ship bug receives full
documentation parity under `docs/bug_fixes`.

## Feature descriptor

Construct this exact in-memory descriptor; do not persist a second manifest:

```text
workKind: feature
bundleParent: docs/features
primaryArtifact: FEATURE.md
planningGate: feature_discovery
```

Reject any mixed combination. Active identity is
`docs/features/in_progress/<YYYY-MM-DD-slug>/`, where the basename is `work_id`.

## Invoke the shared lifecycle

Read these files completely only after classification:

1. `../../workflows/spec-work-orchestrator/CORE.md`.
2. The phase-specific reference named by the core, only when entering that phase.

Initialize new feature state through:

```sh
.agents/workflows/spec-work-orchestrator/scripts/work-state init \
  --work-kind feature \
  --work-id <YYYY-MM-DD-slug> \
  --work-bundle docs/features/in_progress/<YYYY-MM-DD-slug>
```

The shared core owns all lifecycle behavior and coordinator authority from that point.
This adapter owns only feature-vs-bug classification and the feature descriptor.

## Feature domain hooks

- Discovery establishes the product goal, accepted behavior, scope, non-goals,
  compatibility constraints, document authority, acceptance, and consequential open
  decisions before planning.
- Always create the interview ledger. Ask the user only when evidence cannot resolve a
  consequential product, behavior, acceptance, or architecture decision.
- When implementation exposes defect-shaped evidence, use the debugging skill for
  root-cause work without changing the bundle's declared `workKind` unless the root
  returns to Discuss and the user explicitly changes the outcome.

## Canonical format boundary

The `scripts/feature-state` and `scripts/task-git-binding` paths are feature-default
aliases into the canonical shared tools and own no separate schema or behavior.
Workflow-owned artifacts use stable unversioned kind identifiers. An update replaces
the canonical contract in place; never add numeric versions or compatibility behavior
unless the user explicitly requests it. Unsupported state fails clearly; archive the
stale bundle or restart manually. Never migrate or reinterpret it, and do not backfill
historical bundles.
