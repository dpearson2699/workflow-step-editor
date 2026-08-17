# Key-event semantic classifier in the MVP core

Auto-title generation must already distinguish a shortcut chord (Cmd+S) from
plain typing, and both stretch capabilities — text-input grouping and keyboard
shortcuts — need exactly the same distinction as their boundary rule. We
therefore ship one pure, stateless, unpersisted key-event semantic classifier
(working name `KeySemantics`) in the MVP recording/parser core, and every
consumer — auto-titles now, burst boundaries and toggle filtering later —
routes through it. Chord detection uses modifier state only (any held
non-Shift modifier), never timing thresholds: users may hold modifiers
arbitrarily long (accessibility), matching OS shortcut semantics.

Decided with a ChatGPT Pro consultation on 2026-08-17
(https://chatgpt.com/c/6a828957-7574-83ea-b577-2e1fb36b199a) and adopted by
the owner in wayfinder ticket #9.

## Considered options

- Let a stretch capability own the classifier: rejected — it creates an
  artificial dependency between independently buildable capabilities, and
  title generation would accidentally own chord semantics in the meantime.
- Persist classifier verdicts per event in events.jsonl: rejected — a
  capture-time verdict is application policy, not a raw fact; it risks stale
  annotations and a dual source of truth. Revisit only if events.jsonl
  becomes a supported external interchange format, an audit requires
  capture-time verdicts, or classification starts depending on context the
  raw events do not retain; the field is an additive JSONL change then.

## Consequences

- events.jsonl stays a lossless record of captured facts; any future re-parse
  derives current semantics from raw events, versioned by `schema_version`.
- Shortcut key-downs remain user-facing `type` steps; the classification enum
  `click`/`type`/`wait`/`assert` never gains a shortcut value. A distinct
  `[Cmd+Tab]` presentation label belongs to the keyboard-shortcuts capability.
- Text-input grouping owns stateful burst formation; keyboard shortcuts owns
  app hotkeys, toggle filtering, and shortcut presentation; both consume the
  classifier and neither redefines its chord semantics.
- Changes to the classifier contract require reviewing both stretch
  capabilities' acceptance tests.
