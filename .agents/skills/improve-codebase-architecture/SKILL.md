---
name: improve-codebase-architecture
description: >-
  Use this skill only when the user explicitly asks for an architecture or
  deepening survey of the current repository — "survey the architecture",
  "find deepening opportunities", "where is the codebase getting muddy",
  "architecture health check", or a periodic maintenance review. It scans for
  shallow modules and friction using the shared codebase-design vocabulary,
  presents candidates as a visual HTML report, grills through the candidate
  the user picks, and routes the outcome into a durable GitHub issue or the
  spec-work workflow. Do not use it to execute a refactor, review a pull
  request, explain code, or fix a bug. It never starts a work bundle and
  never edits production code.
---

# Improve Codebase Architecture

Surface architectural friction and propose **deepening opportunities** —
refactors that turn shallow modules into deep ones. The aim is testability
and AI-navigability.

This command is _informed_ by the project's domain model and built on a
shared design vocabulary:

- Read
  `../../workflows/spec-work-orchestrator/references/codebase-design.md` for
  the architecture vocabulary (**module**, **interface**, **depth**,
  **seam**, **adapter**, **leverage**, **locality**) and its principles (the
  deletion test, "the interface is the test surface", "one adapter =
  hypothetical seam, two = real"). Use these terms exactly in every
  suggestion — don't drift into "component," "service," "API," or
  "boundary."
- The domain language in `CONTEXT.md` gives names to good seams; ADRs in the
  repository's ADR location record decisions this command should not
  re-litigate.

This skill is survey and triage only. It never edits production code, never
starts a spec-work bundle, and never satisfies any lifecycle gate. Its only
repository writes are the domain-modeling side effects named in step 3.

## Process

### 1. Explore

**Scope before you scan — YAGNI.** Deepening a module pays off by making
future changes to it easier, so put extra weight on the parts of the codebase
that have recently changed. Decide *where* to look before you look:

- If the user named a direction — a module, a subsystem, a pain point — take
  it, and skip the inference below.
- Otherwise, walk back a good stretch of the commit history
  (`git log --oneline`) to find the codebase's hot spots — the files and
  areas that keep coming up — and let those paths pull your attention first.
  If the changes are scattered with no clear hot spot, widen the net.

Read the project's domain glossary (`CONTEXT.md`) and any ADRs in the area
you're touching first.

Then spawn a read-only sub-agent to walk the codebase. Don't follow rigid
heuristics — explore organically and note where you experience friction:

- Where does understanding one concept require bouncing between many small
  modules?
- Where are modules **shallow** — interface nearly as complex as the
  implementation?
- Where have pure functions been extracted just for testability, but the
  real bugs hide in how they're called (no **locality**)?
- Where do tightly-coupled modules leak across their seams?
- Which parts of the codebase are untested, or hard to test through their
  current interface?

Apply the **deletion test** to anything you suspect is shallow: would
deleting it concentrate complexity, or just move it? A "yes, concentrates"
is the signal you want.

### 2. Present candidates as an HTML report

Write a single CDN-backed HTML file to the OS temp directory so nothing lands in
the repo. Resolve the temp dir from `$TMPDIR`, falling back to `/tmp` (or
`%TEMP%` on Windows), and write to
`<tmpdir>/architecture-review-<timestamp>.html` so each run gets a fresh
file. Open it for the user — `xdg-open <path>` on Linux, `open <path>` on
macOS, `start <path>` on Windows — and tell them the absolute path.

The report uses **Tailwind via CDN** for layout and styling, and **Mermaid
via CDN** for diagrams where a graph/flow/sequence reliably communicates the
structure. Mix Mermaid with hand-crafted CSS/SVG visuals — use Mermaid when
relationships are graph-shaped (call graphs, dependencies, sequences), and
hand-built divs/SVG when you want something more editorial (mass diagrams,
cross-sections, collapse animations). Each candidate gets a **before/after
visualisation**. Be visual.

For each candidate, render a card with:

- **Files** — which files/modules are involved
- **Problem** — why the current architecture is causing friction
- **Solution** — plain English description of what would change
- **Benefits** — explained in terms of locality and leverage, and how tests
  would improve
- **Before / After diagram** — side-by-side, custom-drawn, illustrating the
  shallowness and the deepening
- **Recommendation strength** — one of `Strong`, `Worth exploring`,
  `Speculative`, rendered as a badge

End the report with a **Top recommendation** section: which candidate you'd
tackle first and why.

**Use CONTEXT.md vocabulary for the domain, and the codebase-design
vocabulary for the architecture.** If `CONTEXT.md` defines "Order," talk
about "the Order intake module" — not "the FooBarHandler," and not "the
Order service."

**ADR conflicts**: if a candidate contradicts an existing ADR, only surface
it when the friction is real enough to warrant revisiting the ADR. Mark it
clearly in the card (e.g. a warning callout: _"contradicts ADR-0007 — but
worth reopening because…"_). Don't list every theoretical refactor an ADR
forbids.

See [references/html-report.md](references/html-report.md) for the full HTML
scaffold, diagram patterns, and styling guidance.

Do NOT propose interfaces yet. After the file is written, ask the user:
"Which of these would you like to explore?"

### 3. Grilling loop

Once the user picks a candidate, run a grilling session per
`../../workflows/spec-work-orchestrator/references/grilling.md` to walk the
decision tree with them — constraints, dependencies, the shape of the
deepened module, what sits behind the seam, what tests survive.

Side effects happen inline as decisions crystallize — apply
`../../workflows/spec-work-orchestrator/references/domain-modeling.md` to
keep the domain model current as you go:

- **Naming a deepened module after a concept not in `CONTEXT.md`?** Add the
  term to `CONTEXT.md`. Create the file lazily if it doesn't exist.
- **Sharpening a fuzzy term during the conversation?** Update `CONTEXT.md`
  right there.
- **User rejects the candidate with a load-bearing reason?** Offer an ADR,
  framed as: _"Want me to record this as an ADR so future architecture
  reviews don't re-suggest it?"_ Only offer when the reason would actually
  be needed by a future explorer to avoid re-suggesting the same thing —
  skip ephemeral reasons ("not worth it right now") and self-evident ones.
- **Want to explore alternative interfaces for the deepened module?** Use
  the parallel sub-agent pattern in
  `../../workflows/spec-work-orchestrator/references/design-it-twice.md`.

### 4. Route the outcome

A validated candidate leaves this skill through an owned lifecycle; this
skill never implements it.

- Default: hand it to `../spec-driven-feature-orchestrator/SKILL.md` as a
  feature request — a deepening is a materially expanded capability, not a
  defect. The grilled constraints and decisions become Discuss evidence
  there; the orchestrator creates the owning `enhancement` issue at bundle
  init.
- When the user wants it recorded but not built now: create one
  `enhancement` issue through the API or CLI, mirroring the repository
  issue template, containing the candidate card's problem, solution, and
  wins, and add it to the Spec Work board as a `Backlog` card in the same
  step per the Backlog projection section of
  `../../workflows/spec-work-orchestrator/references/github-board-sync.md`.
  It carries no defect fingerprint and does not enter the defect
  lifecycle; the future bundle adopts it as its owning issue.
- If the grilling reveals an actual defect (a failed invariant with
  affected surfaces), that part alone routes through the durable GitHub
  issue lifecycle in `AGENTS.md`.
- A rejected candidate ends as an ADR (per step 3) or as nothing.

Stop after routing. Survey, report, grilling, and routing are this skill's
whole job.
