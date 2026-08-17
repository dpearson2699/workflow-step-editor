//! `KeySemantics`: the one pure, stateless, unpersisted key-event semantic
//! classifier (ADR 0002, issue #9).
//!
//! A key-down is a shortcut chord exactly when a non-Shift semantic
//! modifier — Command, Control, Option, or Fn — is held. Shift and lock
//! keys never make a chord on their own, and there are no timing rules.
//! Verdicts are derived on demand and are never written to `events.jsonl`.
//!
//! This type also pins the normalized chord presentation (DEC-011): the
//! modifier order is Fn, Ctrl, Opt, Shift, Cmd around the key, and Shift
//! is shown only inside a chord.

use crate::domain::schema::Modifier;

/// Pure key-event semantic classifier. Stateless by construction: every
/// answer is a function of the modifiers passed in.
pub struct KeySemantics;

/// The normalized chord presentation order (DEC-011): Fn, Ctrl, Opt,
/// Shift, Cmd. Lock keys never appear in a chord presentation.
const PRESENTATION_ORDER: [Modifier; 5] = [
    Modifier::Fn,
    Modifier::Control,
    Modifier::Option,
    Modifier::Shift,
    Modifier::Command,
];

impl KeySemantics {
    /// True when `modifier` is in the semantic chord mask: Command,
    /// Control, Option, or Fn. Shift and lock keys are excluded.
    pub fn is_semantic_modifier(modifier: Modifier) -> bool {
        matches!(
            modifier,
            Modifier::Command | Modifier::Control | Modifier::Option | Modifier::Fn
        )
    }

    /// True when the key-down is a shortcut chord: any held non-Shift
    /// semantic modifier. No timing rules; holding a modifier through
    /// repeated key-downs yields one chord verdict per key-down.
    pub fn is_chord(held: &[Modifier]) -> bool {
        held.iter().copied().any(Self::is_semantic_modifier)
    }

    /// The held modifiers a chord title presents, deduplicated and in the
    /// normalized order Fn, Ctrl, Opt, Shift, Cmd. Shift is included here
    /// because it is shown inside a chord; lock keys are dropped. The
    /// caller decides chord membership with [`Self::is_chord`] first.
    pub fn presented_modifiers(held: &[Modifier]) -> Vec<Modifier> {
        PRESENTATION_ORDER
            .into_iter()
            .filter(|candidate| held.contains(candidate))
            .collect()
    }

    /// The presentation label for one modifier (DEC-011 order tokens).
    pub fn modifier_label(modifier: Modifier) -> &'static str {
        match modifier {
            Modifier::Fn => "Fn",
            Modifier::Control => "Ctrl",
            Modifier::Option => "Opt",
            Modifier::Shift => "Shift",
            Modifier::Command => "Cmd",
            Modifier::CapsLock => "CapsLock",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::schema::{
        CaptureMeta, ElementInfo, ElementSource, Event, EventKind, KeyInfo, Modifier, Pos, Rect,
        ShotPaths,
    };

    use super::*;

    #[test]
    fn plain_key_is_not_a_chord() {
        assert!(!KeySemantics::is_chord(&[]));
    }

    #[test]
    fn shift_only_key_is_not_a_chord() {
        assert!(!KeySemantics::is_chord(&[Modifier::Shift]));
    }

    #[test]
    fn lock_key_is_not_a_chord() {
        assert!(!KeySemantics::is_chord(&[Modifier::CapsLock]));
        assert!(!KeySemantics::is_chord(&[
            Modifier::CapsLock,
            Modifier::Shift
        ]));
    }

    #[test]
    fn one_non_shift_semantic_modifier_makes_a_chord() {
        assert!(KeySemantics::is_chord(&[Modifier::Command]));
        assert!(KeySemantics::is_chord(&[Modifier::Control]));
        assert!(KeySemantics::is_chord(&[Modifier::Option]));
        assert!(KeySemantics::is_chord(&[Modifier::Fn]));
    }

    #[test]
    fn multiple_modifiers_make_a_chord_and_normalize_in_order() {
        let held = [
            Modifier::Command,
            Modifier::Shift,
            Modifier::Fn,
            Modifier::Option,
            Modifier::Control,
        ];
        assert!(KeySemantics::is_chord(&held));
        assert_eq!(
            KeySemantics::presented_modifiers(&held),
            vec![
                Modifier::Fn,
                Modifier::Control,
                Modifier::Option,
                Modifier::Shift,
                Modifier::Command,
            ],
        );
    }

    #[test]
    fn lock_keys_are_dropped_from_the_presented_chord() {
        let held = [Modifier::CapsLock, Modifier::Command];
        assert!(KeySemantics::is_chord(&held));
        assert_eq!(
            KeySemantics::presented_modifiers(&held),
            vec![Modifier::Command],
        );
    }

    #[test]
    fn repeated_key_downs_under_a_held_modifier_yield_one_verdict_each() {
        // The classifier is stateless: the same held-modifier fact yields
        // the same single verdict on every key-down, with no timing rules.
        let held = [Modifier::Command];
        let verdicts: Vec<bool> = (0..3).map(|_| KeySemantics::is_chord(&held)).collect();
        assert_eq!(verdicts, vec![true, true, true]);
    }

    /// AC-002: no classifier verdict is persisted. A chord key-down event
    /// serializes with exactly the schema v1 field set — no chord,
    /// shortcut, or verdict field anywhere in the line.
    #[test]
    fn serialized_chord_event_carries_no_classifier_verdict() {
        let event = Event {
            id: "evt_0001".into(),
            ts: "2026-08-16T22:31:05.123Z".into(),
            kind: EventKind::KeyDown,
            display_id: 1,
            pos: Pos { x: 512.0, y: 384.0 },
            button: None,
            key: Some(KeyInfo {
                key_code: 1,
                chars: "s".into(),
                modifiers: vec![Modifier::Command],
            }),
            window: None,
            element: ElementInfo {
                role: None,
                title: None,
                frame: Rect {
                    x: 0,
                    y: 0,
                    w: 200,
                    h: 200,
                },
                source: ElementSource::Fallback,
            },
            shots: ShotPaths::for_event("evt_0001"),
            capture: CaptureMeta { frame_age_ms: 5 },
        };
        assert!(KeySemantics::is_chord(&event.key.as_ref().unwrap().modifiers));

        let value = serde_json::to_value(&event).unwrap();
        let top_keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            top_keys,
            vec![
                "button", "capture", "display_id", "element", "id", "key", "kind", "pos", "shots",
                "ts", "window",
            ],
        );
        let key_keys: Vec<&str> = value["key"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(key_keys, vec!["chars", "key_code", "modifiers"]);

        let line = serde_json::to_string(&event).unwrap();
        for verdict_marker in ["chord", "shortcut", "verdict", "semantic"] {
            assert!(
                !line.contains(verdict_marker),
                "event line must not persist a classifier verdict; found {verdict_marker:?} in {line}",
            );
        }
    }
}
