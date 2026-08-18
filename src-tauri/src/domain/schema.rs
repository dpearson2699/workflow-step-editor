//! Schema v1 data types for `events.jsonl` and `workflow.json`.
//!
//! Field names and shapes are fixed by the issue #7 decision record; the
//! `window: null` shape for unresolvable windows extends the schema's null
//! pattern per DEC-011. The golden tests in this module pin the exact
//! decided field shapes. Unknown fields are not rejected on read: ADR 0002
//! reserves the possibility of an additive JSONL field within schema v1.

use serde::{Deserialize, Serialize};

/// The schema version this build reads and writes.
pub const SCHEMA_VERSION: u32 = 1;

/// A screen position in display points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

/// An integer pixel rectangle (window bounds and element frames).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// The kind of one captured raw event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Click,
    KeyDown,
}

/// The mouse button of a click event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// A modifier key held during a key-down, recorded as a raw fact.
///
/// Chord semantics over these values belong to
/// [`crate::domain::key_semantics::KeySemantics`]; the log stores only what
/// was held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Fn,
    Control,
    Option,
    Shift,
    Command,
    CapsLock,
}

/// The `key` object of a key-down event (`null` on clicks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyInfo {
    /// macOS virtual key code (for example 4 for the H key).
    pub key_code: u16,
    /// The characters the key-down produced; empty for non-character keys.
    pub chars: String,
    /// Modifiers held during the key-down, in capture order.
    pub modifiers: Vec<Modifier>,
}

/// The window the event hit (`null` when no window resolves, DEC-011).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app: String,
    pub title: String,
    pub pid: i32,
    pub bounds: Rect,
}

/// Where the element metadata came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementSource {
    Ax,
    Fallback,
}

/// Resolved UI-element metadata for the event.
///
/// For the fallback shape (DEC-011): `role` and `title` are `null`, `frame`
/// is the fallback crop rectangle, and `source` is `"fallback"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementInfo {
    pub role: Option<String>,
    pub title: Option<String>,
    pub frame: Rect,
    pub source: ElementSource,
}

/// Relative paths of the persisted screenshot triple, as recorded in the
/// event line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotPaths {
    pub full: String,
    pub window: String,
    pub element: String,
}

impl ShotPaths {
    /// The canonical shot paths for one event id.
    pub fn for_event(event_id: &str) -> Self {
        Self {
            full: format!("shots/{event_id}.full.png"),
            window: format!("shots/{event_id}.window.png"),
            element: format!("shots/{event_id}.element.png"),
        }
    }
}

/// Capture metadata for the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureMeta {
    /// Nonnegative event-to-selected-frame age in milliseconds,
    /// saturating at zero; a post-event key-down frame therefore
    /// reports 0.
    pub frame_age_ms: u64,
}

/// One line of `events.jsonl`: a raw captured event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    /// RFC 3339 UTC timestamp with millisecond precision.
    pub ts: String,
    pub kind: EventKind,
    pub display_id: u32,
    pub pos: Pos,
    /// `null` for key-down events.
    pub button: Option<MouseButton>,
    /// `null` for click events.
    pub key: Option<KeyInfo>,
    /// `null` when no window resolves (DEC-011).
    pub window: Option<WindowInfo>,
    pub element: ElementInfo,
    pub shots: ShotPaths,
    pub capture: CaptureMeta,
}

/// The step classification enum. Capture-time defaults are
/// click -> `click` and key-down -> `type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    Click,
    Type,
    Wait,
    Assert,
}

/// One reviewable step in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    /// The raw events this step was parsed from. An array so text-input
    /// grouping needs no schema change (issue #7).
    pub event_ids: Vec<String>,
    pub classification: Classification,
    pub title: String,
    pub description: String,
}

