# Screen recording script

One silent screen recording, about 3 minutes 30 seconds of footage. I add
the title card, the callouts, the lower thirds, the decision cards, and the
music in the edit. You do not talk.

## Before you press record

1. Build and launch the signed app:
   `APPLE_SIGNING_IDENTITY="Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)" npm run tauri build`,
   then open `src-tauri/target/release/bundle/macos/workflow-step-editor.app`.
   Confirm all three permission pills are green.
2. Have at least two saved workflows in the list already so the landing page
   is not empty. Delete any test junk you do not want on camera ("Typing
   Still Bugged" and "First Character Not Showing" should go).
3. Open TextEdit with one blank document, and open a browser tab on
   `https://github.com/dpearson2699/workflow-step-editor` (repo root, then
   you will visit Pull requests, Issues, and one docs folder).
4. Turn on Do Not Disturb. Hide the desktop clutter. Set the app window
   large enough that the step list and the detail pane both read clearly
   (about 1400 x 900 points is good). Do not resize it during the take.
5. Recording: press `Cmd+Shift+5`, choose **Record Entire Screen** (you
   switch apps during the take, so a window recording will not work),
   Options → Microphone **None**, Save to `~/Movies`. Timer off is fine.
6. Move the mouse slowly and pause about 2 seconds wherever the script says
   **hold**. The holds are where the callouts land.

If you fumble, keep going or restart. One clean take is easiest for me,
but multiple takes are fine; tell me which files are which.

## The take

Segment times are targets, not limits.

### 1. Landing page (0:00–0:15)

- Start on the workflow list. **Hold** 3 s so I can title-card over it.
- Move the pointer along the permission strip in the header, left to
  right. **Hold** on the third pill 2 s.
- Hover one saved workflow row so **⌘ Reveal** appears. **Hold** 2 s.

### 2. Record a workflow (0:15–1:05)

- Click **● Record New Workflow**. **Hold** 2 s on the red Stop banner.
- Switch to TextEdit. Click into the document. Type `hello` at a normal
  pace. **Hold** 2 s.
- Open the **Format** menu, then click **Font → Show Fonts**. **Hold** 2 s
  on the Fonts panel, then close it.
- Press `Cmd+S`, then press `Escape` to cancel the save sheet.
- Switch back to the recorder. Watch the step rows for 3 s. **Hold**.
- Click **■ Stop Recording**.

### 3. Draft review (1:05–2:20)

- The editor opens in draft mode. **Hold** 3 s on the whole view.
- Click the first `type` step (the `h` press). **Hold** 3 s. Then click
  the window-crop thumbnail, **hold** 2 s, click the element-crop
  thumbnail, **hold** 2 s.
- Click the `click` step for the Format menu. **Hold** 3 s so the
  screenshot with the open menu is visible.
- Click the `Cmd+S` step. **Hold** 2 s.
- Click the step for the Escape key. Change its classification to
  **wait**. **Hold** 2 s.
- Click any step. Click into the title, change it to something short (for
  example `Type hello into TextEdit`), click into the description, type
  one sentence. **Hold** 2 s.
- Delete one noise step (a click you did not mean, or the Escape step) and
  confirm. **Hold** 2 s.
- Click **Save…**. The name field is preselected. Type `TextEdit demo`
  and save. **Hold** 2 s on the saved view.

### 4. Library and disk (2:20–2:50)

- Click **‹ Workflows**. **Hold** 2 s on the list with the new row at the
  top.
- Hover the new row and click **⌘ Reveal**. When Finder opens, click into
  the folder so `workflow.json`, `events.jsonl`, and `shots/` are visible.
  Open `shots/` and **hold** 3 s. Go back and switch to the recorder.
- Click the row to reopen it. Rename it in the header to
  `TextEdit demo (renamed)`. **Hold** 2 s.

### 5. The process (2:50–3:30)

- Switch to the browser on the repo root. Scroll slowly down the README
  once (about 8 s). **Hold** on the "How this was built" heading 2 s.
- Click **Pull requests**, filter **Closed** so the merged list shows.
  **Hold** 3 s.
- Click **Issues**. **Hold** 3 s on the open list.
- Open `docs/features/completed/2026-08-17-capture-pipeline-and-backend-foundation/`
  and click `DECISIONS.md`. Scroll slowly for 5 s. **Hold** 2 s.
- Stop the recording (`Cmd+Ctrl+Esc` or the stop button in the menu bar).

## After

Put the file(s) at `~/Movies/wse-walkthrough/` and tell me the names. I
take it from there.
