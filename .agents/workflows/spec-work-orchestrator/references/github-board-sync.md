# GitHub Board Sync

The GitHub Projects board is the human-oversight projection of spec-work
lifecycle state — the cross-feature sibling of `STATUS.md`. It is never
authority. `state.json` remains the sole operational authority; the
`work-state` bridge stays GitHub-free; board identifiers never enter
`state.json`. A sync fault is a structured recoverable warning, never a
lifecycle blocker. For bundle-scoped syncs its durable home is
`review/board-sync.md` in the
bundle (outside the approved-source inventory): the root appends one line
per fault — timestamp, intended Status, error — and reconciles that file
at three points: on every resume, before every later sync, and before the
final completion handoff. Reconciliation never replays a stored Status: it
recomputes the desired Status from the bundle's current phase and blockers
— and computes `Complete` only from fresh `verify-publication` REACHABLE
evidence obtained in that same reconciliation, never from the `complete`
phase alone — performs one sync to that value, and on success truncates
the file to empty — so a stale fault can never move the card backward or
forward past publication. A fault that
survives the completion handoff is reported to the user as the one
remaining manual step, so the board can never stay silently stale. The
map-exit projection below runs with no bundle and records its faults on
the map issue instead.

Verification status: every bundle-lifecycle operation below was exercised
end to end on 2026-08-14 (gh 2.97.0) against a live repository: creation,
linking, column rewrite, board layout switch, card add, and the full
Discuss -> Blocked -> Delivery -> Complete status walk. The map-exit
projection composes those same verified operations; its Backlog option
and marker guard were not part of that exercise.

## Tooling rules

- Invoke the real GitHub CLI binary directly for every `gh project`
  command. Do not route them through an RTK wrapper; it does not pass
  `gh project` through. Resolve the binary once per session with
  `command -v gh` (and confirm it is not a wrapper via `gh --version`
  printing a `gh version` line), then invoke that exact path. Homebrew
  layouts differ across machines; never hardcode one.
- All board mutations are root-owned semantic receipt work, the same
  pattern as follow-up issue publication. Support agents never sync the
  board.
- Prerequisite: the token must carry the `project` scope. Check with
  `gh auth status`. When it is missing, record one recoverable warning
  naming the operator command `gh auth refresh -s project` (an interactive
  one-time browser flow the operator runs; never attempt it from an agent)
  and continue the lifecycle without board sync until the scope exists.

## First-time board creation (once per repository)

Creation is not atomic across its steps, and concurrent roots on the same
repository could each observe "no board" — so it is serialized and made
crash-safe by a repository-scoped bootstrap claim settled the same way
wayfinder settles ticket claims — by creation order, because GitHub does
not enforce unique titles: before step 1, create an open issue titled
exactly `spec-work-board-bootstrap` whose body names your session id, then
re-read every open issue with that exact title; the one with the earliest
server-assigned `createdAt` owns the bootstrap. If that is not yours,
close yours with a `bootstrap-yielded` comment and wait, re-running
discovery until the owner's board appears or the owner's issue is closed
or older than one hour (stale — take over by the same rule). The owner's
issue body records each step's result as it completes
(project number and node id after step 1, `linked`, `columns`, `board`,
`stamped`), so a crashed bootstrap resumes from the recorded step rather
than creating a second project. Close the bootstrap issue after step 5.

1. Create the project and capture its number and node id:

   ```sh
   gh project create --owner <repo-owner> --title "Spec Work" --format json
   ```

2. Link it to the repository so it appears in the repo's Projects tab:

   ```sh
   gh project link <number> --owner <repo-owner> --repo <owner>/<repo>
   ```

3. Replace the built-in Status field's options with the lifecycle columns.
   Find the Status field id with
   `gh project field-list <number> --owner <repo-owner> --format json`,
   then rewrite its options (this makes the board a lifecycle Kanban; no
   web template is involved — `gh project create` has no template flag):

   ```sh
   gh api graphql -f query='mutation($f:ID!){updateProjectV2Field(input:{
     fieldId:$f, singleSelectOptions:[
       {name:"Backlog",  color:GRAY,   description:"Cleared capability awaiting a bundle"},
       {name:"Discuss",  color:BLUE,   description:"Interview and spec in progress"},
       {name:"Plan",     color:PURPLE, description:"Slice DAG and Pro planning"},
       {name:"Delivery", color:YELLOW, description:"Implementation, review, merges"},
       {name:"Blocked",  color:RED,    description:"Active bundle blocker"},
       {name:"Complete", color:GREEN,  description:"Published to the default branch"}
     ]}){projectV2Field{... on ProjectV2SingleSelectField{id}}}}' -f f=<status-field-id>
   ```

