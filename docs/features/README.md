# Feature Work Bundles

The spec-driven feature workflow creates active bundles under `in_progress/`
using the identity `<YYYY-MM-DD-slug>`. It moves a bundle to `completed/` only
after every slice is delivered, acceptance and review are current, tasks are
quiescent, and the immutable completed projection is verified on the default
branch.

The primary specification, interview ledger, decisions, acceptance criteria,
slice plans, and receipts are durable project-management history. `state.json`
and generated `STATUS.md` are owned by the workflow scripts and must not be
hand-edited.

The directories are otherwise expected to be empty in a fresh project.
