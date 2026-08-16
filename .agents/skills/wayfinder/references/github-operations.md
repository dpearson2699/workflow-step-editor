# Wayfinding operations on GitHub

This repository's tracker is GitHub. Every operation below was verified live
against a real repository on 2026-08-14 with `gh` 2.97.0. Every map or
ticket mutation goes through `gh` — never assert tracker state from prose.

Run each operation as its own single command. Do not chain node IDs through
shell word-splitting (zsh does not split unquoted variables; a chained
`set -- $ids` silently passes empty IDs).

## Capability probe (once per repository)

```sh
gh api graphql -f query='{ m: __type(name:"Mutation"){fields{name}} }' \
  --jq '[.data.m.fields[].name] | map(select(. == "addSubIssue" or . == "addBlockedBy")) | length'
```

Judge the two outcomes separately. A non-zero `gh` exit (network,
authentication, rate limit) is a transient failure, not a capability
verdict: surface it and stop; do not degrade. A zero exit that prints `2`
selects NATIVE mode (everything below). A zero exit that prints less than
`2` is a real capability absence — the host has no native hierarchy or
blocking — and stops the session with that report; do not chart on such a
host, there is no supported fallback in this repository. The count is
computed inside the successful `--jq` reply so a pipeline exit code never
masks the difference.

## Ensure labels (per repository, before first map)

List existing labels with an explicit full limit (`gh label list --limit
1000 --json name`; the default is 30 and would hide existing labels), then
create each wayfinder label that is missing, using the exact names,
colors, and descriptions in `.github/issue-label-policy.json` under
`wayfinder.label_metadata`:

```sh
gh label create "wayfinder:map" --color 1d76db \
  --description "Wayfinder map: the canonical index of one fog-wrapped effort's decisions"
```

## Create the map and its tickets

Before the first creation in a repository, inspect `.github/ISSUE_TEMPLATE/`
(and any issue forms) and mirror the applicable template's required title
format, body sections, and labels in the `gh issue create` payload; the map
and ticket bodies below fill the template's free-form section. Absent a
template, use the bodies as shown.

```sh
gh issue create --title "Map: <destination gist>" --label "wayfinder:map" \
  --body "<map body per the template in SKILL.md>"
gh issue create --title "<question title>" --label "wayfinder:<type>" \
  --parent <map-number> \
  --body "$(printf '## Question\n\n<the decision this ticket resolves>')"
```

`--parent` (verified present in gh 2.97.0) creates the ticket already
attached as a sub-issue of the map in one operation, so a mid-chart crash
never leaves an unparented orphan and there is no separate hierarchy pass.
Exactly one `wayfinder:*` label per issue; never a creation issue type or a
severity (policy contract). Before any retry after an unknown result, list
the map's `subIssues` and skip a ticket whose title already exists.

## Node IDs (needed by the GraphQL mutations)

```sh
gh api graphql -f query='query($o:String!,$r:String!,$n:Int!){
  repository(owner:$o,name:$r){issue(number:$n){id}}}' \
  -f o=<owner> -f r=<repo> -F n=<number> \
  --jq '.data.repository.issue.id'
```

## Wire blocking (second pass, after all tickets exist)

Read each ticket's existing `blockedBy{nodes{number}}` first and add only
the missing edges, so a retried pass creates no duplicates:

```sh
gh api graphql -f query='mutation($i:ID!,$b:ID!){
  addBlockedBy(input:{issueId:$i,blockingIssueId:$b}){issue{number}}}' \
  -f i=<blocked-ticket-id> -f b=<blocker-ticket-id>
```

The blocking input field is `blockingIssueId` (verified live; it is not
`blockedByIssueId`). GitHub caps sub-issues at 100 children per parent and
8 nesting levels; a map approaching the cap is a mis-scoped destination.

## Frontier query

Open, unblocked, unclaimed children of the map:

```sh
gh api graphql -f query='query($o:String!,$r:String!,$n:Int!){
  repository(owner:$o,name:$r){issue(number:$n){subIssues(first:100){
    nodes{number state assignees(first:1){totalCount}
      blockedBy(first:100){totalCount nodes{state}}
      comments(first:100){nodes{body}}}}}}}' \
  -f o=<owner> -f r=<repo> -F n=<map-number> \
  --jq '[.data.repository.issue.subIssues.nodes[]
    | select(.state=="OPEN" and .assignees.totalCount==0
      and .blockedBy.totalCount<=100
      and ([.blockedBy.nodes[]|select(.state=="OPEN")]|length)==0
      and ([.comments.nodes[].body
              | capture("^(?<kind>claim|claim-released|claim-yielded): session (?<id>\\S+)")]
            | group_by(.id)
            | map(last.kind)
            | index("claim")) == null)
    | .number]'
```

