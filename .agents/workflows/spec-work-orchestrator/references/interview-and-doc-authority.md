# Interview and Document Authority

Read this reference when entering or resuming Discuss, closing a question, or
when Plan exposes ambiguity. The root owns the user interview and every semantic
write. Support agents may inspect evidence during Discuss and Plan, but never ask
the user or close a product decision.

## Evidence order

Inspect before asking:

1. Current code and tests establish runtime behavior.
2. Domain docs, ADRs, architecture, invariants, and accepted wayfinder map
   decision tickets establish durable intended policy — cite the ticket URL,
   not only the map, so a reader can zoom to the primary discussion.
3. Current official external documentation or live data establishes external facts.
4. Audits, development docs, and remediation plans provide scoped constraints and evidence.
5. Historical plans are evidence only.

Record conflicts with both anchors; do not silently choose one. Current behavior
is not automatically intended policy, and an old plan is never authority merely
because it is specific.

When repository evidence is absent, treat each consequential gray area not
settled by the user request as low confidence. Record a linked user question
before Discuss closes. Do not re-ask a decision the user already supplied, and
do not convert a clear user statement into a confirmation question.

## Design tree and frontier

Apply `grilling.md` — the design-tree and frontier interview discipline — to
every Discuss interview: relentless rounds over the decision tree until the
frontier is empty. In this workflow:

- Format frontier questions with their ledger `Q-*` identifiers. When the
  active user-input tool structures questions natively, carry the title,
  body, options, and recommended answer into the tool call, and respect its
  per-call question limit; a frontier larger than the limit continues across
  consecutive calls within the same round.
- An empty frontier means every `GA-*` and `Q-*` entry holds a terminal
  status. When at least one question went to the user, do not leave Discuss
  until the user confirms you have reached a shared understanding. When
  evidence settled every entry with zero user-asked questions, the core's
  zero-question closure path applies unchanged.

When a decision-shaped gray area concerns a module's interface or seam, build
its recommendation with `design-it-twice.md`: dispatch parallel support agents
to design the interface several radically different ways, then put the
comparison to the user as the question's options and recommended answer.

When a decision-shaped gray area is best judged by feel — "does this state
model feel right", "what should this look like" — build its recommendation
with `prototype.md`: a throwaway prototype the user drives, whose verdict
closes the question through the ledger.

## Durable interview ledger

Create `INTERVIEW.md` with stable `GA-*` gray-area and `Q-*` question IDs.
Never reuse or renumber an ID.

Each gray area records:

- Status: `open | answered_by_docs | answered_by_user | question_required | closed | deferred | blocked`.
- Kind: `fact | decision`.
- Uncertainty and why it matters.
- Inspected evidence with repository or external anchors.
- Confidence: `high | medium | low`.
- Question: linked `Q-*` or `none`.

Each question records:

- Status: `open | answered | answered_by_docs | superseded | deferred | blocked`.
- Product-facing decision and why it is consequential.
- Recommendation or recommended default, options, concrete tradeoffs, and
  what breaks if we guess wrong — the concrete failure (migration, rewrite,
  breach, churn) that makes the question load-bearing.
- Answer/source, closure reason, linked `DEC-*`, and canonical-doc impact.

`Kind` separates fact-shaped gray areas from decision-shaped ones:

- A `fact` gray area is settled by the environment: current code, tests, docs,
  or live external data. Close it by evidence at any confidence. Never ask the
  user for a fact you could look up yourself.
- A `decision` gray area is a consequential choice among viable alternatives.
  The decisions are the user's. Evidence closes a `decision` as
  `answered_by_docs` only when a current authoritative source — an ADR, a
  `DEC-*`, an accepted spec, an accepted wayfinder map decision, or an
  explicit prior user statement — records that the same decision was already
  made; cite that anchor. Confidence never closes
  a new `decision`: a high-confidence recommendation is still a question, asked
  with that recommendation, not a recorded assumption.

