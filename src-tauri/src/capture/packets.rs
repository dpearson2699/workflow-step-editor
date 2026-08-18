//! Pure assembly of one [`CapturePacket`] from the copied event facts,
//! the frame the worker selected, and the resolved window/AX metadata.
//!
//! This is where the DEC-008 and DEC-011 rules live as testable logic:
//! display selection, the deterministic implausible-frame fallback, the
//! fixed-size fallback crop, and the null-window shapes. No macOS API
//! is touched here. Which frame a job uses is decided by the worker
//! ([`crate::capture::worker`]): the pinned pre-event frame for a click,
//! the bounded post-event frame or the pinned fallback for a key-down
//! (DEC-001/DEC-002); [`build_packet`] cuts all three shots from that
//! one [`SelectedFrame`].

use std::sync::Arc;

use crate::capture::broker::FrameData;
use crate::capture::encoder::encode_crop_png;
use crate::capture::geometry::{
    crop_px, display_for_point, display_for_rect, fallback_rect_pt, full_display_px,
    is_implausible, DisplayGeometry, PointPt, RectPt,
};
use crate::capture::queue::{CaptureJob, RawInput};
use crate::domain::schema::{ElementInfo, ElementSource, Pos, WindowInfo};
use crate::recording::pipeline::{CapturePacket, PacketInput};
use crate::recording::store::ShotPayloads;

/// The resolved window under or focused for the event, in global
/// points.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedWindow {
    pub app: String,
    pub title: String,
    pub pid: i32,
    pub bounds_pt: RectPt,
}

/// The resolved AX element, in global points.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedElement {
    pub role: Option<String>,
    pub title: Option<String>,
    pub frame_pt: RectPt,
}

/// What the worker-side resolver produced for one event.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedMetadata {
    pub window: Option<ResolvedWindow>,
    pub element: Option<ResolvedElement>,
    pub frontmost_app: Option<String>,
}

/// Packet assembly failure; both variants map to the coordinator's
/// single fail-stop path.
#[derive(Debug, PartialEq, Eq)]
pub enum PacketBuildError {
    /// The selected display retains no frame at all (DEC-007 explicit
    /// fail-stop; no event is silently dropped).
    NoRetainedFrame { display_id: u32 },
    Encode(String),
}

impl std::fmt::Display for PacketBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRetainedFrame { display_id } => write!(
                f,
                "no retained pre-event frame for display {display_id}",
            ),
            Self::Encode(error) => write!(f, "screenshot encoding failed: {error}"),
        }
    }
}

/// The frame one job's screenshot triple is cut from: the event-time
/// display it was selected for plus the frame lease. The worker builds
/// it from the pinned snapshot ([`select_pinned_frame`]) and, for a
/// key-down, may swap in the bounded post-event frame of the same
/// display geometry.
#[derive(Debug, Clone)]
pub struct SelectedFrame {
    pub display: DisplayGeometry,
    pub frame: Arc<FrameData>,
}

/// Display selection shared by the worker and packet assembly. Clicks:
/// the display containing the click point, else the main display
/// (DEC-011). Key-downs (DEC-008): the display containing the focused
/// element's center, else the focused window's center, else the main
/// display. The backend lists the main display first; `None` only for
/// an empty display set.
pub fn select_display<'a>(
    displays: &'a [DisplayGeometry],
    job: &CaptureJob,
    meta: &ResolvedMetadata,
) -> Option<&'a DisplayGeometry> {
    let display = match job.input {
        RawInput::Click { .. } => display_for_point(displays, PointPt { x: job.x, y: job.y }),
        RawInput::KeyDown { .. } => meta
            .element
            .as_ref()
            .and_then(|element| display_for_rect(displays, &element.frame_pt))
            .or_else(|| {
                meta.window
                    .as_ref()
                    .and_then(|window| display_for_rect(displays, &window.bounds_pt))
            }),
    };
    display.or_else(|| displays.first())
}

