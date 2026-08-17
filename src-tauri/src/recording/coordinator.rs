//! The recording coordinator: the `Idle -> Starting -> Recording ->
//! Stopping/Failed -> Idle` state machine with exactly one active
//! recording and exactly one terminal outcome per session.
//!
//! Start is permission-gated through the PR-01 permission seam. Stop and
//! fail-stop share one finalization owner — the session worker — which
//! drains accepted capture work before the manifest saves and the
//! terminal envelope emits, and ignores stale callbacks afterwards.
//! Startup failure before the workflow id publishes rolls the created
//! folder back through the store guard.
//!
//! Pinned invariant: a step is published to the channel only after its
//! event line and all three screenshots are committed.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::domain::parser::parse_step;
use crate::domain::schema::{CaptureMeta, Event, EventKind, Manifest, ShotPaths};
use crate::permissions::{PermissionReport, PermissionService, PermissionSource, PermissionStatus};
use crate::recording::channel::{LiveEnvelope, StepSink};
use crate::recording::clock::{default_workflow_name, event_timestamp, Clock};
use crate::recording::pipeline::{CapturePacket, CapturePipeline, PacketEmitter, PacketInput, PipelineEvent};
use crate::recording::store::{LoadedWorkflow, StoreError, WorkflowStore, WorkflowSummary};

/// The permission seam the coordinator gates start on. Production wires
/// the shared PR-01 `PermissionService`; tests substitute a fake source
/// behind the same service.
pub trait PermissionGate: Send + Sync {
    fn report(&self) -> PermissionReport;
}

/// The production gate: the same mutex-serialized `PermissionService` the
/// permission commands use.
impl<S: PermissionSource + Send> PermissionGate for Mutex<PermissionService<S>> {
    fn report(&self) -> PermissionReport {
        self.lock()
            .expect("permission service mutex poisoned")
            .check_all()
    }
}

/// Builds one pipeline per recording session.
pub type PipelineFactory = Box<dyn Fn() -> Box<dyn CapturePipeline> + Send + Sync>;

/// Errors from the capture-lifecycle service.
#[derive(Debug)]
pub enum RecordingError {
    /// A recording is already active or starting.
    AlreadyActive,
    /// Stop was called while start was still in progress; the workflow id
    /// has not been published yet, so there is nothing to stop.
    StartInProgress,
    /// Stop was called while a finalization was already in progress.
    StopInProgress,
    /// Stop was called with no active recording.
    NotRecording,
    /// One or more of the three required permissions did not pass.
    PermissionsMissing(PermissionReport),
    Store(StoreError),
    PipelineStart(String),
    /// The recording stopped, but finalization could not persist the
    /// manifest; committed events and screenshots remain on disk.
    FinalizationFailed(String),
    Internal(String),
}

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyActive => write!(f, "a recording is already active or starting"),
            Self::StartInProgress => write!(f, "recording start is in progress"),
            Self::StopInProgress => write!(f, "recording finalization is in progress"),
            Self::NotRecording => write!(f, "no active recording"),
            Self::PermissionsMissing(report) => write!(
                f,
                "recording requires Input Monitoring, Accessibility, and Screen Recording; \
                 current status: input_monitoring={:?}, accessibility={:?}, screen_recording={:?}",
                report.input_monitoring, report.accessibility, report.screen_recording,
            ),
            Self::Store(error) => write!(f, "storage error: {error}"),
            Self::PipelineStart(error) => write!(f, "capture pipeline failed to start: {error}"),
            Self::FinalizationFailed(error) => {
                write!(f, "recording finalization failed: {error}")
            }
            Self::Internal(error) => write!(f, "internal recording error: {error}"),
        }
    }
}

impl std::error::Error for RecordingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StoreError> for RecordingError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

enum SessionMsg {
    Pipeline(PipelineEvent),
    Stop,
}

type SharedPipeline = Arc<Mutex<Box<dyn CapturePipeline>>>;

/// Signal set once `start_recording` has installed the session in the
/// phase. The worker waits on it before its first ownership check, so a
/// pipeline failure arriving during startup cannot finalize against a
/// not-yet-installed session and leave a dead session behind.
#[derive(Default)]
struct InstallSignal {
    installed: Mutex<bool>,
    condvar: Condvar,
}

impl InstallSignal {
    fn set(&self) {
        *self.installed.lock().expect("install signal mutex poisoned") = true;
        self.condvar.notify_all();
    }

    fn wait(&self) {
        let mut installed = self.installed.lock().expect("install signal mutex poisoned");
        while !*installed {
            installed = self
                .condvar
                .wait(installed)
                .expect("install signal mutex poisoned");
        }
    }
}

struct Session {
    workflow_id: String,
    tx: Sender<SessionMsg>,
    worker: JoinHandle<()>,
    pipeline: SharedPipeline,
    /// Set by the worker when finalization could not save the manifest.
    save_failure: Arc<Mutex<Option<String>>>,
}

