//! Pure crop/scale arithmetic for the capture pipeline.
//!
//! Coordinate convention: every input rectangle and point lives in the
//! global top-left-origin display point space shared by `CGEvent`
//! locations, `CGWindowListCopyWindowInfo` bounds, AX positions, and
//! `SCDisplay` frames. Pixel rectangles are local to one display's
//! captured frame: origin at the display's top-left corner, scaled by
//! the display's point-to-pixel scale.
//!
//! Everything here is arithmetic over plain values; no macOS API is
//! touched, so the DEC-008/DEC-011 crop rules are fully testable.

use crate::domain::schema::Rect;

/// The fixed-size fallback element crop in points (about 300x200 pt,
/// DEC-008/DEC-011).
pub const FALLBACK_WIDTH_PT: f64 = 300.0;
pub const FALLBACK_HEIGHT_PT: f64 = 200.0;

/// An AX frame covering at least this fraction of its display is a
/// coarse container (AXWebArea-class): its crop would approximate the
/// full-screen shot, so it carries no element information and the
/// fallback applies.
pub const COARSE_AREA_FRACTION: f64 = 0.6;

/// A point in global display points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointPt {
    pub x: f64,
    pub y: f64,
}

/// A rectangle in global display points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPt {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl RectPt {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn center(&self) -> PointPt {
        PointPt {
            x: self.x + self.w / 2.0,
            y: self.y + self.h / 2.0,
        }
    }

    /// Containment over the half-open extent `[x, x+w) x [y, y+h)`.
    pub fn contains(&self, p: PointPt) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }

    pub fn area(&self) -> f64 {
        if self.w <= 0.0 || self.h <= 0.0 {
            0.0
        } else {
            self.w * self.h
        }
    }

    pub fn intersect(&self, other: &RectPt) -> Option<RectPt> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.w).min(other.x + other.w);
        let bottom = (self.y + self.h).min(other.y + other.h);
        if right > x && bottom > y {
            Some(RectPt::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.w.is_finite() && self.h.is_finite()
    }

    /// The schema's integer point rectangle (rounded).
    pub fn to_schema_rect(&self) -> Rect {
        Rect {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            w: self.w.round() as i32,
            h: self.h.round() as i32,
        }
    }
}

/// One display's identity and geometry: its global point frame and its
/// point-to-pixel scale (2.0 on Retina).
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayGeometry {
    pub id: u32,
    pub frame_pt: RectPt,
    pub scale: f64,
}

impl DisplayGeometry {
    /// The captured frame's pixel width for this display.
    pub fn width_px(&self) -> u32 {
        (self.frame_pt.w * self.scale).round() as u32
    }

    pub fn height_px(&self) -> u32 {
        (self.frame_pt.h * self.scale).round() as u32
    }
}

/// A pixel crop rectangle local to one display's captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The display whose frame contains `point`.
pub fn display_for_point(displays: &[DisplayGeometry], point: PointPt) -> Option<&DisplayGeometry> {
    displays.iter().find(|d| d.frame_pt.contains(point))
}

/// Display selection for a rectangle (spanning windows, DEC-008): the
/// display containing the rectangle's center, else the display with the
/// largest intersection area, else `None`.
pub fn display_for_rect<'a>(
    displays: &'a [DisplayGeometry],
    rect: &RectPt,
) -> Option<&'a DisplayGeometry> {
    if let Some(display) = display_for_point(displays, rect.center()) {
        return Some(display);
    }
    displays
        .iter()
        .filter_map(|d| {
            d.frame_pt
                .intersect(rect)
                .map(|overlap| (d, overlap.area()))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(d, _)| d)
}

/// Converts a global point rectangle into a pixel crop on `display`:
/// intersect with the display frame, translate to display-local points,
/// scale to pixels, and clamp into the frame. `None` when the rectangle
/// misses the display entirely.
pub fn crop_px(display: &DisplayGeometry, target_pt: &RectPt) -> Option<CropPx> {
    if !target_pt.is_finite() || target_pt.w <= 0.0 || target_pt.h <= 0.0 {
        return None;
    }
    let clipped = display.frame_pt.intersect(target_pt)?;
    let scale = display.scale;
    let local_x = (clipped.x - display.frame_pt.x) * scale;
    let local_y = (clipped.y - display.frame_pt.y) * scale;
    let x = local_x.floor().max(0.0) as u32;
    let y = local_y.floor().max(0.0) as u32;
    let right = ((local_x + clipped.w * scale).ceil() as u32).min(display.width_px());
    let bottom = ((local_y + clipped.h * scale).ceil() as u32).min(display.height_px());
    if right <= x || bottom <= y {
        return None;
    }
    Some(CropPx {
        x,
        y,
        w: right - x,
        h: bottom - y,
    })
}

