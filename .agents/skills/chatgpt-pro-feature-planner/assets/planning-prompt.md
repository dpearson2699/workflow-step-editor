# ChatGPT work-planning prompt template

Replace every angle-bracket placeholder. Remove empty optional sections. Keep the
stable instructions above the variable repository and work context.

```text
Act as the primary repository researcher and architectural planner for one
work bundle in the named repository. Produce evidence-grounded candidate architecture for
the spec-work root to synthesize, not implementation or semantic authority. Be critical about
architecture, slice boundaries, acceptance coverage, concurrency, persistence,
migrations, tests, and UI proof where relevant.

Planning identity

- Work ID: <WORK_ID>
- Work bundle: <WORK_BUNDLE_PATH>
- Specification digest: <SPECIFICATION_DIGEST>
- Pass responsibility: primary research and planning

Repository evidence contract

- Use the selected GitHub app to inspect only repository: <OWNER/REPO>
- Branch provenance: <BRANCH>
- Exact retrieval target: commit <REMOTE_COMMIT_SHA>
- Coordinator provenance digest: <PROVENANCE_DIGEST>
- Approved source-inventory digest: <SOURCE_INVENTORY_DIGEST>
- The spec-work root already verified with Git that the named remote branch points to this
  commit and that every requested committed path is present. Treat that deterministic
  branch-to-commit binding as supplied provenance, not as a GitHub-app research task.
- Retrieve repository content by the exact commit. Do not require the GitHub app to
  return or independently prove branch metadata when the exact commit and paths are
  accessible.
- Treat this commit as the source checkpoint for the specification digest above.
- Do not silently substitute the default branch, another commit, memory, or chat history.
- Do not use or request any prior ChatGPT Pro planning answer; evaluate only the source
  inventory in this prompt.
- If the exact commit or any requested path is unavailable, stop repository analysis
  and say so. Inability to enumerate the branch alone is not a blocker.
- Read these exact relative paths first:
<GITHUB_PATH_LIST>
- Follow repository references only when needed to answer the planning question. Cite
  every additional path you inspect.

Accepted work goal

<WORK_GOAL>

Accepted decision and acceptance anchors

<DECISION_AND_ACCEPTANCE_IDS>

Local-only context

The blocks below were supplied from the local worktree and are not GitHub evidence.
Treat each block as applying over base commit <LOCAL_BASE_COMMIT>. Cite it as
`local-only:<PATH_OR_DIFF_LABEL>` and never imply that GitHub retrieved it.

<LOCAL_ONLY_BLOCKS_OR_NONE>

Suggested organization

Use the headings below for readability when they fit. They are not a response
schema: introductory or concluding prose, additional headings, tables, different
heading depths, or an equivalent organization are acceptable when the response
still provides substantive planning analysis.

## Evidence map

Map every material claim to a repository-relative path and a symbol, Markdown heading,
or line span when available. Mark local-only evidence distinctly.

## Assumptions

Use three explicit subsections: Verified, Inferred, and Unknown. Explain what would
invalidate every inferred assumption.

## Architecture

Describe owning seams, data/control flow, persistence and concurrency boundaries,
external dependencies, and compatibility or migration obligations. Call out conflicts
with accepted decisions or acceptance criteria instead of silently resolving them.

## Reviewable delivery slices

Propose the smallest practical vertical slices. For each slice include outcome, owned
behavior and paths, dependencies, acceptance IDs covered, verification, UI/live-data
gates, and non-goals. Do not invent implementation details for files you did not inspect.

## Risks and missing evidence

Prioritize failure modes, ambiguous decisions, unavailable paths, and evidence that the
spec-work root must gather before delivery.

## Recommendation

Give one recommended plan and explain rejected alternatives. Keep product decisions with
the spec-work root; identify any consequential unresolved question explicitly.
```