enum Phase {
    Idle,
    Starting,
    Recording(Session),
    /// A stop owns the session and is joining the worker.
    Stopping,
    /// The worker owns a fail-stop finalization.
    Failed,
}

/// The capture-lifecycle application service.
pub struct RecordingCoordinator {
    store: Arc<dyn WorkflowStore>,
    gate: Arc<dyn PermissionGate>,
    factory: PipelineFactory,
    clock: Arc<dyn Clock>,
    state: Arc<Mutex<Phase>>,
}

impl RecordingCoordinator {
    pub fn new(
        store: Arc<dyn WorkflowStore>,
        gate: Arc<dyn PermissionGate>,
        factory: PipelineFactory,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            gate,
            factory,
            clock,
            state: Arc::new(Mutex::new(Phase::Idle)),
        }
    }

    /// Starts a recording and returns the published workflow id. A
    /// missing or blank name defaults to a timestamp. Start refuses
    /// unless all three permissions pass, and only one recording can be
    /// active: concurrent starts yield exactly one success.
    pub fn start_recording(
        &self,
        name: Option<&str>,
        sink: Box<dyn StepSink>,
    ) -> Result<String, RecordingError> {
        {
            let mut phase = self.lock_state();
            match *phase {
                Phase::Idle => *phase = Phase::Starting,
                _ => return Err(RecordingError::AlreadyActive),
            }
        }
        match self.start_session(name, sink) {
            Ok((session, guard, install)) => {
                *self.lock_state() = Phase::Recording(session);
                // The worker defers its first ownership check until the
                // session is installed, so an immediate pipeline failure
                // finalizes against the installed session instead of
                // leaving a dead one behind.
                install.set();
                // Publish only after the session is installed; every
                // earlier failure rolled the folder back via the guard.
                Ok(guard.publish())
            }
            Err(error) => {
                *self.lock_state() = Phase::Idle;
                Err(error)
            }
        }
    }

    fn start_session(
        &self,
        name: Option<&str>,
        sink: Box<dyn StepSink>,
    ) -> Result<
        (
            Session,
            crate::recording::store::UnpublishedWorkflow,
            Arc<InstallSignal>,
        ),
        RecordingError,
    > {
        let report = self.gate.report();
        if !all_granted(&report) {
            return Err(RecordingError::PermissionsMissing(report));
        }

        let name = match name.map(str::trim) {
            Some(trimmed) if !trimmed.is_empty() => trimmed.to_owned(),
            _ => default_workflow_name(self.clock.now()),
        };
        let created = self.store.create(&name)?;
        let manifest = created.manifest;
        let guard = created.guard;
        let workflow_id = guard.id().to_owned();

        let mut pipeline = (self.factory)();
        let (tx, rx) = mpsc::channel();
        let emitter_tx = tx.clone();
        let emitter = PacketEmitter::new(move |event| {
            // A send after finalization is a stale callback; ignore it.
            let _ = emitter_tx.send(SessionMsg::Pipeline(event));
        });
        pipeline
            .start(emitter)
            // Returning drops `guard`, which rolls the folder back.
            .map_err(RecordingError::PipelineStart)?;
        let pipeline: SharedPipeline = Arc::new(Mutex::new(pipeline));

        let install = Arc::new(InstallSignal::default());
        let save_failure: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let context = WorkerContext {
            rx,
            store: self.store.clone(),
            clock: self.clock.clone(),
            sink,
            manifest,
            state: self.state.clone(),
            pipeline: pipeline.clone(),
            install: install.clone(),
            save_failure: save_failure.clone(),
        };
        let worker = thread::Builder::new()
            .name("recording-worker".into())
            .spawn(move || run_worker(context))
            .map_err(|error| {
                pipeline
                    .lock()
                    .expect("pipeline mutex poisoned")
                    .stop();
                RecordingError::Internal(format!("could not spawn recording worker: {error}"))
            })?;

        Ok((
            Session {
                workflow_id,
                tx,
                worker,
                pipeline,
                save_failure,
            },
            guard,
            install,
        ))
    }

    /// Stops the active recording: stops the pipeline, drains accepted
    /// capture work, saves the manifest, emits the terminal envelope, and
    /// returns the workflow id. Without an active recording this returns
    /// a defined error.
    pub fn stop_recording(&self) -> Result<String, RecordingError> {
        let session = {
            let mut phase = self.lock_state();
            match std::mem::replace(&mut *phase, Phase::Stopping) {
                Phase::Recording(session) => session,
                other => {
                    let error = match other {
                        Phase::Idle => RecordingError::NotRecording,
                        Phase::Starting => RecordingError::StartInProgress,
                        Phase::Stopping | Phase::Failed => RecordingError::StopInProgress,
                        Phase::Recording(_) => unreachable!("matched above"),
                    };
                    *phase = other;
                    return Err(error);
                }
            }
        };
        let Session {
            workflow_id,
            tx,
            worker,
            pipeline,
            save_failure,
        } = session;

        pipeline.lock().expect("pipeline mutex poisoned").stop();
        // Packets already accepted into the queue precede this message,
        // so the worker drains them before finalizing.
        let _ = tx.send(SessionMsg::Stop);
        drop(tx);
        let _ = worker.join();
        *self.lock_state() = Phase::Idle;
        // The worker records a manifest-save failure; surface it instead
        // of reporting a successful stop for a stale manifest.
        let failure = save_failure
            .lock()
            .expect("save failure mutex poisoned")
            .take();
        match failure {
            Some(error) => Err(RecordingError::FinalizationFailed(error)),
            None => Ok(workflow_id),
        }
    }

    pub fn list_workflows(&self) -> Result<Vec<WorkflowSummary>, RecordingError> {
        Ok(self.store.list()?)
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Result<LoadedWorkflow, RecordingError> {
        Ok(self.store.load(workflow_id)?)
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, Phase> {
        self.state.lock().expect("recording state mutex poisoned")
    }
}

fn all_granted(report: &PermissionReport) -> bool {
    [
        report.input_monitoring,
        report.accessibility,
        report.screen_recording,
    ]
    .iter()
    .all(|status| *status == PermissionStatus::Granted)
}

struct WorkerContext {
    rx: Receiver<SessionMsg>,
    store: Arc<dyn WorkflowStore>,
    clock: Arc<dyn Clock>,
    sink: Box<dyn StepSink>,
    manifest: Manifest,
    state: Arc<Mutex<Phase>>,
    pipeline: SharedPipeline,
    install: Arc<InstallSignal>,
    save_failure: Arc<Mutex<Option<String>>>,
}

enum Outcome {
    Stopped,
    Failed(String),
}

/// The single finalization owner for stop and fail-stop.
fn run_worker(context: WorkerContext) {
    let WorkerContext {
        rx,
        store,
        clock,
        sink,
        mut manifest,
        state,
        pipeline,
        install,
        save_failure,
    } = context;
    let workflow_id = manifest.id.clone();

    let mut steps = Vec::new();
    let mut seq: u32 = 0;
    // `None` means every sender vanished without a stop or failure
    // decision (session abandoned mid-start): no terminal to emit.
    let outcome: Option<Outcome> = loop {
        match rx.recv() {
            Ok(SessionMsg::Pipeline(PipelineEvent::Packet(packet))) => {
                seq += 1;
                let CapturePacket {
                    input,
                    pos,
                    display_id,
                    window,
                    element,
                    frontmost_app,
                    frame_age_ms,
                    shots,
                } = *packet;
                let event_id = format!("evt_{seq:04}");
                let (kind, button, key) = match input {
                    PacketInput::Click { button } => (EventKind::Click, Some(button), None),
                    PacketInput::KeyDown { key } => (EventKind::KeyDown, None, Some(key)),
                };
                let event = Event {
                    id: event_id.clone(),
                    ts: event_timestamp(clock.now()),
                    kind,
                    display_id,
                    pos,
                    button,
                    key,
                    window,
                    element,
                    shots: ShotPaths::for_event(&event_id),
                    capture: CaptureMeta { frame_age_ms },
                };
                match store.append_event(&workflow_id, &event, &shots) {
                    Ok(()) => {
                        let step =
                            parse_step(format!("step_{seq:04}"), &event, frontmost_app.as_deref());
                        steps.push(step.clone());
                        // Emitted only after the event line and all three
                        // screenshots are committed. A channel disconnect
                        // never interrupts disk persistence.
                        let _ = sink.emit(LiveEnvelope::Step { step });
                    }
                    Err(error) => {
                        break Some(Outcome::Failed(format!(
                            "event persistence failed: {error}"
                        )));
                    }
                }
            }
            Ok(SessionMsg::Pipeline(PipelineEvent::Failed(error))) => {
                break Some(Outcome::Failed(error));
            }
            Ok(SessionMsg::Stop) => break Some(Outcome::Stopped),
            Err(_) => break None,
        }
    };

    let Some(outcome) = outcome else {
        return;
    };

    // Wait until `start_recording` has installed the session, so an
    // immediate pipeline failure cannot finalize against Phase::Starting
    // and leave a dead session installed afterwards.
    install.wait();

    // Fail-stop initiated here: mirror the stop path by moving the phase
    // to Failed before finalizing. When a concurrent stop already owns
    // the session (Stopping), that stop keeps state ownership.
    let owns_state = {
        let mut phase = state.lock().expect("recording state mutex poisoned");
        if matches!(*phase, Phase::Recording(_)) {
            // Dropping the taken session detaches this worker's own join
            // handle and closes the packet channel.
            *phase = Phase::Failed;
            true
        } else {
            false
        }
    };

    manifest.steps = steps;
    let save_result = store.save_manifest(&workflow_id, &manifest);
    if let Err(error) = &save_result {
        // Recorded for the stop path, which surfaces it to its caller.
        *save_failure.lock().expect("save failure mutex poisoned") =
            Some(format!("manifest save failed: {error}"));
    }
    let terminal = match outcome {
        Outcome::Failed(error) => LiveEnvelope::Failed { workflow_id, error },
        Outcome::Stopped => match save_result {
            Ok(()) => LiveEnvelope::Stopped { workflow_id },
            Err(error) => LiveEnvelope::Failed {
                workflow_id,
                error: format!("manifest save failed: {error}"),
            },
        },
    };
    // Terminal-last: no step emission can follow this point.
    let _ = sink.emit(terminal);

    if owns_state {
        pipeline.lock().expect("pipeline mutex poisoned").stop();
        *state.lock().expect("recording state mutex poisoned") = Phase::Idle;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use chrono::TimeZone;

    use crate::domain::schema::Classification;
    use crate::recording::fake_pipeline::{click_packet, key_packet, FakeCapturePipeline, FakeController};
    use crate::recording::store::JsonWorkflowStore;
    use crate::recording::testutil::{
        fixed_clock, gate_with_status, granted_gate, wait_until, EmitGate, TestSink,
    };
    use crate::permissions::NativeStatus;

    use super::*;

    struct Harness {
        coordinator: Arc<RecordingCoordinator>,
        controllers: Vec<FakeController>,
        root: PathBuf,
        _temp: tempfile::TempDir,
    }

    /// A coordinator over a real JSON store in a temp dir, a granted
    /// permission gate, a fixed clock, and `pipelines` pre-built fake
    /// pipelines handed out in order.
    fn harness(pipelines: usize) -> Harness {
        harness_with(pipelines, granted_gate(), None)
    }

    fn harness_with(
        pipelines: usize,
        gate: Arc<dyn PermissionGate>,
        store_override: Option<Arc<dyn WorkflowStore>>,
    ) -> Harness {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let clock = fixed_clock(
            chrono::Utc
                .with_ymd_and_hms(2026, 8, 16, 22, 31, 5)
                .unwrap(),
        );
        let store: Arc<dyn WorkflowStore> = store_override.unwrap_or_else(|| {
            Arc::new(JsonWorkflowStore::new(root.clone(), clock.clone()))
        });

        let mut queue = Vec::new();
        let mut controllers = Vec::new();
        for _ in 0..pipelines {
            let (pipeline, controller) = FakeCapturePipeline::new();
            queue.push(pipeline);
            controllers.push(controller);
        }
        let queue = Mutex::new(queue.into_iter());
        let factory: PipelineFactory = Box::new(move || {
            Box::new(
                queue
                    .lock()
                    .unwrap()
                    .next()
                    .expect("test prepared too few fake pipelines"),
            )
        });

        let coordinator = Arc::new(RecordingCoordinator::new(store, gate, factory, clock));
        Harness {
            coordinator,
            controllers,
            root,
            _temp: temp,
        }
    }

    fn workflow_dir(harness: &Harness, id: &str) -> PathBuf {
        harness.root.join(id)
    }

    /// Polls stop_recording until the coordinator is observably idle
    /// again (a worker-owned fail-stop finished).
    fn wait_for_idle(coordinator: &RecordingCoordinator) {
        assert!(
            wait_until(Duration::from_secs(5), || matches!(
                coordinator.stop_recording(),
                Err(RecordingError::NotRecording)
            )),
            "coordinator did not return to Idle",
        );
    }

    #[test]
    fn full_lifecycle_persists_streams_in_order_and_commits_before_emitting() {
        let harness = harness(1);
        let root = harness.root.clone();
        let published_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let violations: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        // The observer runs inside emit: at that moment the event line
        // and all three screenshots must already be on disk.
        let (sink, log) = TestSink::recording();
        let sink = sink.with_step_observer({
            let published_id = published_id.clone();
            let violations = violations.clone();
            let root = root.clone();
            move |step| {
                let id = published_id.lock().unwrap().clone().expect("id published");
                let dir = root.join(&id);
                let event_id = &step.event_ids[0];
                let log_text = fs::read_to_string(dir.join("events.jsonl")).unwrap_or_default();
                if !log_text.contains(event_id) {
                    violations
                        .lock()
                        .unwrap()
                        .push(format!("{event_id}: emitted before its JSONL line"));
                }
                for suffix in ["full", "window", "element"] {
                    if !dir.join(format!("shots/{event_id}.{suffix}.png")).is_file() {
                        violations
                            .lock()
                            .unwrap()
                            .push(format!("{event_id}: emitted before {suffix} shot"));
                    }
                }
            }
        });

        let id = harness
            .coordinator
            .start_recording(Some("Approve invoice"), Box::new(sink))
            .unwrap();
        *published_id.lock().unwrap() = Some(id.clone());

        harness.controllers[0].emit(click_packet());
        harness.controllers[0].emit(key_packet());
        let stopped_id = harness.coordinator.stop_recording().unwrap();
        assert_eq!(stopped_id, id);

        // Channel: two ordered steps, then the terminal, terminal-last.
        let items = log.items();
        assert_eq!(items.len(), 3);
        let LiveEnvelope::Step { step: step1 } = &items[0] else {
            panic!("first item must be a step, got {:?}", items[0]);
        };
        let LiveEnvelope::Step { step: step2 } = &items[1] else {
            panic!("second item must be a step, got {:?}", items[1]);
        };
        assert_eq!(items[2], LiveEnvelope::Stopped { workflow_id: id.clone() });
        assert_eq!(step1.title, "Click \"OK\" — TextEdit");
        assert_eq!(step1.classification, Classification::Click);
        assert_eq!(step1.event_ids, vec!["evt_0001".to_owned()]);
        assert_eq!(step2.title, "Press Cmd+S — TextEdit");
        assert_eq!(step2.classification, Classification::Type);
        assert_eq!(step2.event_ids, vec!["evt_0002".to_owned()]);
        assert_eq!(violations.lock().unwrap().as_slice(), &[] as &[String]);

        // Disk: two JSONL lines, six PNGs, a manifest with two steps in
        // the same order, all through the store seam.
        let loaded = harness.coordinator.get_workflow(&id).unwrap();
        assert_eq!(loaded.manifest.name, "Approve invoice");
        assert_eq!(loaded.manifest.schema_version, 1);
        let event_ids: Vec<&str> = loaded.events.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(event_ids, vec!["evt_0001", "evt_0002"]);
        assert_eq!(loaded.events[0].ts, "2026-08-16T22:31:05.000Z");
        let manifest_steps: Vec<(&str, &str)> = loaded
            .manifest
            .steps
            .iter()
            .map(|s| (s.id.as_str(), s.title.as_str()))
            .collect();
        assert_eq!(
            manifest_steps,
            vec![
                ("step_0001", "Click \"OK\" — TextEdit"),
                ("step_0002", "Press Cmd+S — TextEdit"),
            ],
        );
        let dir = workflow_dir(&harness, &id);
        let jsonl = fs::read_to_string(dir.join("events.jsonl")).unwrap();
        assert_eq!(jsonl.lines().count(), 2);
        let png_count = fs::read_dir(dir.join("shots"))
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".png")
            })
            .count();
        assert_eq!(png_count, 6);

        // The controller's pipeline was stopped exactly once by the stop.
        assert_eq!(harness.controllers[0].stop_count(), 1);
    }

    #[test]
    fn channel_disconnect_never_interrupts_disk_persistence() {
        let harness = harness(1);
        let (sink, log) = TestSink::failing();
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.controllers[0].emit(click_packet());
        harness.controllers[0].emit(key_packet());
        harness.coordinator.stop_recording().unwrap();

        // Every emit failed, yet both events and the manifest are on disk.
        assert_eq!(log.items().len(), 3);
        let loaded = harness.coordinator.get_workflow(&id).unwrap();
        assert_eq!(loaded.events.len(), 2);
        assert_eq!(loaded.manifest.steps.len(), 2);
    }

    #[test]
    fn start_refuses_unless_all_three_permissions_pass() {
        let denied_combos = [
            (NativeStatus::Denied, NativeStatus::Granted, NativeStatus::Granted),
            (NativeStatus::Granted, NativeStatus::Denied, NativeStatus::Granted),
            (NativeStatus::Granted, NativeStatus::Granted, NativeStatus::Denied),
            // First launch: Input Monitoring not requested yet, so
            // Accessibility reports blocked_by_prerequisite.
            (NativeStatus::NotDetermined, NativeStatus::Granted, NativeStatus::Granted),
        ];
        for (input_monitoring, accessibility, screen_recording) in denied_combos {
            let harness = harness_with(
                0,
                gate_with_status(input_monitoring, accessibility, screen_recording),
                None,
            );
            let (sink, log) = TestSink::recording();
            let error = harness
                .coordinator
                .start_recording(Some("w"), Box::new(sink))
                .unwrap_err();
            assert!(
                matches!(error, RecordingError::PermissionsMissing(_)),
                "got {error}",
            );
            // Refusal happens before any folder is created.
            assert_eq!(fs::read_dir(&harness.root).unwrap().count(), 0);
            assert_eq!(log.items().len(), 0);
        }

        let harness = harness_with(
            1,
            gate_with_status(
                NativeStatus::Granted,
                NativeStatus::Granted,
                NativeStatus::Granted,
            ),
            None,
        );
        let (sink, _log) = TestSink::recording();
        harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.coordinator.stop_recording().unwrap();
    }

    #[test]
    fn concurrent_starts_yield_exactly_one_success() {
        let harness = harness(1);
        harness.controllers[0].hold_start();

        let coordinator = harness.coordinator.clone();
        let (sink1, log1) = TestSink::recording();
        let first = thread::spawn(move || {
            coordinator.start_recording(Some("first"), Box::new(sink1))
        });
        harness.controllers[0].wait_for_start_entered();

        // The second start arrives while the first is still starting.
        let (sink2, _log2) = TestSink::recording();
        let second = harness
            .coordinator
            .start_recording(Some("second"), Box::new(sink2));
        assert!(
            matches!(second, Err(RecordingError::AlreadyActive)),
            "got {second:?}",
        );

        harness.controllers[0].release_start();
        let first = first.join().unwrap();
        assert!(first.is_ok(), "got {first:?}");
        harness.coordinator.stop_recording().unwrap();
        let terminals = log1.items().iter().filter(|i| i.is_terminal()).count();
        assert_eq!(terminals, 1);
    }

    #[test]
    fn second_start_while_recording_is_refused() {
        let harness = harness(1);
        let (sink, _log) = TestSink::recording();
        harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        let (sink2, _log2) = TestSink::recording();
        let second = harness
            .coordinator
            .start_recording(Some("again"), Box::new(sink2));
        assert!(matches!(second, Err(RecordingError::AlreadyActive)));
        harness.coordinator.stop_recording().unwrap();
    }

    #[test]
    fn stop_without_an_active_recording_returns_a_defined_error() {
        let harness = harness(0);
        let error = harness.coordinator.stop_recording().unwrap_err();
        assert!(matches!(error, RecordingError::NotRecording), "got {error}");
    }

    #[test]
    fn stop_during_start_is_a_defined_error_and_the_session_has_one_terminal() {
        let harness = harness(1);
        harness.controllers[0].hold_start();
        let coordinator = harness.coordinator.clone();
        let (sink, log) = TestSink::recording();
        let starter =
            thread::spawn(move || coordinator.start_recording(None, Box::new(sink)));
        harness.controllers[0].wait_for_start_entered();

        let stop = harness.coordinator.stop_recording();
        assert!(
            matches!(stop, Err(RecordingError::StartInProgress)),
            "got {stop:?}",
        );

        harness.controllers[0].release_start();
        let id = starter.join().unwrap().unwrap();
        let stopped = harness.coordinator.stop_recording().unwrap();
        assert_eq!(stopped, id);
        let items = log.items();
        assert_eq!(items, vec![LiveEnvelope::Stopped { workflow_id: id }]);
    }

    #[test]
    fn pipeline_startup_failure_rolls_back_and_returns_to_idle() {
        let harness = harness(2);
        harness.controllers[0].fail_start("event tap refused");

        let (sink, log) = TestSink::recording();
        let error = harness
            .coordinator
            .start_recording(Some("doomed"), Box::new(sink))
            .unwrap_err();
        assert!(
            matches!(&error, RecordingError::PipelineStart(e) if e.contains("event tap refused")),
            "got {error}",
        );
        // Startup failed before the id published: the folder is gone and
        // no envelope was emitted.
        assert_eq!(fs::read_dir(&harness.root).unwrap().count(), 0);
        assert_eq!(log.items().len(), 0);

        // Back to Idle: the next start succeeds.
        let (sink2, _log2) = TestSink::recording();
        harness
            .coordinator
            .start_recording(Some("recovers"), Box::new(sink2))
            .unwrap();
        harness.coordinator.stop_recording().unwrap();
    }

    /// The fail-during-start race: the pipeline reports Failed through
    /// the emitter before `start` even returns. The worker must wait for
    /// session installation, own the fail-stop, and converge to Idle
    /// with exactly one terminal — no dead session, no user stop needed.
    #[test]
    fn pipeline_failure_during_startup_converges_to_idle_with_one_terminal() {
        let harness = harness(2);
        harness.controllers[0].fail_after_start("tap died during start");

        let (sink, log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("doomed"), Box::new(sink))
            .unwrap();

        // Convergence: a fresh start succeeds without any explicit stop
        // of the failed session.
        assert!(
            wait_until(Duration::from_secs(5), || {
                let (retry_sink, _retry_log) = TestSink::recording();
                harness
                    .coordinator
                    .start_recording(Some("recovers"), Box::new(retry_sink))
                    .is_ok()
            }),
            "coordinator did not return to Idle after the startup failure",
        );
        harness.coordinator.stop_recording().unwrap();

        // Exactly one terminal on the failed session's channel, and the
        // worker-owned fail-stop stopped its pipeline.
        let items = log.items();
        assert_eq!(
            items,
            vec![LiveEnvelope::Failed {
                workflow_id: id,
                error: "tap died during start".into(),
            }],
        );
        assert_eq!(harness.controllers[0].stop_count(), 1);
    }

    #[test]
    fn pipeline_failure_fail_stops_with_one_terminal_and_preserves_committed_data() {
        let harness = harness(1);
        let (sink, log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.controllers[0].emit(click_packet());
        harness.controllers[0].fail("event tap disabled");
        wait_for_idle(&harness.coordinator);

        let items = log.items();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], LiveEnvelope::Step { .. }));
        assert_eq!(
            items[1],
            LiveEnvelope::Failed {
                workflow_id: id.clone(),
                error: "event tap disabled".into(),
            },
        );
        // Committed data is preserved and the manifest holds the
        // committed step.
        let loaded = harness.coordinator.get_workflow(&id).unwrap();
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.manifest.steps.len(), 1);
        // The worker's fail-stop also stopped the pipeline.
        assert_eq!(harness.controllers[0].stop_count(), 1);
    }

    #[test]
    fn simultaneous_failures_yield_exactly_one_fail_stop_transition() {
        let harness = harness(1);
        let (sink, log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.controllers[0].fail("stream lost");
        harness.controllers[0].fail("tap disabled");
        wait_for_idle(&harness.coordinator);

        let items = log.items();
        assert_eq!(
            items,
            vec![LiveEnvelope::Failed {
                workflow_id: id,
                error: "stream lost".into(),
            }],
        );
    }

    #[test]
    fn concurrent_stop_versus_fail_yields_exactly_one_terminal() {
        let harness = harness(1);
        let gate = Arc::new(EmitGate::default());
        let (sink, log) = TestSink::recording();
        let sink = sink.with_gate(gate.clone());
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();

        // The worker blocks inside the first step emission; a pipeline
        // failure and a stop request are then both queued behind it.
        harness.controllers[0].emit(click_packet());
        gate.wait_for_waiter();
        harness.controllers[0].fail("tap disabled");

        let coordinator = harness.coordinator.clone();
        let stopper = thread::spawn(move || coordinator.stop_recording());
        // The stop owns the session once it has stopped the pipeline.
        assert!(wait_until(Duration::from_secs(5), || {
            harness.controllers[0].stop_count() >= 1
        }));
        gate.open();

        let stop_result = stopper.join().unwrap();
        assert_eq!(stop_result.unwrap(), id);
        let items = log.items();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], LiveEnvelope::Step { .. }));
        assert_eq!(
            items[1],
            LiveEnvelope::Failed {
                workflow_id: id,
                error: "tap disabled".into(),
            },
        );
    }

    /// Store wrapper that fails the nth `append_event`.
    struct FailingAppendStore {
        inner: JsonWorkflowStore,
        fail_on: usize,
        seen: AtomicUsize,
    }

    impl WorkflowStore for FailingAppendStore {
        fn create(&self, name: &str) -> Result<crate::recording::store::CreatedWorkflow, StoreError> {
            self.inner.create(name)
        }

        fn append_event(
            &self,
            workflow_id: &str,
            event: &Event,
            shots: &crate::recording::store::ShotPayloads,
        ) -> Result<(), StoreError> {
            if self.seen.fetch_add(1, Ordering::SeqCst) + 1 == self.fail_on {
                return Err(StoreError::Io {
                    context: "injected append failure".to_owned(),
                    source: std::io::Error::from(std::io::ErrorKind::Other),
                });
            }
            self.inner.append_event(workflow_id, event, shots)
        }

        fn load(&self, workflow_id: &str) -> Result<LoadedWorkflow, StoreError> {
            self.inner.load(workflow_id)
        }

        fn save_manifest(&self, workflow_id: &str, manifest: &Manifest) -> Result<(), StoreError> {
            self.inner.save_manifest(workflow_id, manifest)
        }

        fn list(&self) -> Result<Vec<WorkflowSummary>, StoreError> {
            self.inner.list()
        }
    }

    /// Store wrapper whose `save_manifest` always fails.
    struct FailingSaveStore {
        inner: JsonWorkflowStore,
    }

    impl WorkflowStore for FailingSaveStore {
        fn create(&self, name: &str) -> Result<crate::recording::store::CreatedWorkflow, StoreError> {
            self.inner.create(name)
        }

        fn append_event(
            &self,
            workflow_id: &str,
            event: &Event,
            shots: &crate::recording::store::ShotPayloads,
        ) -> Result<(), StoreError> {
            self.inner.append_event(workflow_id, event, shots)
        }

        fn load(&self, workflow_id: &str) -> Result<LoadedWorkflow, StoreError> {
            self.inner.load(workflow_id)
        }

        fn save_manifest(&self, _workflow_id: &str, _manifest: &Manifest) -> Result<(), StoreError> {
            Err(StoreError::Io {
                context: "injected save failure".to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::Other),
            })
        }

        fn list(&self) -> Result<Vec<WorkflowSummary>, StoreError> {
            self.inner.list()
        }
    }

    #[test]
    fn stop_surfaces_a_manifest_save_failure_instead_of_ok() {
        let temp = tempfile::tempdir().unwrap();
        let clock = fixed_clock(
            chrono::Utc
                .with_ymd_and_hms(2026, 8, 16, 22, 31, 5)
                .unwrap(),
        );
        let store: Arc<dyn WorkflowStore> = Arc::new(FailingSaveStore {
            inner: JsonWorkflowStore::new(temp.path().to_path_buf(), clock),
        });
        let harness = harness_with(1, granted_gate(), Some(store));

        let (sink, log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.controllers[0].emit(click_packet());

        let error = harness.coordinator.stop_recording().unwrap_err();
        assert!(
            matches!(
                &error,
                RecordingError::FinalizationFailed(e) if e.contains("injected save failure")
            ),
            "got {error}",
        );
        // The channel still carries the committed step and one Failed
        // terminal, and the coordinator is back to Idle.
        let items = log.items();
        assert_eq!(items.len(), 2, "got {items:?}");
        assert!(matches!(items[0], LiveEnvelope::Step { .. }));
        assert!(
            matches!(
                &items[1],
                LiveEnvelope::Failed { workflow_id, error }
                    if *workflow_id == id && error.contains("injected save failure")
            ),
            "got {:?}",
            items[1],
        );
        let after = harness.coordinator.stop_recording().unwrap_err();
        assert!(matches!(after, RecordingError::NotRecording), "got {after}");
    }

    #[test]
    fn append_failure_emits_no_channel_step_and_fail_stops() {
        let temp = tempfile::tempdir().unwrap();
        let clock = fixed_clock(
            chrono::Utc
                .with_ymd_and_hms(2026, 8, 16, 22, 31, 5)
                .unwrap(),
        );
        let store: Arc<dyn WorkflowStore> = Arc::new(FailingAppendStore {
            inner: JsonWorkflowStore::new(temp.path().to_path_buf(), clock),
            fail_on: 2,
            seen: AtomicUsize::new(0),
        });
        let harness = harness_with(1, granted_gate(), Some(store));

        let (sink, log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("w"), Box::new(sink))
            .unwrap();
        harness.controllers[0].emit(click_packet());
        harness.controllers[0].emit(key_packet());
        wait_for_idle(&harness.coordinator);

        let items = log.items();
        assert_eq!(items.len(), 2, "no step for the failed append: {items:?}");
        assert!(matches!(items[0], LiveEnvelope::Step { .. }));
        assert!(
            matches!(
                &items[1],
                LiveEnvelope::Failed { workflow_id, error }
                    if *workflow_id == id && error.contains("injected append failure")
            ),
            "got {:?}",
            items[1],
        );

        // The committed first event survives; the failed one left no line.
        let loaded = harness.coordinator.get_workflow(&id).unwrap();
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.manifest.steps.len(), 1);
    }

    #[test]
    fn blank_or_missing_name_defaults_to_the_timestamp() {
        let harness = harness(3);
        for (index, name) in [None, Some(""), Some("   ")].into_iter().enumerate() {
            let (sink, _log) = TestSink::recording();
            let id = harness
                .coordinator
                .start_recording(name, Box::new(sink))
                .unwrap();
            assert!(
                id.starts_with("2026-08-16-223105-"),
                "folder name {id} should carry the timestamp prefix",
            );
            let loaded = harness.coordinator.get_workflow(&id).unwrap();
            assert_eq!(
                loaded.manifest.name, "2026-08-16 22:31:05",
                "case {index}: blank or missing name takes the timestamp default",
            );
            harness.coordinator.stop_recording().unwrap();
        }
    }

    #[test]
    fn an_explicit_name_is_kept() {
        let harness = harness(1);
        let (sink, _log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some(" Approve invoice "), Box::new(sink))
            .unwrap();
        let loaded = harness.coordinator.get_workflow(&id).unwrap();
        assert_eq!(loaded.manifest.name, "Approve invoice");
        harness.coordinator.stop_recording().unwrap();
    }

    #[test]
    fn list_and_get_delegate_to_the_store() {
        let harness = harness(1);
        assert_eq!(harness.coordinator.list_workflows().unwrap(), vec![]);
        let (sink, _log) = TestSink::recording();
        let id = harness
            .coordinator
            .start_recording(Some("listed"), Box::new(sink))
            .unwrap();
        harness.coordinator.stop_recording().unwrap();

        let list = harness.coordinator.list_workflows().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].name, "listed");
        let error = harness.coordinator.get_workflow("missing-id").unwrap_err();
        assert!(
            matches!(error, RecordingError::Store(StoreError::NotFound(_))),
            "got {error}",
        );
    }
}
