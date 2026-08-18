# AC-005 verification evidence — final head (root-observed, 2026-08-18)

Producing slice: PR-02, final reviewed head
`545e1b616aaded4a748a6543f0f6e10f47d3b350`, worktree tree
`0cc4de640003b5c997cd9bd0cae27ec790e4d8e4`.

All AC-005 invariants from the initial evidence
(`ac-005-verification.md`, head `5e665b1a`) re-verified at the final
head after the review's two remediation batches, which strengthened this
exact surface: absence proof now accepts only `ErrorKind::NotFound` (a
stat failure is a removal failure, never success), `require_real_dir`
re-validation guards the manifest restore, the Escape key is ignored
while a deletion is in flight so failures always render, and a failed
deletion re-arms queued autosaves serialized behind the orphaned
in-flight save.

Suites at the final head: `cargo test` (src-tauri) 149/149 PASS;
`npx vitest run` 41/41 PASS (12 review-added regression tests);
`npm run build` PASS. Exact-head review verdict CLEAN
(`review-terminal.json`); final Codex pass at this head reported no
major issues.

The live deletion flow is additionally exercised inside AC-001's final
human loop on the PR-03 gate.