4. New projects open in Table layout. Switch the default view to Board —
   no UI step is needed. Find the view id, then:

   ```sh
   gh api graphql -f query='query{node(id:"<project-id>"){
     ... on ProjectV2{views(first:5){nodes{id name layout}}}}}'
   gh api graphql -f query='mutation{updateProjectV2View(input:{
     viewId:"<view-id>", layout:BOARD_LAYOUT}){projectV2View{layout}}}'
   ```

5. Stamp the project so it is unambiguous: set its short description to
   the exact marker `spec-work-board` via
   `gh project edit <number> --owner <repo-owner> --description
   "spec-work-board"`.

Rediscover the board later through the repository link, not by owner-wide
title listing (titles collide across repositories under one owner):

```sh
gh api graphql --paginate -f query='query($endCursor:String){
  repository(owner:"<owner>",name:"<repo>"){
    projectsV2(first:100, after:$endCursor){
      nodes{number title shortDescription}
      pageInfo{hasNextPage endCursor}}}}'
```

`--paginate` walks every page, so a marked board can never hide past a
page boundary. Select the repository-linked project whose
`shortDescription` is exactly `spec-work-board`. Exactly one must match
across all pages: zero means the board is not yet created (run first-time
creation); more than one is a fail-closed condition — record a recoverable
warning naming the project numbers and perform no mutation until the
operator removes the duplicate marker.

## Map-exit backlog projection (wayfinder repositories)

When a wayfinder map classifies **Complete**, the same closing session
projects the cleared way onto the board before the map issue closes — so
the finished map hands off as visible backlog cards, not prose in a
closed issue. This is root-owned semantic receipt work in a live HITL
session; the user confirms the capability list first (the wayfinder
skill's Reaching-the-destination step owns that confirmation). A map
whose confirmed list is empty — a pure-decision map — skips this whole
section, including board bootstrap, and proceeds to its closing comment.

1. Run the board rediscovery above; when zero marked boards exist, run
   first-time creation.
2. For each confirmed capability, mint one `enhancement` issue: inspect
   any repository issue template first and mirror it; the title is the
   capability name; the body names the goal, cites the map issue and
   every decision ticket the capability rests on by URL, and carries
   exactly one marker
   `<!-- spec-work-backlog:<map-number>/<capability-slug> -->`
   (kebab-case of the confirmed name; slugs must be unique within the
   map — disambiguate colliding names at confirmation). Mint it as a
   top-level issue, never with `--parent`: the wayfinder frontier query
   and derived decision index read only the map's sub-issues, so
   parentage — not labels — is what keeps backlog issues out of them.
   Minting is guarded by marker discovery: before creating, search all
   issue states for `spec-work-backlog:<map-number>/` and skip every
   capability whose exact marker already exists — open means already
   minted; closed means delivered or dropped, never re-mint. Issue
   search is eventually consistent, so after an unknown create result,
   confirm through a consistent read — list the repository's most
   recently created issues (the GraphQL `issues` connection ordered by
   `CREATED_AT`) and inspect their bodies for the marker — before
   creating again. These are spec-work owning issues, not wayfinder
   issues: no `wayfinder:*` label, no severity, no defect fingerprint.
3. Project every **open** issue carrying this map's
   `spec-work-backlog:<map-number>/` marker — not only those minted in
   this run — as a card with Status `Backlog`, using the same item-add
   and item-edit commands as the card contract below. Both commands are
   safe to repeat, so a rerun after a crash between mint and projection
   converges; closed-marker issues get no card.

A backlog card's issue becomes a feature bundle's owning issue at bundle
initialization through the user-supplied-owner path in the card contract
below. Cards are ordered manually within the column; the workflow
assigns no priority. For a minted issue not yet adopted by any bundle,
the recomputed desired Status is always `Backlog`.

A wayfinder session has no bundle, so `review/board-sync.md` is not
available as the fault home. Record a board fault (including a missing
`project` scope) as one comment on the map issue naming the fault — and,
for scope, the operator command `gh auth refresh -s project` — then
continue: minting requires only the `repo` scope and proceeds
regardless. Deferred projection has a durable owner: every later
root-owned board sync in this repository — ordinarily the next bundle
initialization — first sweeps every open `spec-work-backlog:` issue
that lacks a card into `Backlog`, then performs its own sync.

## The card: one owning issue per bundle

- `feature` bundles: a feature bundle has exactly one owning issue, and
  its identity is mandatory. At bundle initialization the root first
  checks for a user-supplied owner: an issue the user named, verified as
  open in this repository, is the owner and no issue is created. A
  backlog issue minted at map exit is the ordinary case: name it when
  starting the bundle, and verification and adoption follow this same
  path. Otherwise, before creating, the root checks the open issues
  carrying a `spec-work-backlog:` marker for one whose capability
  matches the accepted goal and surfaces a match to the user for
  adoption instead of creating a duplicate. Only when none matches does
  the root create the feature issue — inspect any repository issue
  template first and mirror it; label it `enhancement`; title is the
  accepted goal gist; body names the goal and carries the exact marker
  `<!-- spec-work-feature:<work_id> -->`. Creation is guarded by
  marker discovery: before creating, and after any unknown result, search
  the repository's open issues for that exact marker and adopt the match
  instead of creating a duplicate. Record the owner's URL under
  `FEATURE.md`'s required `## Source / Issue` heading (the
  semantic-authority home for owning-issue identity, mirroring the
  bug-fix `Source / Incident` field) so a resumed root re-selects the card
  from Markdown, never from memory or a title search. Before every board
  sync and before completion, the root validates that the heading holds
  exactly one `https://github.com/<owner>/<repo>/issues/<n>` URL; zero or
  more than one is a bundle blocker (`BLK-*`), not a warning, because
  completion cannot execute the owner's closure without it. This issue is
  the route-selected owning issue: it is not a defect, carries no
  fingerprint, and follows no defect-lifecycle discovery. Its completion
  contract is the `feature` branch of the core's owning-issue closure:
  close it directly, with a completion comment linking the merged PRs,
  when `verify-publication` proves REACHABLE — no fingerprint
  reconciliation, no marker reconciliation, no severity. Slice PRs
  reference it with non-closing references per the PR contract.