Verified semantics: a claimed ticket leaves the frontier; closing a blocker
promotes its dependents into it. `blockedBy` is fetched to GitHub's page
maximum and guarded by `totalCount`, so a ticket with more blockers than
one page is never misclassified as ready — it is simply excluded until
inspected by hand (a ticket with 100+ blockers is a mis-charted map). Past
100 children, page the `subIssues` connection with `after:`. The claim test
is per session and chronological: comments arrive in creation order, so
for each session id it takes that session's **latest** claim-protocol
event; the ticket is unclaimed only when no session's latest event is a
`claim`. A renewed claim (`claim A`, `claim-released A`, `claim A`) is
therefore active again, which a set-difference test would miss. Fetch
comments to the page maximum and page with `after:` past 100.

## Claim (first, before any work)

Claims are settled by comments, not by the assignee, because concurrent
sessions usually share one GitHub identity. Every claim-protocol comment
names its own session id, so the active set is computable per session.
Protocol, in this order:

1. Post the claim comment first:
   `gh issue comment <number> --body "claim: session <session-id> <ISO-8601 UTC>"`.
2. Re-read the ticket's comments. A session's claim is **active** when its
   `claim: session <id>` comment has no later `claim-released: session
   <id> ...` or `claim-yielded: session <id> ...` comment naming that same
   id. The winner is the active claim with the earliest server-assigned
   `createdAt`; on an equal timestamp, the lexicographically smaller
   comment node id wins.
3. If you won, assign the ticket (`gh issue edit <number> --add-assignee
   "@me"`) and start work. If not, comment
   `claim-yielded: session <your-id> to <winning-id>` and pick another
   frontier ticket.

Assignment is therefore a consequence of winning, never the claim itself,
so a crash after assignment leaves an active claim comment with a
timestamp, and a released claim drops out of the active set instead of
winning forever. A claim older than 24 hours with no resolution is stale
— any session may release it with `claim-released: session <stale-id>
stale`, remove the assignee, and re-claim through the same protocol. The
frontier query treats a ticket as claimed when any session's claim is
active or an assignee is set.

## Resolve

```sh
gh issue comment <number> --body "<resolution: the answer, with citations
(source URL, observed date, confidence) for research tickets>"
gh issue close <number> --reason completed
```

The resolution comment is the answer's one home. Wayfinder owns no branch:
an oversized research or task artifact is attached to the issue (GitHub
issue attachments) or kept in the OS temp directory and summarized in the
comment — never pasted into the map, and never committed to a branch. The
only branch writes in wayfinder belong to prototype tickets, which follow
the prototype route's own branch identity and retention rules.

## Append to Decisions so far

Decisions-so-far is derived, not maintained. The map body's `## Decisions
so far` section holds only the sentence "Derived from closed child tickets
— see each ticket's resolution comment." The index itself is computed on
demand from the closed children, so concurrent sessions never race on a
shared mutable body:

```sh
gh api graphql -f query='query($o:String!,$r:String!,$n:Int!){
  repository(owner:$o,name:$r){issue(number:$n){subIssues(first:100){
    nodes{number title url state closedAt
      comments(first:100){nodes{body}}}}}}}' \
  -f o=<owner> -f r=<repo> -F n=<map-number> \
  --jq '[.data.repository.issue.subIssues.nodes[]
    | select(.state=="CLOSED")
    | . as $t
    | ([$t.comments.nodes[].body | select(test("^(decision|fact): "))] | first) as $res
    | select($res != null)
    | select(([$t.comments.nodes[].body | select(test("^(superseded|out-of-scope): "))] | length) == 0)
    | {title: $t.title, url: $t.url, closedAt: $t.closedAt, gist: $res}]
    | sort_by(.closedAt)'
```

Each resolution comment starts with its kind and gist on the first line
— `decision: <one-line gist>` or `fact: <one-line gist>` — and the index
selects the first comment matching that prefix, so later discussion never
displaces the resolution. A closed ticket with no such comment (closed by
mistake) is excluded rather than indexed, and a ticket carrying a
`superseded:` or `out-of-scope:` comment is excluded even if it was
resolved earlier — a superseded decision leaves the index the moment it
is superseded. The derived index
reads exactly like Matt's hand-written one, and a session loading the map
at low resolution renders it in a single call.

The map body is therefore mostly static. Its only mutable sections are
**Not yet specified** and **Out of scope**, and those are edited only by
the session that holds the **map claim**: before editing the map body,
run the claim protocol above against the map issue itself, edit while
holding it, and release it (`claim-released: session <id> done`) immediately after the
write. Serializing map-body edits through that claim is what makes the
read-modify-write safe, since GitHub offers no write precondition:

```sh
gh issue view <map-number> --json body --jq .body > <tmp-body>
# edit <tmp-body>, then:
gh issue edit <map-number> --body-file <tmp-body>
```

Refer to tickets by name with the link riding inside it, never by bare
number.
