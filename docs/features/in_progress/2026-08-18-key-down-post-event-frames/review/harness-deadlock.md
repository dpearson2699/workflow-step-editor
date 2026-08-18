# Harness deadlock record — 2026-08-18

`BLK-HARNESS-WAIVER-LINEAGE-LOST` (bundle revision 23).

Sequence: the AC-001 gate failed on PR-01 (PR #39) before the final-PR
snapshot; the accepted rule changed (DEC-004); the user waived a fresh Pro
primary (DEC-005). `adopt-plan` under the waiver succeeded (revision 18) and
advanced the specification binding to `46afa6…`/`797fa56`. Superseding
PR-01's task then required a Discuss hop (the waiver binds the blind receipt
to the task-state projection), and re-entering Plan cleared the one-shot
waiver. With the binding already advanced past the last material
applicability target (`817a99…`/`5438840`), no `adopt-plan` waiver candidate
or new applicability receipt is admissible, and Plan-to-Delivery demands a
fresh Pro primary the user has explicitly declined.

Decision (user, 2026-08-18): finish the MVP without further Pro rounds. The
remaining delivery of PR-02 (implementation, user gate, review, merge, close
#38) proceeds as ordinary harness-native worktree tasks outside the bundle's
task-state machine; this record and the blocker keep the bundle truthful.
The harness gap is filed as a `harness` issue.