/// The editable `workflow.json` manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    /// RFC 3339 UTC timestamp with second precision.
    pub created_at: String,
    pub steps: Vec<Step>,
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    /// The exact click-event example from the issue #7 decision record.
    const GOLDEN_CLICK_EVENT: &str = r#"{"id":"evt_01H...","ts":"2026-08-16T22:31:05.123Z","kind":"click",
 "display_id":1,"pos":{"x":512.0,"y":384.0},"button":"left",
 "key":null,
 "window":{"app":"TextEdit","title":"Untitled","pid":871,"bounds":{"x":100,"y":50,"w":800,"h":600}},
 "element":{"role":"AXButton","title":"OK","frame":{"x":480,"y":360,"w":80,"h":32},"source":"ax"},
 "shots":{"full":"shots/evt_01H.full.png","window":"shots/evt_01H.window.png","element":"shots/evt_01H.element.png"},
 "capture":{"frame_age_ms":12}}"#;

    /// The key-down shape from issue #7: `key` carries
    /// `{"key_code":4,"chars":"h","modifiers":[]}` and `button` is null.
    const GOLDEN_KEY_DOWN_EVENT: &str = r#"{"id":"evt_01H...","ts":"2026-08-16T22:31:06.456Z","kind":"key_down",
 "display_id":1,"pos":{"x":512.0,"y":384.0},"button":null,
 "key":{"key_code":4,"chars":"h","modifiers":[]},
 "window":{"app":"TextEdit","title":"Untitled","pid":871,"bounds":{"x":100,"y":50,"w":800,"h":600}},
 "element":{"role":"AXTextArea","title":null,"frame":{"x":120,"y":80,"w":760,"h":540},"source":"ax"},
 "shots":{"full":"shots/evt_01H.full.png","window":"shots/evt_01H.window.png","element":"shots/evt_01H.element.png"},
 "capture":{"frame_age_ms":9}}"#;

    /// The DEC-011 null-window shape: `window: null`, element `role: null`,
    /// `title: null`, the fallback crop rectangle as `frame`, and
    /// `source: "fallback"`.
    const GOLDEN_NULL_WINDOW_EVENT: &str = r#"{"id":"evt_01H...","ts":"2026-08-16T22:31:07.000Z","kind":"click",
 "display_id":1,"pos":{"x":512.0,"y":384.0},"button":"left",
 "key":null,
 "window":null,
 "element":{"role":null,"title":null,"frame":{"x":412,"y":284,"w":200,"h":200},"source":"fallback"},
 "shots":{"full":"shots/evt_01H.full.png","window":"shots/evt_01H.window.png","element":"shots/evt_01H.element.png"},
 "capture":{"frame_age_ms":12}}"#;

    /// The exact manifest example from the issue #7 decision record.
    const GOLDEN_MANIFEST: &str = r#"{"schema_version":1,"id":"01H...","name":"Approve invoice",
 "created_at":"2026-08-16T22:31:00Z",
 "steps":[{"id":"step_01H...","event_ids":["evt_01H..."],
   "classification":"click","title":"Click OK","description":""}]}"#;

    /// Round-trips golden JSON through the typed schema and asserts the
    /// re-serialized value is identical: every field name and value the
    /// record pins survives, and no extra field appears.
    fn assert_round_trip<T>(golden: &str)
    where
        T: serde::de::DeserializeOwned + Serialize,
    {
        let original: Value = serde_json::from_str(golden).expect("golden text parses as JSON");
        let typed: T = serde_json::from_str(golden).expect("golden text parses into the schema type");
        let reserialized: Value =
            serde_json::to_value(&typed).expect("schema type serializes back to JSON");
        assert_eq!(reserialized, original);
    }

    #[test]
    fn golden_click_event_round_trips_exactly() {
        assert_round_trip::<Event>(GOLDEN_CLICK_EVENT);
    }

    #[test]
    fn golden_key_down_event_round_trips_exactly() {
        assert_round_trip::<Event>(GOLDEN_KEY_DOWN_EVENT);
    }

    #[test]
    fn golden_null_window_event_round_trips_exactly() {
        assert_round_trip::<Event>(GOLDEN_NULL_WINDOW_EVENT);
    }

    #[test]
    fn golden_manifest_round_trips_exactly() {
        assert_round_trip::<Manifest>(GOLDEN_MANIFEST);
    }

    #[test]
    fn null_window_event_parses_to_the_dec_011_shape() {
        let event: Event = serde_json::from_str(GOLDEN_NULL_WINDOW_EVENT).unwrap();
        assert_eq!(event.window, None);
        assert_eq!(event.element.role, None);
        assert_eq!(event.element.title, None);
        assert_eq!(event.element.source, ElementSource::Fallback);
    }

    #[test]
    fn shot_paths_derive_from_the_event_id() {
        assert_eq!(
            ShotPaths::for_event("evt_0001"),
            ShotPaths {
                full: "shots/evt_0001.full.png".into(),
                window: "shots/evt_0001.window.png".into(),
                element: "shots/evt_0001.element.png".into(),
            },
        );
    }
}
