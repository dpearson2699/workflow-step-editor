# Acceptance

## AC-001: Full record-review-save-delete loop on a real recording

- Ownership: slice
- Invariant: With the dev build, a user can record across two apps while
  steps stream live, stop, review the draft, save it with a name, reopen it
  from the landing list, edit titles, descriptions, and classifications,
  delete a step, delete a saved workflow through the confirmation, and
  reveal a workflow in Finder — and the UI matches the pinned prototype
  direction (`prototype/map-1-8` variant D).
- Owning seam: The running app end to end — capture channel, storage, and
  the review UI together.
- Evidence required: User-run final design acceptance under
  `final_pr_design_gate` with the typed UI receipt and attestation.

## AC-002: Landing page lists, gates, and navigates

- Ownership: slice
- Invariant: The app opens on the workflow list: one row per saved workflow
  showing thumbnail, name, and `date · step count · duration`; a permission
  status strip; a Record button disabled until all three permissions are
  granted, with an explanatory hint; hover Reveal-in-Finder per row; row
  navigation into the detail view and `‹ Workflows` back.
- Owning seam: The rendered landing view over the real `list_workflows`
  summary data.
- Evidence required: Frontend tests over the landing view plus automated UI
  proof from the repository-native UI route.

## AC-003: Detail view reviews and edits a saved workflow

- Ownership: slice
- Invariant: The detail view shows the compact text-only step list and a
  detail pane where all three screenshots stay visible (one large, two
  labeled click-to-swap thumbnails); title and description edits and the
  four-value classification dropdown persist without an explicit save;
  step deletion removes the step while its events stay in the log; the
  metadata grid shows time, app/window, coordinates, key, and element with
  source; the workflow can be renamed.
- Owning seam: The review UI over the production `get_workflow`,
  `update_step`, `delete_step`, and `rename_workflow` commands and the
  stored artifacts they mutate.
- Evidence required: Rust command/store tests for the mutations; frontend
  tests for the editing flows; automated UI proof.

## AC-004: Record flow streams live and lands in draft review

- Ownership: slice
- Invariant: Record enters a live capture view whose only visible action is
  a prominent Stop Recording banner while step rows stream in over the
  capture channel; Stop enters draft review with a `draft` badge, full
  editing, Discard behind a confirmation, and Save…; Save opens a naming
  dialog with a pre-selected timestamp default and saving names the
  workflow; Discard removes the draft folder and returns to the landing
  page. A recording that ends in failure enters draft review behind an
  error banner when its workflow still loads, and surfaces the error on
  the landing page when it does not; a failed Save or Discard keeps the
  draft state and shows the error instead of exiting draft.
- Owning seam: The review UI over the production `start_recording` channel,
  `stop_recording`, `rename_workflow`, and the shared folder-removal
  primitive.
- Evidence required: Frontend tests over the record-flow states; automated
  UI proof.

## AC-005: Saved-workflow deletion is a validated backend hard delete

- Ownership: slice
- Invariant: Deleting a saved workflow presents a destructive confirmation
  naming the keystroke data with Cancel as default; the backend resolves
  the id inside the workflow root, refuses targets that escape the root or
  follow a substituted link, and removes the directory; the UI removes the
  row only after backend success and surfaces failures; an already-missing
  directory counts as deleted and refreshes the list; no tombstone, trash,
  or audit copy is written.
- Owning seam: The `delete_workflow` command and the store's removal
  primitive over the real filesystem root.
- Evidence required: Rust store/command tests covering the validation and
  missing-directory invariants; frontend confirmation-flow tests. The
  deletion flow is also exercised live inside AC-001's final human loop.
