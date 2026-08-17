# Workflow Step Editor

A macOS desktop tool that records how a human performs a task — clicks, typing,
and screenshots — and lets them review and annotate the recording as automation
workflow steps.

## Language

**Event**:
One raw input occurrence observed during recording — a click or a key-down.
_Avoid_: action, input, click record

**Step**:
The reviewable unit the editor displays, parsed from one or more events; it
carries a title, a description, and a classification.
_Avoid_: action, task, annotation

**Screenshot triple**:
The three images captured for every event: the full screen, the window crop,
and the element crop.
_Avoid_: screenshots, captures

**Workflow**:
One recorded task — the events captured in a single recording run together
with the steps a user reviews and annotates.
_Avoid_: recording, session, script

**Classification**:
The step's kind, one of `click`, `type`, `wait`, or `assert`.
_Avoid_: type, category, label

**Shortcut**:
A captured key-down that occurs while the user holds a non-Shift modifier
key. Each such key-down is one shortcut; detection uses modifier state, never
timing.
_Avoid_: chord, key combo

**Hotkey**:
A key combination the app itself listens for, such as the global record
toggle.
_Avoid_: app shortcut
