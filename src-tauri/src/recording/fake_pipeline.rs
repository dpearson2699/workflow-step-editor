//! A deterministic fake `CapturePipeline`.
//!
//! The fake emits exactly the packets its controller is told to emit, in
//! that order, on the caller's thread. It supplies fixture PNG payloads,
//! never touches a macOS API, and supports scripted start failure and a
//! start gate for deterministic concurrency tests.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use crate::domain::schema::{
    ElementInfo, ElementSource, KeyInfo, Modifier, MouseButton, Pos, Rect, WindowInfo,
};
use crate::recording::pipeline::{CapturePipeline, CapturePacket, PacketEmitter, PacketInput};
use crate::recording::store::ShotPayloads;

/// Minimal valid 1x1 RGB PNGs used as the fixture screenshot payloads.
/// The three roles carry distinct bytes (red, green, blue pixels) so a
/// swapped full/window/element write cannot pass the store-seam tests.
pub const FIXTURE_PNG_FULL: [u8; 69] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
    0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8,
    0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0xc9, 0xfe, 0x92, 0xef, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

pub const FIXTURE_PNG_WINDOW: [u8; 69] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
    0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60,
    0xf8, 0xcf, 0x00, 0x00, 0x02, 0x02, 0x01, 0x00, 0x7b, 0x09, 0x81, 0x78, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

pub const FIXTURE_PNG_ELEMENT: [u8; 69] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
    0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60,
    0x60, 0xf8, 0x0f, 0x00, 0x01, 0x03, 0x01, 0x00, 0x08, 0x89, 0xc2, 0xec, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// A fixture screenshot triple with role-distinct payload bytes.
pub fn fixture_shots() -> ShotPayloads {
    ShotPayloads {
        full: FIXTURE_PNG_FULL.to_vec(),
        window: FIXTURE_PNG_WINDOW.to_vec(),
        element: FIXTURE_PNG_ELEMENT.to_vec(),
    }
}

/// A deterministic click packet on a titled TextEdit button.
pub fn click_packet() -> CapturePacket {
    CapturePacket {
        input: PacketInput::Click {
            button: MouseButton::Left,
        },
        pos: Pos { x: 512.0, y: 384.0 },
        display_id: 1,
        window: Some(WindowInfo {
            app: "TextEdit".into(),
            title: "Untitled".into(),
            pid: 871,
            bounds: Rect {
                x: 100,
                y: 50,
                w: 800,
                h: 600,
            },
        }),
        element: ElementInfo {
            role: Some("AXButton".into()),
            title: Some("OK".into()),
            frame: Rect {
                x: 480,
                y: 360,
                w: 80,
                h: 32,
            },
            source: ElementSource::Ax,
        },
        frontmost_app: Some("TextEdit".into()),
        frame_age_ms: 12,
        shots: fixture_shots(),
    }
}

/// A deterministic Cmd+S key-down packet in TextEdit.
pub fn key_packet() -> CapturePacket {
    CapturePacket {
        input: PacketInput::KeyDown {
            key: KeyInfo {
                key_code: 1,
                chars: "s".into(),
                modifiers: vec![Modifier::Command],
            },
        },
        pos: Pos { x: 512.0, y: 384.0 },
        display_id: 1,
        window: Some(WindowInfo {
            app: "TextEdit".into(),
            title: "Untitled".into(),
            pid: 871,
            bounds: Rect {
                x: 100,
                y: 50,
                w: 800,
                h: 600,
            },
        }),
        element: ElementInfo {
            role: Some("AXTextArea".into()),
            title: None,
            frame: Rect {
                x: 120,
                y: 80,
                w: 760,
                h: 540,
            },
            source: ElementSource::Ax,
        },
        frontmost_app: Some("TextEdit".into()),
        frame_age_ms: 9,
        shots: fixture_shots(),
    }
}

#[derive(Default)]
struct ControllerState {
    emitter: Mutex<Option<PacketEmitter>>,
    start_error: Mutex<Option<String>>,
    start_gate: StartGate,
    stop_count: AtomicU32,
}

#[derive(Default)]
struct StartGate {
    state: Mutex<StartGateState>,
    condvar: Condvar,
}

#[derive(Default)]
struct StartGateState {
    hold: bool,
    entered: bool,
}

/// Test-side handle over a [`FakeCapturePipeline`].
#[derive(Clone, Default)]
pub struct FakeController {
    state: Arc<ControllerState>,
}

