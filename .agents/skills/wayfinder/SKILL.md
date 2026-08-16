---
name: wayfinder
description: >-
  Use this skill when the user wants to chart or work a wayfinder map — a
  shared map of decision tickets on GitHub for a loose idea too big for one
  session: "chart a map for X", "wayfind this", "work the map", "resolve the
  next decision ticket". Charting names the destination, grills
  breadth-first, and creates the map issue plus child decision tickets with
  native blocking edges. Working claims one frontier ticket per session,
  resolves it by its type (grilling, research, prototype, task), and
  graduates fog into new tickets. Planning only: it produces decisions, not
  deliverables; route the cleared way into the spec-work workflow. Do not
  use it for a nameable single feature (spec orchestrator), a defect
  (project-debugging), or an architecture survey
  (improve-codebase-architecture).
---

# Wayfinder

A loose idea has arrived — too big for one agent session, and wrapped in
fog: the way from here to the **destination** isn't visible yet. Wayfinding
is about finding that way, not charging at the destination. This skill
charts the way as a **shared map** on the repo's issue tracker, then works
its **decision tickets** — questions whose resolution is a decision, not
slices of a build to execute — one at a time until the route is clear.

The destination varies per effort, and naming it is the first act of
charting — it shapes every ticket. It might be a spec to hand off and
iterate on, a decision to lock before planning starts, or a change made in
place like a data-structure migration. In this repository the map plans
repository work: its destination is one or more nameable capabilities or
repository decisions, so that the exit route below has an owner.
Non-repository efforts have no exit here; chart them elsewhere.

## Plan, don't do

Wayfinder is **planning** by default: each ticket resolves a decision, and
the map is done when the way is clear — nothing left to decide before
someone goes and does the thing. The pull to just do the work is usually
the signal you've reached the edge of the map and it's time to hand off.
Produce decisions, not deliverables. In this repository the planning-only
rule has no override: a map's **Notes** may name skills and standing
preferences, never grant execution; deliverable work always leaves through
an owning execution lifecycle (see Workflow binding).

## Refer by name

Every map and ticket is an issue, so it has a **name** — its title. In
everything the human reads — narration, the map's Decisions-so-far — refer
to it by that name, never by a bare id, number, or slug. A wall of
`#42, #43, #44` is illegible; names read at a glance. The id and URL don't
vanish — a name wraps its link — but they ride _inside_ the name, never
stand in for it.

## The Map

The map is a single issue on this repo's issue tracker, labelled
`wayfinder:map` — the canonical artifact. Its tickets are child issues of
the map.

The map is an **index**, not a store. It lists the decisions made and
points at the tickets that hold their detail; a decision lives in exactly
one place — its ticket — so the map never restates it, only gists it and
links.

**Where the map, its child tickets, blocking, and frontier queries
physically live is tracker-specific.** This repository's tracker is
GitHub. Consult [references/github-operations.md](references/github-operations.md)
for how this repo expresses them — the label set, sub-issue and blocking
mutations, frontier query, claim, and resolution — including the one-time
capability probe. A host without native sub-issues and blocking is
unsupported here: the probe stops the session rather than degrading.

### The map body

The whole map at low resolution, loaded once per session. Open tickets are
**not** listed — they are open child issues, found by query.

```markdown
## Destination

<what reaching the end of this map looks like — the spec, decision, or
change this effort is finding its way to. One or two lines; every session
orients to it before choosing a ticket.>

## Notes

<domain; skills every session should consult; standing preferences for
this effort>

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance,
then zoom the link for the detail the ticket holds. In this repository
the index is DERIVED from the closed child tickets (each resolution
comment's first line carries `decision:` or `fact:` plus the gist) and
rendered on load per github-operations.md, so this section holds only
the pointer sentence below and is never hand-edited. -->

Derived from closed child tickets — see each ticket's resolution comment.

## Not yet specified

<!-- see "Fog of war": in-scope fog you can't ticket yet; graduates as the
frontier advances -->

## Out of scope

<!-- see "Out of scope": work ruled beyond the destination; closed, never
graduates -->
```

### Tickets

Each ticket is a **child issue** of the map; the tracker's issue id is its
identity. Its body is the question, sized to one 100K token agent session:

