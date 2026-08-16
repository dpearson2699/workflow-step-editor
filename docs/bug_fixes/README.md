# Bug-Fix Work Bundles

`project-debugging` keeps diagnosis-only work read-only. A full fix enters the
shared lifecycle only after root cause and a generalized regression abstraction
are complete, then creates an active bundle under `in_progress/` using the
identity `<YYYY-MM-DD-slug>`.

The workflow moves the bundle to `completed/` only after at least one planned
fix PR is merged and all shared completion gates pass. `state.json` and
generated `STATUS.md` are owned by the workflow scripts and must not be
hand-edited.

The directories are otherwise expected to be empty in a fresh project.
