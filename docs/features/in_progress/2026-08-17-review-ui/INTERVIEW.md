# Interview

Zero user questions: the wayfinder decision tickets (#7, #8, #10, #11), the
owning issue #13, and the pinned prototype settle every consequential
decision. Every gray area below closed from those sources.

## GA-001: Draft persistence semantics

- Status: closed
- Kind: decision
- Uncertainty: Whether "draft" is a persisted storage state or a UI-session
  state, and what a crash or quit before Save leaves behind.
- Why it matters: Decides whether schema v1 needs a draft flag and what the
  landing list shows after a crash.
- Evidence inspected: issue #8 record flow ("because events stream to disk
  crash-safely, naming is the save ceremony and all later edits
  auto-save"); issue #7 decision 5 (name optional at start, timestamp
  default, editable afterward); `src-tauri/src/recording/store.rs` (folder
  and manifest exist from `create`).
- Confidence: high
- Question: none
- Closure: Draft is a UI-session state (DEC-005). The folder exists from
  recording start under the timestamp default name; Save renames and exits
  draft mode; Discard removes the folder; a crash before Save leaves the
  workflow listable under its default name. No schema change.

## GA-002: Shipped-versus-needed backend surface

- Status: closed
- Kind: fact
- Uncertainty: Which commands from the issue #7 surface the foundation
  bundle actually shipped.
- Why it matters: Fixes the backend delta this capability owns.
- Evidence inspected: `src-tauri/src/commands/mod.rs` (`start_recording`,
  `stop_recording`, `list_workflows`, `get_workflow`); `src/App.tsx`
  (`check_permissions`, `request_permission` live and invoked);
  `src-tauri/src/recording/store.rs` (`WorkflowStore`: `create`,
  `append_event`, `load`, `save_manifest`, `list` — no delete or rename).
- Confidence: high
- Question: none
- Closure: This capability adds `update_step`, `delete_step`,
  `rename_workflow`, `reveal_workflow`, `delete_workflow`, and the summary
  extension; issue #13 explicitly authorizes the delta.

## GA-003: Screenshot display path to the webview

- Status: closed
- Kind: fact
- Uncertainty: How the React frontend displays PNG files stored under the
  Tauri app-data workflow root.
- Why it matters: Every review screen depends on it.
- Evidence inspected: `src-tauri/tauri.conf.json` (no asset-protocol scope
  configured yet); Tauri v2 provides the asset protocol
  (`convertFileSrc` + scoped `assetProtocol`) and command-based bytes as
  standard options.
- Confidence: high
- Question: none
- Closure: Non-product-facing engineering choice; Plan selects the
  mechanism. Both options are additive and reversible.

## GA-004: Landing-row thumbnail and summary data

- Status: closed
- Kind: fact
- Uncertainty: `WorkflowSummary` carries only id, name, and created_at; the
  landing row needs thumbnail, step count, and duration.
- Why it matters: Landing page cannot render the pinned row format without
  a summary extension.
- Evidence inspected: `src-tauri/src/recording/store.rs` (summary shape);
  prototype `Home.tsx` (row renders thumbnail, `date · step count ·
  duration`).
- Confidence: high
- Question: none
- Closure: Additive summary extension computed from stored artifacts at
  list time; thumbnail source matches the pinned prototype direction. No
  schema change; details are coordinator-owned in Plan.