```markdown
## Question

<the decision or investigation this ticket resolves>
```

Each ticket carries a `wayfinder:<type>` label — one of `research`,
`prototype`, `grilling`, `task` (see [Ticket Types](#ticket-types)).

A session **claims** a ticket **first**, before any work, so concurrent
sessions skip it. In this repository the claim is settled by the comment
protocol in `references/github-operations.md` — post a claim comment,
verify you hold the earliest active claim, and only then assign the ticket
to the dev driving the map. The assignee marks a won claim; an open ticket
with no active claim comment and no assignee is unclaimed.

Blocking uses the tracker's **native** dependency relationship — essential
because it renders the frontier _visually_ in the tracker's own UI, so the
human sees what's takeable without opening the map. Native blocking is
required in this repository; there is no body-convention fallback. A ticket is
**unblocked** when every ticket blocking it is closed; the **frontier** is
the open, unblocked, unclaimed children — the edge of the known.

The answer isn't part of the body — it's recorded on resolution (see
[Work through the map](#work-through-the-map)). Assets created while
resolving a ticket are linked from the issue, not pasted in.

## Ticket Types

Every ticket is either **HITL** — human in the loop, worked _with_ a human
who speaks for themselves — or **AFK**, driven by the agent alone. A HITL
ticket only resolves through that live exchange; the agent never stands in
for the human's side of it (a grilling agent that answers its own
questions has broken this).

- **Research** (AFK): Reading documentation, third-party APIs, or local
  resources like knowledge bases to surface a fact a decision waits on.
  Resolved by an AFK read-only research **sub-agent** (spawn brief below).
  Use when knowledge outside the current working directory is required.
- **Prototype** (HITL): Raise the fidelity of the discussion by making a
  cheap, rough, concrete artifact to react to — an outline, a rough take,
  a stub, or UI/logic code via
  `../../workflows/spec-work-orchestrator/references/prototype.md`. Links
  the prototype as an asset. Use when "how should it look" or "how should
  it behave" is the key question. Prototypes are what stop wayfinder
  becoming waterfall — a huge amount of low-fidelity upfront planning —
  because they're a high-fidelity way to get feedback, so build many. The
  fidelity rule: basic questions resolve in discussion, but when the answer
  has to be _seen_ or _felt_ in action, prototype it. A prototype iterates
  with the user's feedback across variants (A, B, C — then a D that takes
  the best of each) and lands on a throwaway branch — a spec plus real code
  — that the implementer can later go and reference and copy from.
- **Grilling** (HITL): Conversation. The default case. Always apply
  `../../workflows/spec-work-orchestrator/references/grilling.md` and
  `../../workflows/spec-work-orchestrator/references/domain-modeling.md`.
- **Task** (HITL or AFK): Manual work that must happen before a _decision_
  can be made — nothing to decide, prototype, or research, but the
  discussion is blocked until it's done. Signing up for a service so its
  API can be judged, provisioning access, moving data so its shape can be
  seen. This is the one type that _does_ rather than decides — and it
  earns its place by unblocking a decision, not by delivering the
  destination. The agent drives it alone where it can (AFK); otherwise it
  hands the human a precise checklist (HITL). Resolved when the work is
  done; the answer records what was done and any resulting facts
  (credentials location, new URLs, row counts) later tickets depend on.
  Authority boundary: working a map never authorizes external mutations.
  Account creation, provisioning, credential handling, payments, and data
  movement are HITL by default — the ticket produces the checklist and the
  human performs the act — unless the user grants explicit authority for
  that specific mutation on that ticket. AFK task tickets are limited to
  read-only repository work and artifacts in the OS temp directory;
  wayfinder owns no paths, branch, or bundle, so any repository write is
  HITL or belongs to another route.

## Fog of war

The map is _deliberately_ incomplete: don't chart what you can't yet see.
Beyond the live tickets lies the **fog of war** — the dim view of decisions
and investigations you can tell are coming but can't yet pin down, because
they hang on questions still open. Resolving a ticket clears the fog ahead
of it, graduating whatever's now specifiable into fresh tickets — one at a
time, until the way to the destination is clear and no tickets remain.

The map's **Not yet specified** section is where that dim view is written
down: the suspected question, the area to revisit later. It's the
undiscovered frontier _toward_ the destination — everything here is in
scope, just not sharp enough to ticket. Write as loosely or as fully as
the view allows; it doubles as a signpost for collaborators reading where
the effort is headed.

**Fog or ticket?** The test is whether you can state the question
precisely now — _not_ whether you can answer it now.

- **Ticket when** the question is already sharp — even if it's blocked and
  you can't act on it yet.
- **Not yet specified when** you can't yet phrase it that sharply. Don't
  pre-slice the fog into ticket-sized pieces: it's coarser than a ticket,
  and one patch may graduate into several tickets, or none, once the
  frontier reaches it.

**Not yet specified** excludes what's already decided (Decisions so far),
what's already a live ticket, and what's out of scope (the next section).

## Out of scope

Fog only ever gathers _toward_ the destination. The destination fixes the
scope, so work beyond it is **out of scope** — it isn't fog, and it
doesn't belong in **Not yet specified**. It gets its own **Out of scope**
section on the map: work you've consciously ruled out of _this_ effort.
Scope, not sharpness, lands it here.

Out-of-scope work never graduates — the frontier stops at the destination
— so it returns only if the destination is redrawn, and then as a fresh
effort, not a resumption.

Ruling something out of scope is a scoping act, not a step on the route.
When a ticket that already exists turns out to sit past the destination —
mis-scoped in while charting, or exposed by a resolution — **close it** (a
closed ticket is unambiguously off the frontier) and leave one line in the
**Out of scope** section: the gist plus why it's out of scope, linking the
closed ticket. It stays out of **Decisions so far**, which records the
route actually walked — a scope boundary isn't a step on it.

## Invocation

Two modes. Either way, **never resolve more than one ticket per session**
— with the exception of research tickets.

### Chart the map

User invokes with a loose idea.

1. **Name the destination.** Run a grilling and domain-modeling session
   (per the Grilling ticket-type references above) to pin down what this
   map is finding its way to — the spec, decision, or change. The
   destination fixes the scope, so it's settled first.
2. **Map the frontier.** Grill again, **breadth-first** this time: fan out
   across the whole space rather than deep on any one thread, surfacing
   the open decisions and the first steps takeable now. **If this
   surfaces no fog** — the way to the destination is already clear, the
   whole journey small enough for one session — you don't need a map.
   Stop and ask the user how they'd like to proceed.
3. **Create the map** (label `wayfinder:map`): Destination and Notes
   filled in, Decisions-so-far empty, the fog sketched into **Not yet
   specified**.
4. **Create the tickets you can specify now** as child issues of the map,
   each created already attached (`gh issue create --parent <map>`), so a
   mid-chart failure never leaves an orphan issue with no parent — then
   wire blocking edges in a **second pass** (issues need ids before they
   can reference each other), idempotently: before adding an edge, read
   the ticket's `blockedBy` and skip edges that already exist, so a retried
   pass creates no duplicates. Wiring sorts them into the frontier and the
   blocked; everything you can't yet specify stays in the fog — the **Not
   yet specified** section.
5. **Fire the research sub-agents.** Run the frontier query; for each
   `research` ticket it classifies as open, unblocked, and unclaimed (a
   research ticket behind an open blocker waits its turn), claim it, then
   spin up an AFK research sub-agent to resolve it in parallel. Each result
   goes through the full resolution transaction of
   Work-through-the-map step 4 — a `fact:`-prefixed resolution comment with
   citations, then close — and any fog it clears graduates per step 5. A
   research ticket left open with findings only in a comment is not
   resolved and will run again.
6. Stop — charting is one session's work; it hand-resolves nothing else.

### Work through the map

User invokes with a map (URL or number). A ticket is **optional** —
without one, you pick the next decision, not the user.

1. Load the **map** — the low-res view, not every ticket body.
2. Choose the ticket. If the user named one, verify it is on the frontier
   — an open, unblocked, unclaimed child of the map — before taking it; a
   named ticket that is closed, blocked, or claimed is reported back, not
   worked. Otherwise take the first frontier ticket in order. **Claim it**
   through the claim protocol before any work.
3. Resolve it — **zoom as needed**: fetch the full body of any related or
   closed ticket on demand; apply the references the `## Notes` block
   names. If in doubt, use the grilling and domain-modeling references.
4. Record the resolution: post the answer as a **resolution comment** and
   **close** the issue. The comment's first line is the ticket's kind plus
   a one-line gist — `decision: <gist>` when the user chose (grilling and
   prototype tickets), `fact: <gist>` when the ticket surfaced information
   without a user choice (research and task tickets) — because the map's
   Decisions-so-far is derived from exactly these lines (see
   github-operations.md); there is no separate append. Only `decision:`
   entries are accepted user decisions downstream; a `fact:` entry is
   evidence, and any choice it implies still goes to the user.
5. Add newly-surfaced tickets (create-then-wire); graduate any fog the
   answer has made specifiable, clearing each graduated patch from **Not
   yet specified** so it lives only as its new ticket. If the answer
   reveals a ticket — this one or another — sits beyond the destination,
   **rule it out of scope** rather than resolving it on the route. If the
   decision invalidates other tickets, close each as superseded with a
   comment whose first line is `superseded: <superseding ticket>` (and
   `out-of-scope: <reason>` for scope closures) — never delete an issue;
   closed history is the record, and those prefixes keep such closures out
   of the derived decision index.

The user may run unblocked tickets in parallel, so expect other sessions
to be editing the tracker concurrently.

### Reaching the destination

An empty frontier is not by itself completion. After resolving a ticket,
classify the map:

- **Complete** — no open child tickets, **Not yet specified** is empty, and
  the destination is answerable from Decisions-so-far. Post a closing
  comment on the map summarizing the destination and pointing at the exit
  route, then close the map issue.
- **Blocked** — every open ticket is claimed by another session, or is
  blocked and no open blocker is on the frontier (a dependency cycle). A
  claim older than 24 hours is stale and the session releases it itself
  per the claim protocol, so a stale claim never produces Blocked. Do not
  close; report the specific tickets and the reason, and ask the user to
  break the cycle.
- **Foggy** — open fog remains but nothing is ticketable yet. Do not
  close; report the fog and stop for the session.

Only a Complete map hands off. A closed map issue is the terminal marker;
an open map is in progress regardless of frontier size.

## Workflow binding

- Planning only, enforced: this skill never starts a spec-work bundle and
  never edits production code. A `task` ticket does enablement work
  (provisioning, access, data moves), never product implementation.
- Labels follow the `wayfinder` contract in
  `.github/issue-label-policy.json`: exactly one `wayfinder:*` label per
  issue, never a creation issue type or severity. Wayfinder issues are a
  planning class; the defect lifecycle ignores them, and this skill never
  mutates defect issues.
- The research sub-agent brief (AFK, read-only):

  ```text
  ROLE: researcher. READ-ONLY. RESPOND JSON.
  QUESTION: <ticket question verbatim>.
  SOURCES: high-trust primary only (official docs, specs, live APIs).
  PRESERVE EXACT: URLs, versions, quotes <=15 words.
  OUTPUT JSON ONLY: {answer, confidence, findings:[{fact, source_url,
  observed_date}], open_gaps}. NO PROSE.
  ```

  The root turns the receipt into the ticket's resolution comment with
  citations and confidence.
- When the map's destination is reached and the way is clear, route each
  cleared, nameable capability into
  `../spec-driven-feature-orchestrator/SKILL.md`, and charter the map's
  decisions into that capability's Discuss:
  - The root cites each decision ticket the capability rests on by URL as a
    durable **primary source** — in the feature artifact and in the matching
    `INTERVIEW.md` gray areas — so the agent can go and read the ticket when
    the gist is unclear. The map and the spec only gist a decision; the
    ticket holds it.
  - `decision:` entries in Decisions-so-far are accepted prior user
    decisions: Discuss closes matching `decision` gray areas as
    `answered_by_docs` by citing the ticket, per
    `../../workflows/spec-work-orchestrator/references/interview-and-doc-authority.md`,
    and never re-asks them. `fact:` entries are evidence only — they close
    `fact` gray areas and inform recommendations, but any choice resting on
    them is still asked.
  - This is the map's exit — there is no separate spec-compilation step.
    Each capability's Discuss primary artifact is its dense spec, linking
    back to every decision ticket it rests on; a spec that summarised the
    tickets without those links would be only a summary of what was
    actually said, cut off from its primary source.
