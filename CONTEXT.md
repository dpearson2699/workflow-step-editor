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
