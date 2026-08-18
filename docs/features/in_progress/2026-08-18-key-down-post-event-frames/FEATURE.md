# Capture timing: post-event screenshot frames for key-down steps

## Source / Issue

https://github.com/dpearson2699/workflow-step-editor/issues/38

## Goal

Make screenshot frame selection per event kind so that a typing step's
screenshot triple shows the just-typed character while a click step's
triple keeps showing the control as it looked at click time.

## Scope

- Key-down events select the first buffered frame whose display timestamp
  is after the event, instead of the latest pre-event frame. One frame
  interval (~100 ms) of added latency is the expected cost; the wait is
  bounded at 250 ms after the event (DEC-002), which is the single accepted
  latency limit. Operationally this is the oldest retained frame on the
  selected display inside the bounded window `(event_ts, event_ts + 250 ms]`
  (the broker retains two frames per display).
- Click events keep the current pre-event selection byte-for-byte
  (`RetainedFrames::eligible` unchanged, including its bounded-retention
  approximation; ADR-0001 rationale stands: the artifact must show the
  control before the UI repaints).
- The bounded wait for a post-event frame runs on the capture worker,
  never in the tap callback (DEC-009: the tap never blocks). Its deadline
  is anchored to the event timestamp so a burst of key-downs on a static
  screen shares one wait instead of stacking waits (see DEC-002).
- When no in-window post-event frame exists (none arrived before the
  deadline, the display retains no live frame, or the candidate frame's
  display geometry differs from the event snapshot), the key-down uses its
  pinned pre-event frame (DEC-002, GA-006).
- Orderly pipeline stop joins the capture worker before it stops the
  display streams, so an accepted key-down can finish its bounded wait
  (DEC-002).
- `frame_age_ms` keeps its `u64` schema type; a post-event frame reports
  `0` and the field's doc comments say so (GA-003).
- Amend `docs/adr/0001-pre-buffered-screen-capture.md` to record the
  per-kind timing rule; update the README sentence that says every
  screenshot precedes the action.

## Non-Goals

- Changing the continuous per-display SCStream pre-buffer architecture,
  the stream frame rate, or the retained-frame depth beyond what the
  post-event selection needs.
- Capturing both a pre- and a post-event frame per event (rejected in the
  issue for storage and viewer cost).
- Post-event frames for clicks or any event kind other than key-down.
- Any change to key-event element crops (focused text area or fixed-size
  fallback); the issue routes that observation separately.
- Schema v2, a signed frame-offset field, or any `events.jsonl` change.
- UI changes: the review UI renders the stored PNGs unchanged.

## Doc Authority

| Subject | Current authority | Conflict or obligation | Owning update |
| --- | --- | --- | --- |
| Pre-event frame capture rationale | `docs/adr/0001-pre-buffered-screen-capture.md` | Amend for per-kind timing (issue #38 scope) | `docs/adr/0001-pre-buffered-screen-capture.md` |
| Frame selection and pinning | `src-tauri/src/capture/broker.rs`, `src-tauri/src/capture/pipeline.rs` | Behavior change for key-down | none (code) |
| Key-down display selection | DEC-008 of bundle `2026-08-17-capture-pipeline-and-backend-foundation`; `src-tauri/src/capture/packets.rs` | none; unchanged | none |
| Tap never blocks; saturation fail-stops | DEC-009 of the foundation bundle; `src-tauri/src/capture/queue.rs` | Constrains where the post-event wait runs | none |
| `frame_age_ms` meaning | `src-tauri/src/domain/schema.rs`, `src-tauri/src/recording/pipeline.rs` doc comments | Doc comment obligation | those doc comments |
| User-facing capture description | `README.md` ("captured from a pre-event frame ...") | Sentence becomes wrong for typing | `README.md` |
| Vocabulary | `CONTEXT.md` | none | none |

## Open decision IDs

- none

## Codex Task Roster

- Status: registered
- Entry: PR-01 | implementation | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision
- Entry: PR-01 | review | claude-fable-5 high | unrecoverable_task_runtime, unrecoverable_worktree, repository_identity_mismatch, pr_identity_unrecoverable, separate_deliverable_user_decision

## Durable Sources

- `docs/PROJECT_GOAL.md`
- `CONTEXT.md`
- `docs/adr/0001-pre-buffered-screen-capture.md`
- `src-tauri/src/capture/broker.rs`, `src-tauri/src/capture/pipeline.rs`,
  `src-tauri/src/capture/queue.rs`, `src-tauri/src/capture/worker.rs`,
  `src-tauri/src/capture/packets.rs`, `src-tauri/src/capture/streams.rs`,
  `src-tauri/src/capture/macos/stream.rs`
- `docs/features/completed/2026-08-17-capture-pipeline-and-backend-foundation/DECISIONS.md`
  (DEC-008 display selection, DEC-009 saturation policy)
- `docs/features/completed/2026-08-17-capture-pipeline-and-backend-foundation/review/proven-gate-run.md`
  (real-recording acceptance precedent)
