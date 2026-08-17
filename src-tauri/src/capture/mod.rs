//! The real macOS capture adapter for the PR-02 `CapturePipeline`
//! boundary.
//!
//! The pipeline is one live flow behind the single trait seam the
//! coordinator already consumes: a ListenOnly `CGEventTap` copies each
//! event and pins an immutable pre-event frame snapshot ([`tap`],
//! [`broker`]); per-display ScreenCaptureKit streams keep the buffered
//! frames warm ([`streams`], [`macos`]); a bounded queue ([`queue`])
//! feeds one ordered worker ([`worker`]) that resolves window/AX
//! metadata ([`resolver`], [`macos`]), assembles the packet with pure
//! crop geometry ([`geometry`], [`packets`]), encodes the screenshot
//! triple ([`encoder`]), and emits through the health/failure adapter
//! ([`health`]). Only [`pipeline::MacosCapturePipeline`] is observable
//! in production, and only through the coordinator's command layer.
//!
//! Non-macOS builds carry only the pure, platform-independent modules
//! (geometry, host clock, broker, queue, encoder, packet assembly, and
//! the health adapter), which the coordinator never instantiates off
//! macOS.

pub mod broker;
pub mod encoder;
pub mod geometry;
pub mod health;
pub mod hostclock;
pub mod packets;
pub mod queue;
pub mod resolver;
pub mod streams;
pub mod worker;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub mod pipeline;
#[cfg(target_os = "macos")]
pub mod tap;

#[cfg(target_os = "macos")]
pub use pipeline::MacosCapturePipeline;
