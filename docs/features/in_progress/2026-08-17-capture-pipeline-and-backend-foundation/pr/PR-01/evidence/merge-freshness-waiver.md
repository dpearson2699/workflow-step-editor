# PR-01 merge freshness waiver (operator decision, 2026-08-17)

After the review finalized CLEAN at head 3b0d0b0c (base 3aa43fe4), remote
main advanced by one docs-only harness commit (029bbe6c, touching only
.agents/workflows/). The operator explicitly waived the advanced-main
base-sync re-review round ("I don't need a clean review. Let's get this
merged in.") and directed immediate merge. The merge executes through the
deterministic provider with an exact compare-and-swap against the true
current base 029bbe6c and reviewed head 3b0d0b0c; no observation was
falsified. The waived step is recorded here in place of the standard
pending-merge precondition.
