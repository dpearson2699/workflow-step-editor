//! The real macOS [`CapturePipeline`]: wires the ListenOnly event tap,
//! the per-display stream manager, the frame broker, the bounded queue,
//! the ordered capture worker, and the health/failure adapter into the
//! one trait boundary the PR-02 coordinator already consumes.
//!
//! Start order enforces the pre-buffered-capture contract (ADR 0001):
//! the streams warm up (every display retains a first frame) before the
//! event tap enables, so no event can arrive without a pre-event frame.
//! Stop order enforces the `CapturePipeline::stop` quiescence contract:
//! the tap stops (no new jobs), the queue drains through the worker
//! while the streams keep publishing (so an accepted key-down can finish
//! its bounded post-event wait, DEC-002), then the streams stop, and
//! only then does the emitter guard close — so `stop` never returns
//! while a packet emission is still in flight.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::capture::broker::{FrameBroker, POST_EVENT_FRAME_WINDOW_NS, POST_EVENT_SETTLE_NS};
use crate::capture::health::EmitterGuard;
use crate::capture::hostclock::host_now_ns;
use crate::capture::macos::{DisplayReconfigurationObserver, MacosResolver, MacosStreamBackend};
use crate::capture::queue::{
    capture_queue, CaptureJob, EnqueueError, JobSender, QueueDepth, RawInput, queue_capacity,
    RETAINED_FRAME_BYTE_BUDGET,
};
use crate::capture::streams::{FailureSink, StreamManager};
use crate::capture::tap::{start_event_tap, TapEvent, TapHandle, TapInput, TapHealthProbe};
use crate::capture::worker::{run_capture_worker, WaitRuntime};
use crate::recording::pipeline::{CapturePipeline, PacketEmitter};

/// How long to wait for every display to deliver its first frame.
const WARM_UP_TIMEOUT: Duration = Duration::from_secs(8);
/// The tap-health poll interval.
const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(250);
/// How often the worker re-queries the broker during a key-down's
/// bounded post-event settle/wait (DEC-002/DEC-004).
const POST_EVENT_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// The production wait runtime for the worker's key-down post-event
/// wait: the mach host clock (the event and frame timestamp domain),
/// a plain thread sleep, the DEC-002 window, the DEC-004 settle, and a
/// short poll.
struct HostWaitRuntime;

impl WaitRuntime for HostWaitRuntime {
    fn now_ns(&mut self) -> u64 {
        host_now_ns()
    }

    fn wait_for(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn window_ns(&self) -> u64 {
        POST_EVENT_FRAME_WINDOW_NS
    }

    fn settle_ns(&self) -> u64 {
        POST_EVENT_SETTLE_NS
    }

    fn poll_interval(&self) -> Duration {
        POST_EVENT_POLL_INTERVAL
    }
}

/// The production macOS capture pipeline. One instance serves one
/// recording session.
#[derive(Default)]
pub struct MacosCapturePipeline {
    running: Option<RunningPipeline>,
}

impl MacosCapturePipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// The maximum queue depth observed during the session, for the
    /// capture receipt.
    pub fn max_queue_depth(&self) -> usize {
        self.running
            .as_ref()
            .map_or(0, |running| running.depth.max_observed())
    }
}

struct RunningPipeline {
    tap: Option<TapHandle>,
    stream_manager: Option<StreamManager>,
    display_observer: Option<DisplayReconfigurationObserver>,
    worker: Option<JoinHandle<()>>,
    health: HealthMonitor,
    guard: Arc<EmitterGuard>,
    depth: Arc<QueueDepth>,
}

impl CapturePipeline for MacosCapturePipeline {
    fn start(&mut self, emitter: PacketEmitter) -> Result<(), String> {
        if self.running.is_some() {
            return Err("capture pipeline already started".to_owned());
        }
        let guard = Arc::new(EmitterGuard::new(emitter));
        let broker = Arc::new(Mutex::new(FrameBroker::new()));

        // Any asynchronous failure (stream stop, display change,
        // reconfiguration failure) routes into the single fail-stop.
        let failure: FailureSink = {
            let guard = guard.clone();
            Arc::new(move |error: String| guard.fail(error))
        };

        // 1. Streams first: publish the display set and start one
        //    continuous stream per display.
        let stream_manager =
            StreamManager::start(MacosStreamBackend, broker.clone(), failure.clone())
                .map_err(|error| format!("could not start display capture: {error}"))?;

        // 2. Warm-up: every display must retain a first frame before the
        //    tap enables, so no event can precede a pre-event frame.
        if let Err(error) = wait_for_warm_up(&broker) {
            stream_manager.stop();
            return Err(error);
        }

        // 3. Bounded queue sized from the retained-frame byte budget.
        let capacity = queue_capacity(RETAINED_FRAME_BYTE_BUDGET, current_frame_set_bytes(&broker));
        let (job_tx, job_rx) = capture_queue(capacity);
        let depth = job_tx.depth();

        // 4. The single ordered capture worker. It shares the live
        //    broker for the key-down post-event query (DEC-002).
        let worker = {
            let guard = guard.clone();
            let broker = broker.clone();
            match std::thread::Builder::new()
                .name("capture-worker".into())
                .spawn(move || {
                    run_capture_worker(
                        job_rx,
                        Box::new(MacosResolver::new()),
                        broker,
                        HostWaitRuntime,
                        guard,
                    );
                }) {
                Ok(worker) => worker,
                Err(error) => {
                    stream_manager.stop();
                    return Err(format!("could not spawn the capture worker: {error}"));
                }
            }
        };

        // 5. Display reconfiguration observer signals the stream manager.
        let display_observer = match DisplayReconfigurationObserver::start(
            stream_manager.reconfigure_signal(),
        ) {
            Ok(observer) => observer,
            Err(error) => {
                drop(job_tx);
                let _ = worker.join();
                stream_manager.stop();
                return Err(format!("could not observe display configuration: {error}"));
            }
        };

        // 6. The event tap enables last. Its callback pins an immutable
        //    frame snapshot and enqueues without blocking.
        let tap = match start_tap(broker.clone(), job_tx, guard.clone()) {
            Ok(tap) => tap,
            Err(error) => {
                // job_tx already moved into start_tap's closure; on
                // failure it was dropped there, so the worker drains and
                // exits.
                let _ = worker.join();
                drop(display_observer);
                stream_manager.stop();
                return Err(error);
            }
        };

        // 7. The health monitor polls the runtime tap-enabled flag.
        let health = HealthMonitor::start(tap.health_probe(), guard.clone());

        self.running = Some(RunningPipeline {
            tap: Some(tap),
            stream_manager: Some(stream_manager),
            display_observer: Some(display_observer),
            worker: Some(worker),
            health,
            guard,
            depth,
        });
        Ok(())
    }

