# Resolving Merge Conflicts

1. **See the current state** of the merge/rebase. Check git history, and the
   conflicting files.

2. **Find the primary sources** for each conflict. Understand deeply why each
   change was made, and what the original intent was. Read the commit
   messages, check the PRs, check original issues/tickets.

3. **Resolve each hunk.** Preserve both intents where possible. Where
   incompatible, pick the one matching the merge's stated goal and note the
   trade-off. Do **not** invent new behaviour. Always resolve; never
   `--abort`.

4. Discover the project's **automated checks** and run them — typically
   typecheck, then tests, then format. Fix anything the merge broke.

5. **Finish the merge/rebase.** Stage everything and commit. If rebasing,
   continue the rebase process until all commits are rebased.

## Workflow binding

Bounded conflict resolution inside the active route stays with the owning
task or coordinator, and only within that owner's owned paths. A conflict
hunk in a file outside the slice's owned paths is scope expansion: do not
resolve it — stop, surface it, and let the route decide. When a conflict
reveals a material plan change — incompatible accepted decisions, a severed
production path, or scope beyond the accepted deliverable — do not resolve
through it: stop, surface the conflict, and re-enter Plan through the
route. Aborting the mechanical merge to re-plan is the single exception to
never `--abort`. "Stage everything" means every resolved conflict hunk in
owned paths — never unrelated working-copy changes or user edits, which
stay untouched per the dirty-worktree rule.
