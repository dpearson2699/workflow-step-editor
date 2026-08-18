# Workflow Step Editor

A Tauri v2 desktop application that records desktop workflow steps from real
clicks and key presses. This is the application shell with the permission
module; the capture pipeline builds on it.

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

## Limitations

- macOS-only. Other platforms are out of scope.
- Window crop caveat: the window-crop screenshot is a crop of the full-screen
  frame at the window's bounds, not an isolated image of the window. Content
  overlapping those bounds appears in the crop.
