//! The worker-side metadata-resolution seam.
//!
//! The single capture worker resolves window and AX metadata before PNG
//! encoding. The production implementation
//! ([`crate::capture::macos::MacosResolver`]) talks to
//! `CGWindowListCopyWindowInfo` and the AX API; tests substitute a fake
//! at this seam because the AX answers of a live desktop are
//! nondeterministic.

use crate::capture::geometry::PointPt;
use crate::capture::packets::ResolvedMetadata;

pub trait MetadataResolver: Send {
    /// Click resolution: `CGWindowListCopyWindowInfo` hit test at the
    /// click point plus the AX element at that point.
    fn resolve_click(&mut self, point: PointPt) -> ResolvedMetadata;

    /// Key-down resolution (DEC-008): the frontmost application's
    /// focused window plus the system focused UI element.
    fn resolve_key_down(&mut self) -> ResolvedMetadata;
}
