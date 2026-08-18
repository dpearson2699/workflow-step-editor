# AC-005 verification evidence (root-observed, 2026-08-17)

Producing slice: PR-02, head `5e665b1a5b511ed2f098f73f30757604ded17fe5`,
worktree tree `c50b8f869823b8e11ce6e1e332c4d0b3b827172b`.

Backend (`cargo test` in `src-tauri`, 146/146 PASS at the head above)
covering the AC-005 invariants:

- Removal primitive succeeds only when the workflow directory is absent;
  an injected remnant-leaving failure restores `workflow.json` from the
  cached bytes, reports failure, and the workflow stays listed; a retried
  delete completes.
- Missing directory or missing root counts as success; no tombstone,
  trash, or audit artifact is written.
- Symlinked and non-directory targets are refused; id validation and
  root confinement reuse the store's existing checks; deletion goes
  through one `std::fs::remove_dir_all` call (descriptor-relative,
  no per-child path traversal).
- Deleting the active or stopping workflow is rejected (DEC-008 guard);
  deleting one workflow while another records is safe.

Frontend (`npx vitest run`, 32/32 PASS at the head above) covering the
confirmation flow:

- The single detail-header Delete… control opens a destructive
  confirmation naming the keystroke data with Cancel as the default
  action.
- The row disappears only after backend success; a backend failure keeps
  the workflow visible and surfaces the error; a missing directory counts
  as deleted and refreshes the list.
- The workflow's autosave generation is invalidated before deletion, so
  stale queued completions from the removed workflow are ignored.

The live deletion flow is additionally exercised inside AC-001's final
human loop on the PR-03 gate.