impl FakeController {
    /// Scripts the next `start` call to fail with `error`.
    pub fn fail_start(&self, error: impl Into<String>) {
        *self.state.start_error.lock().unwrap() = Some(error.into());
    }

    /// Makes the next `start` call block until [`Self::release_start`].
    pub fn hold_start(&self) {
        self.state.start_gate.state.lock().unwrap().hold = true;
    }

    /// Blocks until a held `start` call has entered the gate.
    pub fn wait_for_start_entered(&self) {
        let gate = &self.state.start_gate;
        let mut state = gate.state.lock().unwrap();
        while !state.entered {
            state = gate.condvar.wait(state).unwrap();
        }
    }

    /// Releases a held `start` call.
    pub fn release_start(&self) {
        let gate = &self.state.start_gate;
        gate.state.lock().unwrap().hold = false;
        gate.condvar.notify_all();
    }

    /// Emits one packet through the running pipeline, in call order.
    /// Ignored when the pipeline is not running (stale callback).
    pub fn emit(&self, packet: CapturePacket) {
        if let Some(emitter) = self.state.emitter.lock().unwrap().as_ref() {
            emitter.packet(packet);
        }
    }

    /// Reports a mid-recording pipeline failure.
    pub fn fail(&self, error: impl Into<String>) {
        if let Some(emitter) = self.state.emitter.lock().unwrap().as_ref() {
            emitter.failed(error.into());
        }
    }

    /// How many times `stop` ran.
    pub fn stop_count(&self) -> u32 {
        self.state.stop_count.load(Ordering::SeqCst)
    }

    /// True while a started pipeline holds its emitter.
    pub fn is_running(&self) -> bool {
        self.state.emitter.lock().unwrap().is_some()
    }
}

/// Deterministic fake capture pipeline. Build one per session with
/// [`FakeCapturePipeline::new`] and drive it through the returned
/// controller.
pub struct FakeCapturePipeline {
    state: Arc<ControllerState>,
}

impl FakeCapturePipeline {
    pub fn new() -> (Self, FakeController) {
        let controller = FakeController::default();
        (
            Self {
                state: controller.state.clone(),
            },
            controller,
        )
    }
}

impl CapturePipeline for FakeCapturePipeline {
    fn start(&mut self, emitter: PacketEmitter) -> Result<(), String> {
        {
            let gate = &self.state.start_gate;
            let mut state = gate.state.lock().unwrap();
            state.entered = true;
            gate.condvar.notify_all();
            while state.hold {
                state = gate.condvar.wait(state).unwrap();
            }
        }
        if let Some(error) = self.state.start_error.lock().unwrap().take() {
            return Err(error);
        }
        *self.state.emitter.lock().unwrap() = Some(emitter);
        Ok(())
    }

    fn stop(&mut self) {
        self.state.stop_count.fetch_add(1, Ordering::SeqCst);
        *self.state.emitter.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::recording::pipeline::PipelineEvent;

    use super::*;

    #[test]
    fn emits_packets_in_controller_order_and_ignores_after_stop() {
        let (mut pipeline, controller) = FakeCapturePipeline::new();
        let (tx, rx) = mpsc::channel();
        let emitter = PacketEmitter::new(move |event| {
            let _ = tx.send(event);
        });
        pipeline.start(emitter).unwrap();

        controller.emit(click_packet());
        controller.emit(key_packet());
        pipeline.stop();
        controller.emit(click_packet());

        let received: Vec<PipelineEvent> = rx.try_iter().collect();
        assert_eq!(
            received,
            vec![
                PipelineEvent::Packet(Box::new(click_packet())),
                PipelineEvent::Packet(Box::new(key_packet())),
            ],
        );
        assert_eq!(controller.stop_count(), 1);
    }

    /// The role-to-path mapping proof in the store tests is only as strong
    /// as the payloads are distinguishable.
    #[test]
    fn fixture_shot_payloads_are_pairwise_distinct() {
        let shots = fixture_shots();
        assert_ne!(shots.full, shots.window);
        assert_ne!(shots.full, shots.element);
        assert_ne!(shots.window, shots.element);
    }

    #[test]
    fn scripted_start_failure_retains_nothing() {
        let (mut pipeline, controller) = FakeCapturePipeline::new();
        controller.fail_start("no tap");
        let emitter = PacketEmitter::new(|_| {});
        assert_eq!(pipeline.start(emitter), Err("no tap".to_owned()));
        assert!(!controller.is_running());
    }
}
