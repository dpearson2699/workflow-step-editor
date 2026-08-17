//! Live 1:1 event-to-step parsing with auto-titles (issue #10, DEC-003).
//!
//! Each captured event parses into exactly one step. Classification
//! defaults are click -> `click` and key-down -> `type`; the description
//! defaults to empty. Titles follow the decided formats:
//!
//! - `Click "OK" — TextEdit` (element title, else element role)
//! - `Click at (512, 384) — TextEdit` (coordinate fallback)
//! - `Press H — Chrome` (plain key)
//! - `Press Cmd+S — TextEdit` (chord; modifiers normalized by
//!   [`KeySemantics`] in the order Fn, Ctrl, Opt, Shift, Cmd)
//!
//! For a null-window event (DEC-011) the title's `{app}` falls back to the
//! frontmost application name, else `Unknown`.

use crate::domain::key_semantics::KeySemantics;
use crate::domain::schema::{Classification, Event, EventKind, KeyInfo, Step};

/// Parses one captured event into exactly one step.
///
/// `frontmost_app` is the resolved frontmost application name at capture
/// time; it is only consulted when the event has no window (DEC-011).
pub fn parse_step(step_id: String, event: &Event, frontmost_app: Option<&str>) -> Step {
    let classification = match event.kind {
        EventKind::Click => Classification::Click,
        EventKind::KeyDown => Classification::Type,
    };
    Step {
        id: step_id,
        event_ids: vec![event.id.clone()],
        classification,
        title: title_for(event, frontmost_app),
        description: String::new(),
    }
}

fn title_for(event: &Event, frontmost_app: Option<&str>) -> String {
    let app = app_name(event, frontmost_app);
    let subject = match event.kind {
        EventKind::Click => click_subject(event),
        EventKind::KeyDown => key_subject(event.key.as_ref()),
    };
    format!("{subject} — {app}")
}

/// The title's `{app}`: the hit window's application, else the frontmost
/// application name, else `Unknown` (DEC-011).
fn app_name(event: &Event, frontmost_app: Option<&str>) -> String {
    if let Some(window) = &event.window {
        return window.app.clone();
    }
    match frontmost_app {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => "Unknown".to_owned(),
    }
}

/// `Click "{element title | role}"`, else the coordinate fallback
/// `Click at (x, y)` (DEC-003).
fn click_subject(event: &Event) -> String {
    let element_name = [&event.element.title, &event.element.role]
        .into_iter()
        .flatten()
        .find(|name| !name.trim().is_empty());
    match element_name {
        Some(name) => format!("Click \"{name}\""),
        None => format!(
            "Click at ({}, {})",
            event.pos.x.round() as i64,
            event.pos.y.round() as i64,
        ),
    }
}

/// `Press {modifier prefixes}{key display}`. Modifier prefixes appear only
/// on a chord, joined by `+` in the normalized order; Shift is shown only
/// inside a chord.
fn key_subject(key: Option<&KeyInfo>) -> String {
    let Some(key) = key else {
        // A key-down without a key payload cannot occur through the
        // capture pipeline; keep the title total anyway.
        return "Press Unknown".to_owned();
    };
    let display = key_display(key);
    if KeySemantics::is_chord(&key.modifiers) {
        let prefix: Vec<&str> = KeySemantics::presented_modifiers(&key.modifiers)
            .into_iter()
            .map(KeySemantics::modifier_label)
            .collect();
        format!("Press {}+{display}", prefix.join("+"))
    } else {
        format!("Press {display}")
    }
}

/// The key's display form: the produced character uppercased when it is a
/// single printable non-whitespace character, else the key-code name.
fn key_display(key: &KeyInfo) -> String {
    let mut chars = key.chars.chars();
    if let (Some(ch), None) = (chars.next(), chars.next()) {
        if !ch.is_whitespace() && !ch.is_control() {
            return ch.to_uppercase().to_string();
        }
    }
    key_code_name(key.key_code)
}