/// The full-frame crop for a display.
pub fn full_display_px(display: &DisplayGeometry) -> CropPx {
    CropPx {
        x: 0,
        y: 0,
        w: display.width_px(),
        h: display.height_px(),
    }
}

/// The fixed-size fallback rectangle (about 300x200 pt): centered at
/// `center`, shifted and clamped so it stays inside `container`
/// intersected with the display frame. When the container is smaller
/// than the fixed size, the fallback shrinks to the container.
pub fn fallback_rect_pt(
    center: PointPt,
    container: &RectPt,
    display: &DisplayGeometry,
) -> RectPt {
    let bounds = container
        .intersect(&display.frame_pt)
        .unwrap_or(display.frame_pt);
    let w = FALLBACK_WIDTH_PT.min(bounds.w);
    let h = FALLBACK_HEIGHT_PT.min(bounds.h);
    let x = (center.x - w / 2.0)
        .max(bounds.x)
        .min(bounds.x + bounds.w - w);
    let y = (center.y - h / 2.0)
        .max(bounds.y)
        .min(bounds.y + bounds.h - h);
    RectPt::new(x, y, w, h)
}

/// The deterministic implausible-frame rule (DEC-008/DEC-011 fallback
/// trigger). An AX frame is implausible when any of these holds:
///
/// 1. It is non-finite or degenerate (width or height below 2 pt).
/// 2. It does not intersect the selected display at all.
/// 3. For clicks: it does not contain the click point (an AX hit whose
///    frame excludes the clicked location is coordinate garbage).
/// 4. Its on-display area covers at least [`COARSE_AREA_FRACTION`] of
///    the display: a coarse AXWebArea-class container whose crop would
///    approximate the full-screen shot.
pub fn is_implausible(
    frame_pt: &RectPt,
    display: &DisplayGeometry,
    click_point: Option<PointPt>,
) -> bool {
    if !frame_pt.is_finite() || frame_pt.w < 2.0 || frame_pt.h < 2.0 {
        return true;
    }
    let Some(on_display) = frame_pt.intersect(&display.frame_pt) else {
        return true;
    };
    if let Some(point) = click_point {
        if !frame_pt.contains(point) {
            return true;
        }
    }
    on_display.area() >= COARSE_AREA_FRACTION * display.frame_pt.area()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retina_main() -> DisplayGeometry {
        DisplayGeometry {
            id: 1,
            frame_pt: RectPt::new(0.0, 0.0, 1728.0, 1117.0),
            scale: 2.0,
        }
    }

    /// A secondary 1x display to the left, above the main origin:
    /// negative global origin on both axes.
    fn negative_origin_side() -> DisplayGeometry {
        DisplayGeometry {
            id: 2,
            frame_pt: RectPt::new(-1920.0, -300.0, 1920.0, 1080.0),
            scale: 1.0,
        }
    }

    #[test]
    fn retina_crop_scales_points_to_pixels() {
        let display = retina_main();
        let crop = crop_px(&display, &RectPt::new(100.0, 50.0, 800.0, 600.0)).unwrap();
        assert_eq!(
            crop,
            CropPx {
                x: 200,
                y: 100,
                w: 1600,
                h: 1200,
            },
        );
        assert_eq!(display.width_px(), 3456);
        assert_eq!(display.height_px(), 2234);
    }

    #[test]
    fn negative_origin_display_normalizes_to_local_pixels() {
        let display = negative_origin_side();
        let crop = crop_px(&display, &RectPt::new(-1820.0, -200.0, 400.0, 300.0)).unwrap();
        assert_eq!(
            crop,
            CropPx {
                x: 100,
                y: 100,
                w: 400,
                h: 300,
            },
        );
    }

    #[test]
    fn window_partly_outside_the_display_clamps_to_the_visible_part() {
        let display = retina_main();
        // Window hangs off the top-left corner of the display.
        let crop = crop_px(&display, &RectPt::new(-100.0, -50.0, 300.0, 200.0)).unwrap();
        assert_eq!(
            crop,
            CropPx {
                x: 0,
                y: 0,
                w: 400,
                h: 300,
            },
        );
        // Window fully off-display yields no crop.
        assert_eq!(
            crop_px(&display, &RectPt::new(-500.0, -400.0, 300.0, 200.0)),
            None,
        );
    }

    #[test]
    fn degenerate_and_non_finite_rects_yield_no_crop() {
        let display = retina_main();
        assert_eq!(crop_px(&display, &RectPt::new(10.0, 10.0, 0.0, 100.0)), None);
        assert_eq!(
            crop_px(&display, &RectPt::new(f64::NAN, 10.0, 50.0, 100.0)),
            None,
        );
    }

    #[test]
    fn display_for_rect_prefers_the_center_display_on_mixed_scales() {
        let displays = vec![retina_main(), negative_origin_side()];
        // A window spanning the seam whose center sits on the 1x display.
        let spanning = RectPt::new(-700.0, 100.0, 1000.0, 400.0);
        assert_eq!(display_for_rect(&displays, &spanning).unwrap().id, 2);
        // Center on the Retina display selects it instead.
        let spanning = RectPt::new(-300.0, 100.0, 1000.0, 400.0);
        assert_eq!(display_for_rect(&displays, &spanning).unwrap().id, 1);
    }

    #[test]
    fn display_for_rect_falls_back_to_largest_overlap_when_center_is_off_screen() {
        let displays = vec![retina_main(), negative_origin_side()];
        // Center is below both displays; the overlap with display 1 wins.
        let rect = RectPt::new(500.0, 900.0, 400.0, 600.0);
        assert_eq!(display_for_rect(&displays, &rect).unwrap().id, 1);
        // No overlap at all.
        let rect = RectPt::new(5000.0, 5000.0, 100.0, 100.0);
        assert!(display_for_rect(&displays, &rect).is_none());
    }

    #[test]
    fn fallback_rect_is_centered_and_clamped_inside_the_container() {
        let display = retina_main();
        let window = RectPt::new(100.0, 50.0, 800.0, 600.0);
        // Centered fit.
        let rect = fallback_rect_pt(PointPt { x: 500.0, y: 350.0 }, &window, &display);
        assert_eq!(rect, RectPt::new(350.0, 250.0, 300.0, 200.0));
        // Near the container corner: shifted inside, size kept.
        let rect = fallback_rect_pt(PointPt { x: 110.0, y: 60.0 }, &window, &display);
        assert_eq!(rect, RectPt::new(100.0, 50.0, 300.0, 200.0));
    }

    #[test]
    fn fallback_rect_shrinks_to_a_small_container() {
        let display = retina_main();
        let tiny = RectPt::new(10.0, 10.0, 120.0, 90.0);
        let rect = fallback_rect_pt(PointPt { x: 20.0, y: 20.0 }, &tiny, &display);
        assert_eq!(rect, RectPt::new(10.0, 10.0, 120.0, 90.0));
    }

    #[test]
    fn key_down_fallback_centers_in_focused_window_bounds() {
        // DEC-008: the key-down fallback centers on the focused window.
        let display = retina_main();
        let window = RectPt::new(200.0, 100.0, 1000.0, 800.0);
        let rect = fallback_rect_pt(window.center(), &window, &display);
        assert_eq!(rect.center(), window.center());
        assert_eq!((rect.w, rect.h), (300.0, 200.0));
    }

    #[test]
    fn implausible_rule_flags_degenerate_missing_and_coarse_frames() {
        let display = retina_main();
        let click = PointPt { x: 500.0, y: 400.0 };
        // Degenerate.
        assert!(is_implausible(
            &RectPt::new(490.0, 390.0, 1.0, 40.0),
            &display,
            Some(click),
        ));
        // Non-finite.
        assert!(is_implausible(
            &RectPt::new(f64::INFINITY, 390.0, 40.0, 40.0),
            &display,
            Some(click),
        ));
        // Entirely off the selected display.
        assert!(is_implausible(
            &RectPt::new(-4000.0, 390.0, 40.0, 40.0),
            &display,
            Some(click),
        ));
        // Frame that excludes the click point.
        assert!(is_implausible(
            &RectPt::new(600.0, 500.0, 40.0, 40.0),
            &display,
            Some(click),
        ));
        // Coarse AXWebArea-class frame covering most of the display.
        assert!(is_implausible(
            &RectPt::new(0.0, 25.0, 1728.0, 1092.0),
            &display,
            Some(PointPt { x: 500.0, y: 400.0 }),
        ));
    }

    #[test]
    fn plausible_frames_pass_the_rule() {
        let display = retina_main();
        let click = PointPt { x: 500.0, y: 380.0 };
        // A button under the pointer.
        assert!(!is_implausible(
            &RectPt::new(480.0, 360.0, 80.0, 32.0),
            &display,
            Some(click),
        ));
        // A large-but-not-coarse text area without a click constraint
        // (key-down path): 800x542 pt is ~22% of the display.
        assert!(!is_implausible(
            &RectPt::new(120.0, 80.0, 800.0, 542.0),
            &display,
            None,
        ));
    }

    #[test]
    fn schema_rect_rounds_to_integer_points() {
        assert_eq!(
            RectPt::new(10.4, 20.6, 300.5, 199.5).to_schema_rect(),
            Rect {
                x: 10,
                y: 21,
                w: 301,
                h: 200,
            },
        );
    }
}