/// Selects the job's display and reads its pinned pre-event frame from
/// the event-time snapshot. A display without any retained frame is the
/// explicit fail-stop (DEC-007), for every event kind.
pub fn select_pinned_frame(
    job: &CaptureJob,
    meta: &ResolvedMetadata,
) -> Result<SelectedFrame, PacketBuildError> {
    let Some(display) = select_display(&job.snapshot.displays, job, meta).cloned() else {
        return Err(PacketBuildError::NoRetainedFrame { display_id: 0 });
    };
    let frame = job
        .snapshot
        .frame_for(display.id)
        .cloned()
        .ok_or(PacketBuildError::NoRetainedFrame {
            display_id: display.id,
        })?;
    Ok(SelectedFrame { display, frame })
}

/// Builds the complete capture packet for one job from the frame the
/// worker selected. All three shots come from that one frame;
/// `frame_age_ms` is the saturating event-to-frame age, so a
/// post-event frame reports `0`.
pub fn build_packet(
    job: &CaptureJob,
    meta: &ResolvedMetadata,
    selected: &SelectedFrame,
) -> Result<CapturePacket, PacketBuildError> {
    let point = PointPt { x: job.x, y: job.y };
    let is_click = matches!(job.input, RawInput::Click { .. });
    let display = &selected.display;
    let frame = &selected.frame;

    // Window: the resolved window when it overlaps the selected
    // display; DEC-011 null-window otherwise (window crop = the full
    // display frame).
    let window_crop = meta
        .window
        .as_ref()
        .and_then(|window| crop_px(&frame.display, &window.bounds_pt))
        .unwrap_or_else(|| full_display_px(&frame.display));

    // Element: the AX frame when plausible; the fixed-size fallback
    // otherwise (DEC-008/DEC-011).
    let click_constraint = is_click.then_some(point);
    let ax_element = meta.element.as_ref().filter(|element| {
        !is_implausible(&element.frame_pt, display, click_constraint)
    });
    let element = match ax_element {
        Some(element) => ElementInfo {
            role: element.role.clone(),
            title: element.title.clone(),
            frame: element.frame_pt.to_schema_rect(),
            source: ElementSource::Ax,
        },
        None => {
            let container = meta
                .window
                .as_ref()
                .map(|window| window.bounds_pt)
                .unwrap_or(display.frame_pt);
            // Fallback center: the click point for clicks; the focused
            // window's center for key-downs; the display center for the
            // null-window key-down (DEC-011).
            let center = if is_click { point } else { container.center() };
            let rect = fallback_rect_pt(center, &container, display);
            ElementInfo {
                role: None,
                title: None,
                frame: rect.to_schema_rect(),
                source: ElementSource::Fallback,
            }
        }
    };
    let element_rect_pt = RectPt::new(
        f64::from(element.frame.x),
        f64::from(element.frame.y),
        f64::from(element.frame.w),
        f64::from(element.frame.h),
    );
    let element_crop = crop_px(&frame.display, &element_rect_pt)
        .unwrap_or_else(|| full_display_px(&frame.display));

    let shots = encode_triple(frame, window_crop, element_crop)
        .map_err(PacketBuildError::Encode)?;

    Ok(CapturePacket {
        input: match &job.input {
            RawInput::Click { button } => PacketInput::Click { button: *button },
            RawInput::KeyDown { key } => PacketInput::KeyDown { key: key.clone() },
        },
        pos: Pos { x: job.x, y: job.y },
        display_id: display.id,
        window: meta.window.as_ref().map(|window| WindowInfo {
            app: window.app.clone(),
            title: window.title.clone(),
            pid: window.pid,
            bounds: window.bounds_pt.to_schema_rect(),
        }),
        element,
        frontmost_app: meta.frontmost_app.clone(),
        frame_age_ms: job.snapshot.frame_age_ms(frame),
        shots,
    })
}

