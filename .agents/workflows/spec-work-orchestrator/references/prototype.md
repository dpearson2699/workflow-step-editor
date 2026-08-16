# Prototype

A prototype is **throwaway code that answers a question**. The question decides
the shape.

## Pick a branch

Identify which question is being answered — from the user's prompt, the
surrounding code, or by asking if the user is around:

- **"Does this logic / state model feel right?"** →
  [prototype-logic.md](./prototype-logic.md). Build a single shareable HTML
  file — free-play buttons plus tabbed guided walkthroughs — that pushes the
  state machine through cases that are hard to reason about on paper, and that
  a non-developer can drive.
- **"What should this look like?"** → [prototype-ui.md](./prototype-ui.md).
  Generate several radically different UI variations on a single route,
  switchable via a URL search param and a floating bottom bar.

The two branches produce very different artifacts — getting this wrong wastes
the whole prototype. If the question is genuinely ambiguous and the user isn't
reachable, default to whichever branch better matches the surrounding code (a
backend module → logic; a page or component → UI) and state the assumption at
the top of the prototype.

## Rules that apply to both

1. **Throwaway from day one, and clearly marked as such.** Locate the
   prototype code close to where it will actually be used (next to the module
   or page it's prototyping for) so context is obvious — but name it so a
   casual reader can see it's a prototype, not production. For throwaway UI
   routes, obey whatever routing convention the project already uses; don't
   invent a new top-level structure.
2. **Trivial to run.** A UI prototype starts from one command in the project's
   task runner — `pnpm <name>`, `python <path>`, `bun <path>`, etc. A logic
   demo is a single HTML file the user double-clicks. Either way, no thinking
   required to start it.
3. **No persistence by default.** State lives in memory. Persistence is the
   thing the prototype is _checking_, not something it should depend on. If the
   question explicitly involves a database, use an isolated local disposable
   store — an in-process or file-backed database inside the prototype
   worktree, or a local file — with a clear "PROTOTYPE — wipe me" name.
   Never point a prototype at a shared, remote, or production database;
   any external store mutation needs the user's explicit authority for that
   specific store, and it stays outside this route by default.
4. **Skip the polish.** No tests, no error handling beyond what makes the
   prototype _runnable_, no abstractions. The point is to learn something fast.
5. **Surface the state.** After every action (logic) or on every variant switch
   (UI), print or render the full relevant state so the user can see what
   changed.
6. **Capture it when done.** Record the validated decision, then capture
   the prototype itself as a **primary source**: commit it to a throwaway
   branch, out of main, push it, and leave a context pointer to that branch
   on the implementation issue. Capture the answer too — the verdict and
   the question it settled — in the issue or a commit. Folding the decision
   into real code is the later Delivery slice's job (see Workflow binding);
   the main branch receives nothing from the prototype session itself.

## Workflow binding

A prototype is Discuss and Plan evidence, never delivery. It never satisfies an
acceptance criterion or the UI gate, and it never enters a slice, the work
branch, or the bundle. In this workflow a prototype is built in a disposable
worktree on its own branch — `prototype/<work_id>-<slug>` inside a bundle,
`prototype/map-<map-number>-<ticket-number>` for a wayfinder ticket — never
in the coordinator checkout and never on the work branch. Inside that
worktree the prototype freely edits whatever paths it needs, including the
product path it is prototyping against; that is the point of the
disposable branch. What is forbidden is any prototype-driven change to the
coordinator checkout, `main`, or a Delivery work branch: "fold the winner
into the real code" is an approved Delivery slice's job, done later and by
its own rules, never the prototype session's. When the verdict is recorded,
push the branch and record its head SHA in the ledger entry (or the
wayfinder ticket's resolution comment); a pushed, SHA-pinned commit is the
primary source, and it is retained until the consuming lifecycle finishes —
"cleanup" means deleting the local worktree, never the pushed branch while
anything still cites it. The throwaway branch is still a citable primary
source:
the implementation slice may reference it and lift validated code from it,
rewritten to production standards (see the anti-pattern in
`prototype-ui.md`), so implementation starts from a spec plus real code, not a
spec alone. The branch itself is never merged. Record the verdict by closing
the owning `GA-*`/`Q-*` through the decision closure transaction in
`interview-and-doc-authority.md`; record the context pointer to the throwaway
branch in that ledger entry, and on the source issue when one exists.
