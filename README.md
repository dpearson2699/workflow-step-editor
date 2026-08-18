# Workflow Step Editor

A Tauri v2 desktop application that records desktop workflow steps from real
clicks and key presses, then lets you review, edit, name, and manage the
recordings.

## Features

- **Live recording.** One Record button captures clicks and key presses
  across every application. Each captured event becomes a step that
  streams into the step list while you work, with a classification dot,
  an auto-generated title, and the event time. A prominent Stop
  Recording banner is the only action while recording. Interactions
  with the recorder's own window — including the Stop click itself —
  are excluded from the captured steps.
- **Draft review and save.** Stopping opens the full editor in draft
  mode. Review the captured steps, then Save… with a name (the dialog
  pre-selects the default timestamp name) or Discard the recording
  behind a confirmation. Events and screenshots are already on disk when
  you stop — naming finishes the save.
- **Step editor.** A compact step list beside a detail pane that keeps
  all three screenshots visible: one large, two labeled click-to-swap
  thumbnails (full screen, window crop, element crop). Click steps are
  cut from a pre-event frame, so the screen shows the state *before* the
  click. Typing steps use the newest retained frame captured within
  250 ms after the key, chosen after a 100 ms settle so the typed
  character is normally visible; when no such frame remains, the
  pre-event frame.
  Edit titles and descriptions, switch the classification
  (`click`/`type`/`wait`/`assert`), and delete steps. Edits save
  automatically.
- **Workflow library.** The landing page lists saved workflows —
  thumbnail, name, `date · step count · duration`, newest first — with
  hover Reveal-in-Finder, rename in the detail header, and hard deletion
  behind a destructive confirmation.
- **Permission-gated.** A header strip shows the three required macOS
  permissions; recording stays disabled until all three are granted.
- **Local-only.** Recordings live under
  `~/Library/Application Support/com.dpearson.workflow-step-editor/workflows/`
  as plain JSON plus PNG screenshots. No external services.

## Requirements

- macOS. The application is macOS-only; the capture backend uses CGEventTap,
  ScreenCaptureKit, and the Accessibility API.
- Node.js 22.22+, 24.15+, or 26+ and npm (the jsdom test harness sets
  this floor).
- Rust (stable) with Cargo.
- Xcode Command Line Tools.

## Setup

```sh
npm install
npm run tauri dev     # run the app in development mode
npm run tauri build   # build the signed release app
```

## Tests

```sh
npm test                       # frontend component tests (vitest + jsdom)
(cd src-tauri && cargo test)   # backend tests
```

The built app lands at
`src-tauri/target/release/bundle/macos/workflow-step-editor.app`.

## Code signing

Dev builds sign with a local Apple Development identity when the
`APPLE_SIGNING_IDENTITY` environment variable is set in the build
environment:

```sh
export APPLE_SIGNING_IDENTITY="Apple Development: dpearson2699@gmail.com (86K7G9BGZ7)"
```

A stable identity plus the fixed bundle identifier
(`com.dpearson.workflow-step-editor`) keeps macOS permission grants (TCC)
across rebuilds. A clone without that certificate still builds with the same
commands; the app is then unsigned.

## Permissions

The recorder needs three macOS permissions: Input Monitoring, Accessibility,
and Screen Recording. Input Monitoring must be requested first: an early
Accessibility check suppresses the Input Monitoring prompt. The backend
enforces this order and reports an out-of-order Accessibility request as
`blocked_by_prerequisite`.

Grant them from the header strip: click each red pill in order and approve
the System Settings prompt. Run the built, signed `.app` (not the dev
binary) when you want grants to persist across rebuilds — macOS ties them
to the signed bundle identity.

## Usage

1. Launch the app. It opens on the workflow list with the permission
   strip in the header. Grant any missing permission; the Record button
   enables when all three pass.
2. Click **● Record New Workflow**. The live capture view opens with the
   red **■ Stop Recording** banner. Work through the flow you want to
   record — click and type in any application. Each click and key press
   appears as a step row while you work.
3. Click **■ Stop Recording**. The editor opens in draft review with a
   `draft` badge. Select steps to check their screenshots and metadata,
   edit titles or classifications, and delete noise steps.
4. Click **Save…**, type a name over the pre-selected timestamp default,
   and save. Or click **Discard** and confirm to delete the recording's
   folder. If a recording fails mid-way (for example a revoked
   permission), the committed steps still open in draft review behind an
   error banner.
5. From the landing list, click a row to reopen a workflow. Edit titles,
   descriptions, and classifications (they save automatically), rename
   the workflow in the header, hover a row for **⌘ Reveal** in Finder,
   or use **Delete…** in the detail header to remove a workflow and all
   its captured data.

## Limitations

- macOS-only. Other platforms are out of scope.
- Window crop caveat: the window-crop screenshot is a crop of the full-screen
  frame at the window's bounds, not an isolated image of the window. Content
  overlapping those bounds appears in the crop.
