//! Shared deterministic test doubles for the recording module: a fixed
//! clock, a fake permission source behind the real PR-01
//! `PermissionService`, controllable step sinks, and schema fixtures.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use crate::domain::schema::{
    CaptureMeta, ElementInfo, ElementSource, Event, EventKind, KeyInfo, Modifier, MouseButton,
    Pos, Rect, ShotPaths, WindowInfo,
};
use crate::permissions::{NativeStatus, PermissionService, PermissionSource};
use crate::recording::channel::{LiveEnvelope, StepSink};
use crate::recording::clock::Clock;
use crate::recording::coordinator::PermissionGate;
use crate::recording::fake_pipeline::fixture_shots;
use crate::recording::store::ShotPayloads;

/// A clock that always returns the same instant.
pub struct FixedClock(DateTime<Utc>);

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

pub fn fixed_clock(instant: DateTime<Utc>) -> Arc<FixedClock> {
    Arc::new(FixedClock(instant))
}

/// Fake native permission source with fixed statuses; requests return
/// the same status as checks.
pub struct FakePermissionSource {
    pub input_monitoring: NativeStatus,
    pub accessibility: NativeStatus,
    pub screen_recording: NativeStatus,
}

impl PermissionSource for FakePermissionSource {
    fn input_monitoring_status(&mut self) -> NativeStatus {
        self.input_monitoring
    }

    fn request_input_monitoring(&mut self) -> NativeStatus {
        self.input_monitoring
    }

    fn accessibility_status(&mut self) -> NativeStatus {
        self.accessibility
    }

    fn request_accessibility(&mut self) -> NativeStatus {
        self.accessibility
    }

    fn screen_recording_status(&mut self) -> NativeStatus {
        self.screen_recording
    }

    fn request_screen_recording(&mut self) -> NativeStatus {
        self.screen_recording
    }
}

/// A permission gate over the real `PermissionService` with a fake
/// native source: coordinator tests exercise the PR-01 seam itself.
pub fn gate_with_status(
    input_monitoring: NativeStatus,
    accessibility: NativeStatus,
    screen_recording: NativeStatus,
) -> Arc<dyn PermissionGate> {
    Arc::new(Mutex::new(PermissionService::new(FakePermissionSource {
        input_monitoring,
        accessibility,
        screen_recording,
    })))
}

pub fn granted_gate() -> Arc<dyn PermissionGate> {
    gate_with_status(
        NativeStatus::Granted,
        NativeStatus::Granted,
        NativeStatus::Granted,
    )
}

/// Read handle over a [`TestSink`]'s emissions.
#[derive(Clone)]
pub struct SinkLog(Arc<Mutex<Vec<LiveEnvelope>>>);

impl SinkLog {
    pub fn items(&self) -> Vec<LiveEnvelope> {
        self.0.lock().unwrap().clone()
    }
}

/// A gate that blocks every `emit` until opened, and reports when a
/// worker is waiting on it.
#[derive(Default)]
pub struct EmitGate {
    state: Mutex<EmitGateState>,
    condvar: Condvar,
}

#[derive(Default)]
struct EmitGateState {
    open: bool,
    waiting: u32,
}

impl EmitGate {
    /// Opens the gate permanently.
    pub fn open(&self) {
        self.state.lock().unwrap().open = true;
        self.condvar.notify_all();
    }

    /// Blocks until at least one emitter waits on the closed gate.
    pub fn wait_for_waiter(&self) {
        let mut state = self.state.lock().unwrap();
        while state.waiting == 0 {
            state = self.condvar.wait(state).unwrap();
        }
    }

    fn pass(&self) {
        let mut state = self.state.lock().unwrap();
        state.waiting += 1;
        self.condvar.notify_all();
        while !state.open {
            state = self.condvar.wait(state).unwrap();
        }
        state.waiting -= 1;
    }
}

type StepObserver = Box<dyn Fn(&crate::domain::schema::Step) + Send>;

/// Controllable [`StepSink`]: records every envelope and optionally
/// fails, blocks on a gate, or runs an observer on each step emission.
pub struct TestSink {
    log: Arc<Mutex<Vec<LiveEnvelope>>>,
    fail: bool,
    gate: Option<Arc<EmitGate>>,
    step_observer: Option<StepObserver>,
}

impl TestSink {
    /// A sink that records and succeeds.
    pub fn recording() -> (Self, SinkLog) {
        let log = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                log: log.clone(),
                fail: false,
                gate: None,
                step_observer: None,
            },
            SinkLog(log),
        )
    }

    /// A disconnected channel: records for inspection but returns an
    /// error on every emission.
    pub fn failing() -> (Self, SinkLog) {
        let (mut sink, log) = Self::recording();
        sink.fail = true;
        (sink, log)
    }

    pub fn with_gate(mut self, gate: Arc<EmitGate>) -> Self {
        self.gate = Some(gate);
        self
    }

    pub fn with_step_observer(
        mut self,
        observer: impl Fn(&crate::domain::schema::Step) + Send + 'static,
    ) -> Self {
        self.step_observer = Some(Box::new(observer));
        self
    }
}

impl StepSink for TestSink {
    fn emit(&self, item: LiveEnvelope) -> Result<(), String> {
        if let Some(gate) = &self.gate {
            gate.pass();
        }
        if let (Some(observer), LiveEnvelope::Step { step }) = (&self.step_observer, &item) {
            observer(step);
        }
        self.log.lock().unwrap().push(item);
        if self.fail {
            Err("channel disconnected".to_owned())
        } else {
            Ok(())
        }
    }
}

/// Polls `condition` until it holds or `timeout` elapses.
pub fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A schema-v1-shaped click event with canonical shot paths.
pub fn sample_click_event(event_id: &str) -> Event {
    Event {
        id: event_id.to_owned(),
        ts: "2026-08-16T22:31:05.123Z".to_owned(),
        kind: EventKind::Click,
        display_id: 1,
        pos: Pos { x: 512.0, y: 384.0 },
        button: Some(MouseButton::Left),
        key: None,
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
        shots: ShotPaths::for_event(event_id),
        capture: CaptureMeta { frame_age_ms: 12 },
    }
}

/// A schema-v1-shaped plain key-down event with canonical shot paths.
pub fn sample_key_event(event_id: &str) -> Event {
    Event {
        kind: EventKind::KeyDown,
        button: None,
        key: Some(KeyInfo {
            key_code: 4,
            chars: "h".into(),
            modifiers: vec![Modifier::Shift],
        }),
        ..sample_click_event(event_id)
    }
}

/// The fixture screenshot triple.
pub fn sample_shots() -> ShotPayloads {
    fixture_shots()
}
