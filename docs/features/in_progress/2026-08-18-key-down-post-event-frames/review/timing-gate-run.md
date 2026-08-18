# AC-001 timing-gate runs — 2026-08-18

User-run gates on the locally built signed app
(`APPLE_SIGNING_IDENTITY="Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)"
npm run tauri build`, identifier `com.dpearson.workflow-step-editor`,
built by the root in a detached checkout of each exact head under
`.claude/worktrees/gate-pr01`); the user drove the recordings in TextEdit
and reviewed the steps in the app; the root spot-checked artifacts on disk.

## Run 1 — PR #39 head `176be565` (PR-01, oldest in-window frame): FAIL

- Workflow `2026-08-18-155755-1d2a` (17 events, 51 shots).
- `evt_0002` (`T`, first key, `frame_age_ms` 0) shows the window title
  already `— Edited` but no glyph; `evt_0003` shows `Te`. Root cause: the
  first keystroke's title-only repaint precedes the glyph paint (GA-007).
- Later keys correct. Verdict (user): issue found. → Q-003 / DEC-004.

## Run 2 — PR #41 head `cf29cfc4` (settle 100 ms, newest in-window): FAIL

- Workflow `2026-08-18-182339-6da6` ("Typing Still Bugged"; 19 events).
- First key correct; `evt_0005` (`e`) shows `Hel`; keys 92-180 ms apart, so
  a one-frame-interval settle lands on the frame after the next key
  (GA-009). Verdict (user): issue found. → Q-005 / DEC-006.

## Run 3 — PR #41 head `63fada66` (DEC-006 content-aware): PASS with waiver

- Workflow `2026-08-18-184540-3aaf` ("First Character Not Showing";
  19 events, 57 shots = 3 per event; keys `H e l l o ␠ w o r k ⌫ l d ! ⌫ ! ␠`
  at ~90-100 ms spacing during bursts; all key events `frame_age_ms` 0).
- User report: every typed step is in sync with its own character except
  the FIRST character of the recording, which does not show in its step.
- User decision (2026-08-18): "if that's a tradeoff to make every other
  character be in sync then I authorize that tradeoff." AC-001 accepted
  with the first-keystroke case explicitly waived by the user.
- Root note on the residual: the first key's element-crop change most
  plausibly comes from the caret state toggling between the pinned frame
  and the first post-event frame (DEC-006 residual risk); it does not
  affect later keys because typing suspends the caret blink. Recorded as
  known-limitation follow-up.

Verdict: AC-001 PASS (user), first-keystroke residual waived by the user.

## Follow-up publication receipt (first-keystroke residual)

- issue_url: https://github.com/dpearson2699/workflow-step-editor/issues/43
- verified_state: OPEN | fingerprint: 590ff7206fba07d1884c28eb5997320b301f8e2e2193d39e36569581f2e94141
- fingerprint_comparison: exact (no prior owner) | issue_type: bug | severity: P3
- expected_labels: [bug, P3] | disposition: created | label_verification_status: verified