fn encode_triple(
    frame: &FrameData,
    window_crop: crate::capture::geometry::CropPx,
    element_crop: crate::capture::geometry::CropPx,
) -> Result<ShotPayloads, String> {
    Ok(ShotPayloads {
        full: encode_crop_png(frame, full_display_px(&frame.display))?,
        window: encode_crop_png(frame, window_crop)?,
        element: encode_crop_png(frame, element_crop)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::capture::broker::FrameSnapshot;
    use crate::capture::geometry::DisplayGeometry;
    use crate::domain::schema::{KeyInfo, MouseButton, Rect};

    use super::*;

    fn retina_main() -> DisplayGeometry {
        DisplayGeometry {
            id: 1,
            frame_pt: RectPt::new(0.0, 0.0, 1000.0, 600.0),
            scale: 2.0,
        }
    }

    fn side_1x() -> DisplayGeometry {
        DisplayGeometry {
            id: 2,
            frame_pt: RectPt::new(1000.0, 0.0, 800.0, 600.0),
            scale: 1.0,
        }
    }

    fn frame_for(display: &DisplayGeometry, ts_ns: u64) -> Arc<FrameData> {
        let width = display.width_px();
        let height = display.height_px();
        Arc::new(FrameData {
            display: display.clone(),
            width_px: width,
            height_px: height,
            bytes_per_row: width as usize * 4,
            ts_ns,
            pixels: vec![120; width as usize * 4 * height as usize],
        })
    }

    fn click_job(x: f64, y: f64, displays: Vec<DisplayGeometry>, frames: Vec<Arc<FrameData>>) -> CaptureJob {
        CaptureJob {
            input: RawInput::Click {
                button: MouseButton::Left,
            },
            x,
            y,
            ts_ns: 10_000_000,
            snapshot: FrameSnapshot::for_test(10_000_000, displays, frames),
        }
    }

    fn key_job(displays: Vec<DisplayGeometry>, frames: Vec<Arc<FrameData>>) -> CaptureJob {
        CaptureJob {
            input: RawInput::KeyDown {
                key: KeyInfo {
                    key_code: 4,
                    chars: "h".into(),
                    modifiers: vec![],
                },
            },
            x: 0.0,
            y: 0.0,
            ts_ns: 10_000_000,
            snapshot: FrameSnapshot::for_test(10_000_000, displays, frames),
        }
    }

    fn window(bounds: RectPt) -> ResolvedWindow {
        ResolvedWindow {
            app: "TextEdit".into(),
            title: "Untitled".into(),
            pid: 871,
            bounds_pt: bounds,
        }
    }

    /// The production composition for a pinned-frame job: select the
    /// display and its pinned frame, then assemble.
    fn assemble(job: &CaptureJob, meta: &ResolvedMetadata) -> Result<CapturePacket, PacketBuildError> {
        let selected = select_pinned_frame(job, meta)?;
        build_packet(job, meta, &selected)
    }

    fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(info.buffer_size());
        (info.width, info.height, buffer)
    }

    fn decode_size(bytes: &[u8]) -> (u32, u32) {
        let (width, height, _) = decode(bytes);
        (width, height)
    }

    #[test]
    fn plausible_ax_click_keeps_the_ax_element_and_crops_all_three_shots() {
        let display = retina_main();
        let frame = frame_for(&display, 8_000_000);
        let job = click_job(500.0, 380.0, vec![display], vec![frame]);
        let meta = ResolvedMetadata {
            window: Some(window(RectPt::new(100.0, 50.0, 800.0, 500.0))),
            element: Some(ResolvedElement {
                role: Some("AXButton".into()),
                title: Some("OK".into()),
                frame_pt: RectPt::new(480.0, 360.0, 80.0, 32.0),
            }),
            frontmost_app: Some("TextEdit".into()),
        };
        let packet = assemble(&job, &meta).unwrap();

        assert_eq!(packet.display_id, 1);
        assert_eq!(packet.element.source, ElementSource::Ax);
        assert_eq!(packet.element.role.as_deref(), Some("AXButton"));
        assert_eq!(
            packet.element.frame,
            Rect { x: 480, y: 360, w: 80, h: 32 },
        );
        assert_eq!(packet.frame_age_ms, 2);
        // Shots: full display, window bounds, element frame — at 2x.
        assert_eq!(decode_size(&packet.shots.full), (2000, 1200));
        assert_eq!(decode_size(&packet.shots.window), (1600, 1000));
        assert_eq!(decode_size(&packet.shots.element), (160, 64));
        assert_eq!(
            packet.window.as_ref().unwrap().bounds,
            Rect { x: 100, y: 50, w: 800, h: 500 },
        );
    }

    #[test]
    fn a_coarse_web_area_click_takes_the_fallback_crop() {
        // A Chromium-style hit: the AX element covers nearly the whole
        // display, so the implausible-frame rule triggers and the
        // fixed-size fallback centers on the click inside the window.
        let display = retina_main();
        let frame = frame_for(&display, 9_000_000);
        let job = click_job(500.0, 300.0, vec![display.clone()], vec![frame]);
        let meta = ResolvedMetadata {
            window: Some(window(RectPt::new(0.0, 25.0, 1000.0, 575.0))),
            element: Some(ResolvedElement {
                role: Some("AXWebArea".into()),
                title: None,
                frame_pt: RectPt::new(0.0, 25.0, 1000.0, 575.0),
            }),
            frontmost_app: Some("Google Chrome".into()),
        };
        let packet = assemble(&job, &meta).unwrap();

        assert_eq!(packet.element.source, ElementSource::Fallback);
        assert_eq!(packet.element.role, None);
        assert_eq!(packet.element.title, None);
        assert_eq!(
            packet.element.frame,
            Rect { x: 350, y: 200, w: 300, h: 200 },
        );
        assert_eq!(decode_size(&packet.shots.element), (600, 400));
        // The window still resolves normally.
        assert!(packet.window.is_some());
    }

    #[test]
    fn a_null_window_click_takes_the_dec_011_shapes() {
        // Desktop click: no window resolves. Display comes from the
        // click point, the window shot is the full display frame, and
        // the element is the fallback centered at the click point.
        let displays = vec![retina_main(), side_1x()];
        let frames = vec![
            frame_for(&displays[0], 9_000_000),
            frame_for(&displays[1], 9_000_000),
        ];
        let job = click_job(1400.0, 300.0, displays, frames);
        let meta = ResolvedMetadata {
            frontmost_app: Some("Finder".into()),
            ..Default::default()
        };
        let packet = assemble(&job, &meta).unwrap();

        assert_eq!(packet.display_id, 2);
        assert_eq!(packet.window, None);
        assert_eq!(packet.frontmost_app.as_deref(), Some("Finder"));
        assert_eq!(packet.element.source, ElementSource::Fallback);
        assert_eq!(packet.element.role, None);
        assert_eq!(
            packet.element.frame,
            Rect { x: 1250, y: 200, w: 300, h: 200 },
        );
        // Window shot = full display frame of the 1x display.
        assert_eq!(decode_size(&packet.shots.window), (800, 600));
        assert_eq!(decode_size(&packet.shots.element), (300, 200));
    }

    #[test]
    fn a_key_down_on_a_spanning_window_selects_the_element_display() {
        // DEC-008: the focused window spans both displays with its
        // center on display 1, but the focused element sits on display
        // 2 — the element's display wins, on mixed-scale geometry.
        let displays = vec![retina_main(), side_1x()];
        let frames = vec![
            frame_for(&displays[0], 9_000_000),
            frame_for(&displays[1], 9_500_000),
        ];
        let job = key_job(displays, frames);
        let meta = ResolvedMetadata {
            window: Some(window(RectPt::new(600.0, 100.0, 800.0, 400.0))),
            element: Some(ResolvedElement {
                role: Some("AXTextArea".into()),
                title: None,
                frame_pt: RectPt::new(1100.0, 150.0, 250.0, 200.0),
            }),
            frontmost_app: Some("TextEdit".into()),
        };
        let packet = assemble(&job, &meta).unwrap();

        assert_eq!(packet.display_id, 2);
        assert_eq!(packet.element.source, ElementSource::Ax);
        // The window crop clamps to the selected display's part.
        assert_eq!(decode_size(&packet.shots.window), (400, 400));
        assert_eq!(packet.frame_age_ms, 0);
    }

    #[test]
    fn a_key_down_without_an_element_falls_back_centered_in_the_window() {
        let display = retina_main();
        let frame = frame_for(&display, 9_000_000);
        let job = key_job(vec![display], vec![frame]);
        let bounds = RectPt::new(200.0, 100.0, 600.0, 400.0);
        let meta = ResolvedMetadata {
            window: Some(window(bounds)),
            element: None,
            frontmost_app: Some("TextEdit".into()),
        };
        let packet = assemble(&job, &meta).unwrap();

        assert_eq!(packet.display_id, 1);
        assert_eq!(packet.element.source, ElementSource::Fallback);
        // Centered on the focused window's center (500, 300).
        assert_eq!(
            packet.element.frame,
            Rect { x: 350, y: 200, w: 300, h: 200 },
        );
    }

    #[test]
    fn a_null_window_key_down_uses_the_main_display_center() {
        let displays = vec![retina_main(), side_1x()];
        let frames = vec![
            frame_for(&displays[0], 9_000_000),
            frame_for(&displays[1], 9_000_000),
        ];
        let job = key_job(displays, frames);
        let packet = assemble(&job, &ResolvedMetadata::default()).unwrap();

        // Main display (listed first), window crop = full display,
        // fallback centered at the display center (500, 300).
        assert_eq!(packet.display_id, 1);
        assert_eq!(packet.window, None);
        assert_eq!(decode_size(&packet.shots.window), (2000, 1200));
        assert_eq!(
            packet.element.frame,
            Rect { x: 350, y: 200, w: 300, h: 200 },
        );
    }

    #[test]
    fn a_display_without_any_retained_frame_is_an_explicit_fail_stop() {
        // Display 2 exists in the set but retains no frame (fresh after
        // a display-configuration change): the event maps to the
        // explicit fail-stop error, never a silent drop.
        let displays = vec![retina_main(), side_1x()];
        let frames = vec![frame_for(&displays[0], 9_000_000)];
        let job = click_job(1400.0, 300.0, displays, frames);
        let error = assemble(&job, &ResolvedMetadata::default()).unwrap_err();
        assert_eq!(error, PacketBuildError::NoRetainedFrame { display_id: 2 });
    }

    #[test]
    fn an_off_display_window_still_crops_against_the_frame_geometry() {
        // The resolved window lies entirely on another display: the
        // window crop degrades to the full selected display rather than
        // failing.
        let display = retina_main();
        let frame = frame_for(&display, 9_000_000);
        let job = click_job(500.0, 300.0, vec![display], vec![frame]);
        let meta = ResolvedMetadata {
            window: Some(window(RectPt::new(5000.0, 50.0, 400.0, 300.0))),
            ..Default::default()
        };
        let packet = assemble(&job, &meta).unwrap();
        assert_eq!(decode_size(&packet.shots.window), (2000, 1200));
    }

    #[test]
    fn an_explicit_post_event_frame_feeds_all_three_shots_with_zero_age() {
        // The worker hands `build_packet` a post-event frame for a
        // key-down (DEC-002): every shot decodes from that frame's
        // pixels, not the pinned frame's, and `frame_age_ms` saturates
        // to 0 because the frame is later than the event.
        let display = DisplayGeometry {
            id: 1,
            frame_pt: RectPt::new(0.0, 0.0, 40.0, 30.0),
            scale: 1.0,
        };
        let pinned = frame_for(&display, 9_000_000); // pixels: 120
        let job = key_job(vec![display.clone()], vec![pinned]);
        let post = Arc::new(FrameData {
            display: display.clone(),
            width_px: 40,
            height_px: 30,
            bytes_per_row: 160,
            ts_ns: job.ts_ns + 60_000_000,
            pixels: [7_u8, 8, 9, 255].repeat(40 * 30),
        });
        let meta = ResolvedMetadata {
            window: Some(window(RectPt::new(4.0, 4.0, 20.0, 20.0))),
            ..Default::default()
        };
        let selected = SelectedFrame {
            display,
            frame: post,
        };
        let packet = build_packet(&job, &meta, &selected).unwrap();

        assert_eq!(packet.frame_age_ms, 0);
        for shot in [&packet.shots.full, &packet.shots.window, &packet.shots.element] {
            let (_, _, rgb) = decode(shot);
            // BGRA (7, 8, 9) decodes to RGB (9, 8, 7) in every shot.
            assert_eq!(&rgb[..3], &[9, 8, 7], "shot pixels come from the post frame");
        }
        assert_eq!(decode_size(&packet.shots.full), (40, 30));
        assert_eq!(decode_size(&packet.shots.window), (20, 20));
    }
}
