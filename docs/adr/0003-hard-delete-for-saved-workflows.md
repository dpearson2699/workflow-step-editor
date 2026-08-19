# Hard delete for saved workflows

Deleting a saved workflow permanently removes its folder from the app's
storage: the raw event log including recorded keystrokes, the manifest, and
all screenshots. The backend performs the deletion: it resolves the workflow
id to a recognized directory inside the configured workflow root, validates
that the operation cannot escape that root or follow an externally
substituted target, and removes the complete directory. The UI removes the
workflow only after backend success; an already-missing directory counts as
successfully deleted. A destructive confirmation that names the keystroke
data, with Cancel as the default action, guards the operation.

The app retains no deleted flag, tombstone, audit copy, or private trash
folder, and provides no restore or purge lifecycle.

Adopted from the user-approved decision record in
[issue #8](https://github.com/dpearson2699/workflow-step-editor/issues/8),
which supersedes the earlier "no workflow deletion in the minimum viable
product (MVP) UI" ruling in
issue #7.

## Considered options

- Front-end soft delete with audit retention: rejected on privacy grounds.
  Retained raw keystroke data has no audit consumer in this local
  single-user product and conflicts with the expected privacy semantics of
  a user-visible Delete action.
- No deletion in the MVP (the original issue #7 ruling): superseded once
  the review UI made saved workflows a managed list.

## Consequences

- Draft Discard and saved-workflow Delete share the same folder-removal
  primitive. Saved-workflow deletion differs only by its explicit
  permanent-deletion confirmation, because the workflow was previously
  named and intentionally saved.
- Deletion applies to the app-managed copy only. It claims no forensic
  secure erasure and does not remove copies in system snapshots, Time
  Machine, or other user-managed backups.
- A failed deletion leaves the workflow visible and surfaces an error; the
  UI never hides a workflow the backend still stores.
