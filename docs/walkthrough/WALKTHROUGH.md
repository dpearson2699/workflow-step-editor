# Written walkthrough

This is the text companion to the video. It follows the same order and
carries the points that do not fit in an on-screen annotation. Five minutes
to read.

## 1. What you are looking at

A Tauri v2 app: React + Vite in the webview, Rust behind it. It records
what a person does on their Mac (clicks and key presses across every app),
takes three screenshots per event, turns each event into a reviewable step,
and stores everything as JSON and PNG under
`~/Library/Application Support/com.dpearson.workflow-step-editor/workflows/`.
No server, no API keys.

The landing page lists saved workflows newest first. The header strip shows
the three macOS permissions the recorder needs. Recording stays disabled
until all three are green. Input Monitoring is requested first on purpose:
an early Accessibility check suppresses the Input Monitoring prompt, so the
backend enforces the order and reports `blocked_by_prerequisite` if you try
it the other way.

## 2. Recording

One button starts a recording. The live view has one visible action, Stop.
While you work in other apps, each click and key press appears as a row.

Under the hood:

- A ListenOnly `CGEventTap` on its own run loop thread sees the input. It
  never modifies or delays real events, and its callback never blocks. It
  pins the newest buffered frame and hands the event to a bounded FIFO
  queue.
- One ScreenCaptureKit stream per display runs continuously at about 10
  fps and keeps the two newest frames in memory. That is what makes a
  click screenshot show the menu *before* the click closed it. On-demand
  capture after the event was rejected for that reason (ADR-0001).
- One worker drains the queue: resolves the window and the accessibility
  element under the pointer (or the focused element for a key), cuts the
  three crops from the selected frame, encodes PNGs, appends one line to
  `events.jsonl`, and streams a step to the UI. If the queue ever
  saturates, the recording fail-stops with an explicit error instead of
  dropping events silently.
- Interactions with the recorder's own window, including the Stop click,
  are dropped before any disk write.

The `hello` typing in the video is the tricky case. For a key press the
pre-event frame shows the field without the character, so key-down steps
select the first frame within 250 ms whose pixels inside the focused
element changed; clicks keep the pre-event frame. That rule took three
real-recording gates to get right; see section 5.

## 3. Draft review

Stopping opens the editor in draft mode. Everything is already on disk at
this point; the Save dialog only names the folder.

The detail pane keeps all three screenshots visible: one large, two
click-to-swap thumbnails (full screen, window crop, element crop). Steps
carry an auto-title such as `Click "Format" — TextEdit` or
`Press Cmd+S — TextEdit`, an empty description, and a classification. Each
event is exactly one step; `click` and `type` are assigned automatically,
`wait` and `assert` are yours to set. Titles, descriptions, and
classifications autosave. Steps can be deleted.

Chords like `Cmd+S` are recognized by a small classifier that looks at
modifier state only, never timing (ADR-0002). Its verdicts are not
persisted, so `events.jsonl` stays a lossless record and grouping or
shortcut presentation can be added later without a schema change.

## 4. Library and disk

Back on the list, the new workflow is at the top with its thumbnail, name,
`date · step count · duration`. Reveal in Finder shows the folder:
`workflow.json` (the editable manifest, `schema_version: 1`, steps
referencing events by id), `events.jsonl` (append-only raw events), and
`shots/` (three PNGs per event). Rename edits the manifest. Delete is a
confirmed hard delete (ADR-0003); no trash lifecycle in the MVP.

## 5. How it was built

The brief allows four hours. I read that as four hours of my own time in
the loop. I did not prompt an assistant line by line. I ran a
repository-local agent harness (`.agents/`) and worked above it:

1. A wayfinder map (issue #1) turned the brief into decision tickets. Four
   research tickets checked the macOS APIs before any code, five grilling
   tickets pinned architecture, storage, parsing, and the stretch boundary,
   and one prototype ticket produced UI variants I picked from.
2. Each capability then ran interview → ChatGPT Pro plan → Codex consensus
   rounds → blind completeness audit → adopted plan → one worktree task
   per PR → independent exact-head review → merge. Eight PRs, seven merged.
3. Every capability ended in a real recording on the signed build that I
   ran and inspected. Merge was blocked until I accepted.

Where it went wrong, and why that mattered: the key-down timing rule was
planned by ChatGPT Pro, agreed by Codex over five rounds, unit-tested
green, and failed the first real recording (it captured the title bar's
dirty-state repaint instead of the typed character). The second rule, a
100 ms settle then the newest frame, passed the same checks and failed the
next recording (it showed characters typed after the step). The
content-aware rule shipped on the third gate with one stated tradeoff: the
very first keystroke of a recording can still show the pre-glyph frame
(issue #43). The unit tests passed all three times. Running the app is what
caught the two bad rules.

Every defect that review found and I chose not to fix inside the four
hours is an open issue with a reproduction, a fingerprint, and a severity.
None of them is reachable on the record → stop → review → save path. The
two I would fix first are incremental manifest persistence (#21) and the
inert display-reconfiguration observer (#24).

## 6. What was cut

Text-input grouping (#14) and keyboard-shortcut presentation (#15) are the
two stretch items left on the table, both sitting on top of the classifier
that already ships. Automatic `wait`/`assert` detection, re-parse of the
event log, SQLite, and any trash lifecycle were cut on purpose. The README
has the full table.