Use evidence to close questions already answered by current sources; never re-ask
them on resume. Non-consequential engineering choices inside accepted scope stay
with the coordinator and do not enter the ledger. Keep questions product-facing
and recommendation-led. Ask independent decisions together in the current round;
a dependent decision belongs to a later round of the design tree. Respect the
active user-input tool's per-call question limit. Continue with later batches
after recording each answer.

`deferred` is legal only when the choice is outside current acceptance and has no
effect on slice boundaries, architecture, risk, or verification. Otherwise keep it
`blocked` and remain in Discuss.

## Decision closure transaction

When the user answers or evidence closes a question, update the following as one
root-owned semantic transaction:

1. Close the `Q-*` and its `GA-*` in `INTERVIEW.md`.
2. Add or supersede the linked `DEC-*` in `DECISIONS.md`.
3. Update the descriptor-selected primary specification's open decision IDs and
   affected scope or policy.
4. Update affected `ACCEPTANCE.md` criteria and slice `PLAN.md` files.
5. Record the canonical-document obligation and clear or create any linked `BLK-*`.

After mutating writers quiesce, re-enter Plan and construct one complete typed Plan
proposal over every current semantic artifact, slice, pair assessment, structured
task authorization, and acceptance assignment. Adopt it once through `work-state
adopt-plan`; do not refresh acceptance, roster, pair, or slice-plan projections as
independent repair steps. The adoption resets each changed unlocked criterion to
pending while leaving unrelated exact-current evidence alone. A completed epoch's
locked criteria cannot be edited or refreshed; reopen with a new `AC-*` ID for new
semantics. Any material Plan change invalidates the prior typed blind requirement.
Record a new requirement against the adopted digest before Delivery; run a fresh
blind-completeness pass only when that decision is `required_*`.
Any change to the primary specification, `INTERVIEW.md`, `DECISIONS.md`, `ACCEPTANCE.md`, or approved
primary source notes changes the specification digest and requires a typed applicability
receipt after Discuss closes. Record exact changed paths, source commits, repository
evidence anchors, and whether recommendations or implementation slices changed.
Both commits must resolve locally as commit objects. Changed paths must equal the
approved-source repository diff for a changed commit range, including deletions and
both sides of a rename under `--no-renames`, or the complete approved local-source
delta when the commit is intentionally unchanged. Record changed paths and evidence
anchors in the literal bundle-relative coordinate. For a changed `toSourceCommit`,
every current approved source byte must equal its blob at that commit before state can
advance or authorize a fresh primary. An evidence anchor may resolve as a current
regular source or at either the from or to commit, so an exact deleted source remains
valid evidence. The helper replays these checks without a network fetch during ordinary
validation.
`non_material` may advance the prior response's accepted-for digest only when both
booleans are false and the evidence proves `prior_evidence_applicable`. `material` and
`uncertain` require a fresh pushed checkpoint plus Pro primary. Blind findings that
change only the candidate plan do not rerun Pro, but any material plan change still
requires a fresh typed blind requirement for the exact new plan digest and a clean
receipt when that decision is `required_*`.

## Canonical ownership

During Discuss, apply `domain-modeling.md`: challenge terms against the
glossary, sharpen fuzzy language, and capture resolved terms and qualifying
decisions the moment they crystallise.

Route durable decisions to the narrowest owning document:

- Ubiquitous language and terminology -> the context glossary (`CONTEXT.md` /
  `CONTEXT-MAP.md`, format in `context-format.md`).
- Product or architecture decisions -> ADRs (default format in `adr-format.md`;
  an owning repository's existing ADR convention wins).
- System flow and component boundaries -> architecture docs.
- Business or state semantics -> domain docs and invariants.
- External-system facts -> reference docs with source and observation date.
- Findings and evidence -> audits.
- Workflow policy -> development docs.

List each obligation in the primary specification's Doc Authority table. The owning canonical
update must land before or with the first dependent implementation slice. A scoped
feature artifact may explain the decision, but does not replace its canonical owner.