- `bug_fix` bundles: the verified owning defect issue is the card. The
  debugging route and the durable issue lifecycle own its creation and
  identity; this reference never creates or mutates defect issues. A
  `fix_full` bundle whose accepted fix fully resolves an in-scope defect
  may legitimately have no issue (the debugging route forbids creating one
  solely for it); such a bundle has no card and skips board projection
  entirely — record that skip once in `review/board-sync.md` so resume
  does not retry it.

Add the card to the board (safe to repeat; re-adding an existing item
returns it):

```sh
gh project item-add <number> --owner <repo-owner> --url <issue-url> --format json
```

## Status mapping and sync points

Map bundle state to the Status column:

| Bundle state | Status |
|---|---|
| phase `discuss` | Discuss |
| phase `plan` | Plan |
| phase `delivery` | Delivery |
| any active bundle blocker | Blocked |
| blocker cleared | back to the phase column |
| `complete` after `verify-publication` proves REACHABLE | Complete |

A pre-minted backlog card needs no special transition: bundle
initialization's first sync computes the phase column from this mapping
and moves the card out of `Backlog`.

Sync immediately after each successful bridge mutation that changes phase
or records or clears a bundle blocker — never before it commits. The one
exception is `Complete`: the bridge accepts `complete` before default-branch
publication is verified, so do not sync on that transition. Set `Complete`
only after `verify-publication` returns REACHABLE, in the same step that
closes the owning issue; until then the card stays in Delivery. Set the value by name (verified live; success is silent, exit
code 0 — its absence from the command's flag listing is a help-text gap):

```sh
gh project item-edit <number> --owner <repo-owner> \
  --url <issue-url> --field "Status" --value "<Option>"
```

The node-id form (`--project-id`, `--id`, `--field-id`,
`--single-select-option-id`) remains available for scripts that already
hold the ids; it avoids nothing otherwise — the by-name form resolves them
server-side in one call.

Slice-level events (`pr_open`, `merged`) do not move the feature card; the
card stays in Delivery and slice detail lives on the issue timeline
through the PRs' non-closing references.