/// Names for the common macOS virtual key codes that produce no printable
/// character.
fn key_code_name(key_code: u16) -> String {
    let name = match key_code {
        36 => "Return",
        48 => "Tab",
        49 => "Space",
        51 => "Delete",
        53 => "Escape",
        76 => "Enter",
        115 => "Home",
        116 => "Page Up",
        117 => "Forward Delete",
        119 => "End",
        121 => "Page Down",
        122 => "F1",
        120 => "F2",
        99 => "F3",
        118 => "F4",
        96 => "F5",
        97 => "F6",
        98 => "F7",
        100 => "F8",
        101 => "F9",
        109 => "F10",
        103 => "F11",
        111 => "F12",
        123 => "Left Arrow",
        124 => "Right Arrow",
        125 => "Down Arrow",
        126 => "Up Arrow",
        _ => return format!("Key {key_code}"),
    };
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use crate::domain::schema::{
        CaptureMeta, ElementInfo, ElementSource, Event, Modifier, MouseButton, Pos, Rect,
        ShotPaths, WindowInfo,
    };

    use super::*;

    fn window(app: &str) -> Option<WindowInfo> {
        Some(WindowInfo {
            app: app.into(),
            title: "Untitled".into(),
            pid: 871,
            bounds: Rect {
                x: 100,
                y: 50,
                w: 800,
                h: 600,
            },
        })
    }

    fn element(role: Option<&str>, title: Option<&str>) -> ElementInfo {
        ElementInfo {
            role: role.map(Into::into),
            title: title.map(Into::into),
            frame: Rect {
                x: 480,
                y: 360,
                w: 80,
                h: 32,
            },
            source: if role.is_some() {
                ElementSource::Ax
            } else {
                ElementSource::Fallback
            },
        }
    }

    fn click_event(window: Option<WindowInfo>, element: ElementInfo) -> Event {
        Event {
            id: "evt_0001".into(),
            ts: "2026-08-16T22:31:05.123Z".into(),
            kind: EventKind::Click,
            display_id: 1,
            pos: Pos { x: 512.0, y: 384.0 },
            button: Some(MouseButton::Left),
            key: None,
            window,
            element,
            shots: ShotPaths::for_event("evt_0001"),
            capture: CaptureMeta { frame_age_ms: 12 },
        }
    }

    fn key_event(app: &str, key_code: u16, chars: &str, modifiers: Vec<Modifier>) -> Event {
        Event {
            key: Some(KeyInfo {
                key_code,
                chars: chars.into(),
                modifiers,
            }),
            kind: EventKind::KeyDown,
            button: None,
            ..click_event(window(app), element(Some("AXTextArea"), None))
        }
    }

    fn title(event: &Event, frontmost_app: Option<&str>) -> String {
        parse_step("step_0001".into(), event, frontmost_app).title
    }

    #[test]
    fn titled_element_click_uses_the_quoted_element_title() {
        let event = click_event(window("TextEdit"), element(Some("AXButton"), Some("OK")));
        assert_eq!(title(&event, None), "Click \"OK\" — TextEdit");
    }

    #[test]
    fn untitled_element_click_falls_back_to_the_role() {
        let event = click_event(window("TextEdit"), element(Some("AXButton"), None));
        assert_eq!(title(&event, None), "Click \"AXButton\" — TextEdit");
    }

    #[test]
    fn unresolved_element_click_falls_back_to_coordinates() {
        let event = click_event(window("TextEdit"), element(None, None));
        assert_eq!(title(&event, None), "Click at (512, 384) — TextEdit");
    }

    #[test]
    fn empty_element_names_fall_back_to_coordinates() {
        let event = click_event(window("TextEdit"), element(Some(""), Some(" ")));
        assert_eq!(title(&event, None), "Click at (512, 384) — TextEdit");
    }

    #[test]
    fn plain_key_press_uppercases_the_character() {
        let event = key_event("Chrome", 4, "h", vec![]);
        assert_eq!(title(&event, None), "Press H — Chrome");
    }

    #[test]
    fn shift_only_key_shows_no_modifier_prefix() {
        let event = key_event("Chrome", 4, "H", vec![Modifier::Shift]);
        assert_eq!(title(&event, None), "Press H — Chrome");
    }

    #[test]
    fn command_chord_uses_the_cmd_prefix() {
        let event = key_event("TextEdit", 1, "s", vec![Modifier::Command]);
        assert_eq!(title(&event, None), "Press Cmd+S — TextEdit");
    }

    #[test]
    fn shift_appears_inside_a_chord() {
        let event = key_event("TextEdit", 1, "s", vec![Modifier::Shift, Modifier::Command]);
        assert_eq!(title(&event, None), "Press Shift+Cmd+S — TextEdit");
    }

    #[test]
    fn fn_combinations_normalize_in_fn_ctrl_opt_shift_cmd_order() {
        let event = key_event(
            "Terminal",
            96,
            "",
            vec![Modifier::Command, Modifier::Fn, Modifier::Control],
        );
        assert_eq!(title(&event, None), "Press Fn+Ctrl+Cmd+F5 — Terminal");

        let all = key_event(
            "Terminal",
            1,
            "s",
            vec![
                Modifier::Shift,
                Modifier::Command,
                Modifier::Option,
                Modifier::Fn,
                Modifier::Control,
            ],
        );
        assert_eq!(
            title(&all, None),
            "Press Fn+Ctrl+Opt+Shift+Cmd+S — Terminal",
        );
    }

    #[test]
    fn fn_only_chord_names_the_special_key() {
        let event = key_event("Finder", 123, "", vec![Modifier::Fn]);
        assert_eq!(title(&event, None), "Press Fn+Left Arrow — Finder");
    }

    #[test]
    fn null_window_title_falls_back_to_the_frontmost_app() {
        let event = click_event(None, element(None, None));
        assert_eq!(title(&event, Some("Finder")), "Click at (512, 384) — Finder");
    }

    #[test]
    fn null_window_without_frontmost_app_falls_back_to_unknown() {
        let event = click_event(None, element(None, None));
        assert_eq!(title(&event, None), "Click at (512, 384) — Unknown");
        assert_eq!(title(&event, Some(" ")), "Click at (512, 384) — Unknown");
    }

    #[test]
    fn click_classification_defaults_to_click() {
        let event = click_event(window("TextEdit"), element(Some("AXButton"), Some("OK")));
        let step = parse_step("step_0001".into(), &event, None);
        assert_eq!(step.classification, Classification::Click);
        assert_eq!(step.event_ids, vec!["evt_0001".to_owned()]);
    }

    #[test]
    fn key_down_classification_defaults_to_type() {
        let event = key_event("Chrome", 4, "h", vec![]);
        let step = parse_step("step_0001".into(), &event, None);
        assert_eq!(step.classification, Classification::Type);
    }

    #[test]
    fn description_defaults_to_empty() {
        let event = key_event("Chrome", 4, "h", vec![]);
        let step = parse_step("step_0001".into(), &event, None);
        assert_eq!(step.description, "");
    }
}
