# Review UI gate — final human script (AC-001, bundle 2026-08-17-review-ui)

This is the user-run final design acceptance for the review UI bundle
(`docs/features/in_progress/2026-08-17-review-ui`, policy
`final_pr_design_gate`). It runs on the locally built, signed `.app`
from the exact final code head of PR-03, because macOS TCC binds
permission grants to the signed bundle identity. The implementer readies
this script; the user runs the loop and judges the result against the
pinned prototype direction (`prototype/map-1-8`, variant D).

## Prerequisites

1. Build the signed app from the final code head:
   ```
   APPLE_SIGNING_IDENTITY="Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)" \
     npm run tauri build
   ```
   The bundle identifier must stay `com.dpearson.workflow-step-editor`
   so existing TCC grants persist across rebuilds.
2. Launch
   `src-tauri/target/release/bundle/macos/workflow-step-editor.app`.
   Launch the `.app`, not `cargo run` / `npm run tauri dev`: the dev
   binary is a different TCC identity.
3. In the header permission strip, confirm all three pills read granted
   (✓ Input Monitoring, ✓ Accessibility, ✓ Screen Recording). If one is
   missing, click its pill and grant it in System Settings — Input
   Monitoring first, then Accessibility, then Screen Recording.
4. Have two target apps ready: **TextEdit** with one open document, and
   **Google Chrome** with any page.

## Sequence

1. **Record across two apps.** Click **● Record New Workflow**. The
   live capture view opens; the prominent red **■ Stop Recording**
   banner is the only visible action. Click a control and type a short
   phrase in TextEdit, then click and type in Chrome. Watch each step
   appear in the list as a compact row — index, classification dot,
   auto-title, event time — in capture order.
2. **Stop into draft review.** Click **■ Stop Recording**. The full
   detail view opens with a `draft` badge beside the default timestamp
   name, and **Discard** / **Save…** in the header. Select a few steps;
   confirm each shows its screenshot triple (one large, two labeled
   click-to-swap thumbnails) and the metadata grid.
3. **Save with a name.** Click **Save…**. The naming dialog pre-selects
   the default timestamp name. Type a real name (for example
   `Approve invoice`) and click **Save**. The draft badge and draft
   actions disappear; the header shows the new name.
4. **Reopen from the landing list.** Click **‹ Workflows**. The saved
   workflow appears as a row — thumbnail, name,
   `date · step count · duration`, newest first. Click the row to
   reopen it.
5. **Edit.** Change one step's title, add a description, and switch a
   classification through the four-value dropdown. Confirm the edits
   save automatically (no explicit save control) and survive leaving
   and reopening the workflow.
6. **Delete a step.** Hover a step row and click its ✕. The step leaves
   the list; the remaining steps keep their order.
7. **Delete a saved workflow.** Click **Delete…** in the header. The
   destructive confirmation names the keystroke data and defaults to
   Cancel. Confirm the deletion; the app returns to the landing page
   and the row is gone.
8. **Reveal in Finder.** Hover another workflow row (record a short one
   first if none is left) and click **⌘ Reveal**. Finder opens with the
   workflow folder selected.

## Pass criteria

The gate passes when the user confirms every numbered outcome above and
judges that the UI matches the pinned variant D prototype direction:
live rows stream during capture with Stop as the sole action, draft
review is the full editor behind the `draft` badge, naming is the save
ceremony, and the landing list, detail view, deletion, and reveal behave
as scripted. Record the result as the typed UI receipt and attestation
required by the bundle's `final_pr_design_gate` policy.
