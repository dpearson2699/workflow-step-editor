//! macOS platform backends for the capture pipeline.
//!
//! These modules own every macOS API call: ScreenCaptureKit streams
//! ([`stream`]), display enumeration and reconfiguration ([`displays`]),
//! and window/AX metadata resolution ([`ax`]). They sit behind the
//! [`crate::capture::streams::StreamBackend`] and
//! [`crate::capture::resolver::MetadataResolver`] seams, so the pure
//! pipeline logic is exercised without a live desktop.

pub mod ax;
pub mod displays;
pub mod stream;

pub use ax::MacosResolver;
pub use displays::{enumerate_displays, DisplayReconfigurationObserver};
pub use stream::MacosStreamBackend;
