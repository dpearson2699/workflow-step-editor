//! The `CapturePipeline` platform boundary (DEC-001).
//!
//! A pipeline observes global input while a recording is active and emits
//! ordered capture packets. Each packet carries the raw input facts, the
//! resolved metadata (window, element, frontmost application), the
//! buffered frame age, and one encoded screenshot triple. PR-03 supplies
//! the real macOS adapter; this slice ships the trait, a deterministic
//! fake, and a placeholder that fails to start.

use std::sync::Arc;

use crate::domain::schema::{ElementInfo, KeyInfo, MouseButton, Pos, WindowInfo};
use crate::recording::store::ShotPayloads;

/// The raw input facts of one capture packet.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketInput {
    Click { button: MouseButton },
    KeyDown { key: KeyInfo },
}

/// One ordered capture packet from a pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturePacket {
    pub input: PacketInput,
    pub pos: Pos,
    pub display_id: u32,
    /// The hit window, or `None` when no window resolves (DEC-011).
    pub window: Option<WindowInfo>,
    pub element: ElementInfo,
    /// The frontmost application name at capture time, used for the
    /// null-window title fallback.
    pub frontmost_app: Option<String>,
    /// Age of the buffered pre-event frame when the event fired.
    pub frame_age_ms: u64,
    /// The encoded screenshot triple.
    pub shots: ShotPayloads,
}

/// What a running pipeline can report to its consumer.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineEvent {
    Packet(Box<CapturePacket>),
    /// The pipeline failed mid-recording (tap disabled, stream lost).
    /// Exactly one fail-stop transition follows, no matter how many
    /// failures are reported.
    Failed(String),
}

/// The consumer end handed to a pipeline at start. Delivery after the
/// recording finalizes is silently ignored (stale callbacks).
#[derive(Clone)]
pub struct PacketEmitter {
    deliver: Arc<dyn Fn(PipelineEvent) + Send + Sync>,
}

impl PacketEmitter {
    pub fn new(deliver: impl Fn(PipelineEvent) + Send + Sync + 'static) -> Self {
        Self {
            deliver: Arc::new(deliver),
        }
    }

    pub fn packet(&self, packet: CapturePacket) {
        (self.deliver)(PipelineEvent::Packet(Box::new(packet)));
    }

    pub fn failed(&self, error: impl Into<String>) {
        (self.deliver)(PipelineEvent::Failed(error.into()));
    }
}

impl std::fmt::Debug for PacketEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketEmitter").finish_non_exhaustive()
    }
}

/// The platform capture boundary.
///
/// Contract: `start` either takes ownership of the emitter and begins
/// emitting ordered packets, or returns an error and retains nothing.
/// `stop` is idempotent, may be called after a reported failure, and
/// stops further emission; packets already handed to the emitter stay
/// valid. One pipeline instance serves one recording session.
pub trait CapturePipeline: Send {
    fn start(&mut self, emitter: PacketEmitter) -> Result<(), String>;
    fn stop(&mut self);
}

/// Placeholder pipeline for builds without a platform adapter. `start`
/// always fails, so `start_recording` rolls back cleanly until PR-03
/// wires the macOS capture adapter.
pub struct UnavailablePipeline;

impl CapturePipeline for UnavailablePipeline {
    fn start(&mut self, _emitter: PacketEmitter) -> Result<(), String> {
        Err("capture pipeline is not available in this build; the macOS adapter arrives with PR-03"
            .to_owned())
    }

    fn stop(&mut self) {}
}