    fn stop(&mut self) {
        let Some(mut running) = self.running.take() else {
            return;
        };
        // Stop the health monitor first: its probe reads the tap's mach
        // port, which `tap.stop()` invalidates and releases, so the
        // probe must quiesce before the tap thread joins (see the
        // `TapHealthProbe` safety contract).
        running.health.stop();
        // Stop the input source: no new jobs are enqueued. Stopping the
        // tap drops the only job sender.
        if let Some(tap) = running.tap.take() {
            tap.stop();
        }
        if let Some(observer) = running.display_observer.take() {
            drop(observer);
        }
        // Drain the worker while the streams still publish: an accepted
        // key-down can finish its bounded post-event wait (DEC-002).
        // Join it so every accepted packet is emitted before the guard
        // closes.
        if let Some(worker) = running.worker.take() {
            let _ = worker.join();
        }
        // Then stop the frame sources.
        if let Some(manager) = running.stream_manager.take() {
            manager.stop();
        }
        // Quiesce: block until no emitter call is in flight, then drop
        // the emitter. `stop` cannot return while a packet emission is
        // still running.
        running.guard.close();
    }
}

impl Drop for MacosCapturePipeline {
    fn drop(&mut self) {
        if self.running.is_some() {
            self.stop();
        }
    }
}

/// Waits until every current display retains a first frame.
fn wait_for_warm_up(broker: &Arc<Mutex<FrameBroker>>) -> Result<(), String> {
    let deadline = Instant::now() + WARM_UP_TIMEOUT;
    loop {
        if broker.lock().expect("frame broker lock poisoned").is_warm() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(
                "screen capture did not deliver a first frame for every display in time".to_owned(),
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The total bytes of one retained frame set for the current display
/// configuration.
fn current_frame_set_bytes(broker: &Arc<Mutex<FrameBroker>>) -> u64 {
    let displays = broker.lock().expect("frame broker lock poisoned").displays();
    displays
        .iter()
        .map(|display| u64::from(display.width_px()) * u64::from(display.height_px()) * 4)
        .sum::<u64>()
        .max(1)
}

/// Starts the event tap with the snapshot-and-enqueue callback.
fn start_tap(
    broker: Arc<Mutex<FrameBroker>>,
    job_tx: JobSender,
    guard: Arc<EmitterGuard>,
) -> Result<TapHandle, String> {
    let on_event = {
        let guard = guard.clone();
        move |event: TapEvent| {
            // Pin the immutable eligible-frame snapshot before the
            // nonblocking enqueue, so a delayed worker keeps its
            // predecessor frame even as the live broker advances.
            let snapshot = broker
                .lock()
                .expect("frame broker lock poisoned")
                .snapshot(event.ts_ns);
            let job = CaptureJob {
                input: match event.input {
                    TapInput::Click { button } => RawInput::Click { button },
                    TapInput::KeyDown { key } => RawInput::KeyDown { key },
                },
                x: event.x,
                y: event.y,
                ts_ns: event.ts_ns,
                snapshot,
            };
            match job_tx.enqueue(job) {
                Ok(()) => {}
                Err(EnqueueError::Saturated) => guard.fail(
                    "capture overloaded: the screenshot queue is full (recording stopped to \
                     preserve the per-event screenshot guarantee)",
                ),
                // The worker is already gone (stop in progress); drop.
                Err(EnqueueError::Closed) => {}
            }
        }
    };
    let on_disabled = {
        let guard = guard.clone();
        move |error: String| guard.fail(error)
    };
    start_event_tap(on_event, on_disabled)
}

/// The tap-health monitor thread: polls `CGEventTapIsEnabled` and
/// fail-stops on a disabled tap (no silent re-enable, DEC-007).
struct HealthMonitor {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl HealthMonitor {
    fn start(probe: TapHealthProbe, guard: Arc<EmitterGuard>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let stop = stop.clone();
            std::thread::Builder::new()
                .name("capture-health".into())
                .spawn(move || {
                    while !stop.load(Ordering::SeqCst) {
                        if guard.has_failed() {
                            return;
                        }
                        if !probe.is_enabled() {
                            guard.fail(
                                "event tap disabled mid-recording (Input Monitoring health check)",
                            );
                            return;
                        }
                        std::thread::sleep(HEALTH_POLL_INTERVAL);
                    }
                })
                .ok()
        };
        Self { stop, thread }
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
