//! The display stream manager: one continuous stream per active
//! display, restarted as a set when the display configuration changes.
//!
//! The manager is generic over [`StreamBackend`] — the production
//! backend drives ScreenCaptureKit ([`crate::capture::macos`]); tests
//! substitute a deterministic fake because live streams are
//! nondeterministic external systems. Behavior on this side of the
//! seam is production behavior: generation publication, frame
//! carry-over, restart, and failure propagation.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::capture::broker::{FrameBroker, FrameData};
use crate::capture::geometry::DisplayGeometry;

/// Where a backend stream publishes its frames.
pub type FrameSink = Arc<dyn Fn(FrameData) + Send + Sync>;
/// Where the manager and backends report asynchronous failures.
pub type FailureSink = Arc<dyn Fn(String) + Send + Sync>;

/// One running per-display stream.
pub trait ActiveStream: Send {
    /// Stops the stream. Frames already published stay retained.
    fn stop(self: Box<Self>);
}

/// The nondeterministic platform seam the manager drives.
pub trait StreamBackend: Send + 'static {
    /// The current display set; the main display is listed first.
    fn current_displays(&mut self) -> Result<Vec<DisplayGeometry>, String>;

    /// Starts one continuous stream for `display`, publishing every
    /// frame through `sink` and reporting stream failure through
    /// `failure`.
    fn start_stream(
        &mut self,
        display: &DisplayGeometry,
        sink: FrameSink,
        failure: FailureSink,
    ) -> Result<Box<dyn ActiveStream>, String>;
}

enum Command {
    /// The display configuration changed; restart the stream set.
    Reconfigured,
    Stop,
}

/// Handle over the manager's control thread.
pub struct StreamManager {
    tx: Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl StreamManager {
    /// Fetches the initial display set, publishes it, and starts one
    /// stream per display. Returns after the initial set is running;
    /// first-frame warm-up is observed by the caller through the
    /// broker.
    pub fn start(
        mut backend: impl StreamBackend,
        broker: Arc<Mutex<FrameBroker>>,
        failure: FailureSink,
    ) -> Result<Self, String> {
        let displays = backend.current_displays()?;
        if displays.is_empty() {
            return Err("no active displays to capture".to_owned());
        }
        broker
            .lock()
            .expect("frame broker lock poisoned")
            .publish_displays(displays.clone());
        let mut streams = start_streams(&mut backend, &displays, &broker, &failure)?;

        let (tx, rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("capture-stream-manager".into())
            .spawn(move || {
                run_manager(&mut backend, &broker, &failure, &rx, &mut streams);
                for stream in streams.drain(..) {
                    stream.stop();
                }
            })
            .map_err(|error| format!("could not spawn stream manager: {error}"))?;
        Ok(Self {
            tx,
            thread: Some(thread),
        })
    }

    /// Signals a display-configuration change (callable from any
    /// thread, including a C callback trampoline).
    pub fn notify_reconfigured(&self) {
        let _ = self.tx.send(Command::Reconfigured);
    }

    /// A cloneable reconfiguration signal for callback registration.
    pub fn reconfigure_signal(&self) -> impl Fn() + Send + Sync + 'static {
        let tx = self.tx.clone();
        move || {
            let _ = tx.send(Command::Reconfigured);
        }
    }

    /// Stops every stream and joins the control thread.
    pub fn stop(mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for StreamManager {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_manager(
    backend: &mut impl StreamBackend,
    broker: &Arc<Mutex<FrameBroker>>,
    failure: &FailureSink,
    rx: &Receiver<Command>,
    streams: &mut Vec<Box<dyn ActiveStream>>,
) {
    while let Ok(command) = rx.recv() {
        match command {
            Command::Stop => return,
            Command::Reconfigured => {
                // Coalesce bursts of reconfiguration signals (the OS
                // fires several per change).
                while let Ok(next) = rx.try_recv() {
                    if matches!(next, Command::Stop) {
                        return;
                    }
                }
                // Stop the outgoing generation first; its newest frames
                // stay retained in the broker and keep serving events
                // during the new generation's warm-up.
                for stream in streams.drain(..) {
                    stream.stop();
                }
                let displays = match backend.current_displays() {
                    Ok(displays) if !displays.is_empty() => displays,
                    Ok(_) => {
                        failure("display configuration changed to no active displays".to_owned());
                        return;
                    }
                    Err(error) => {
                        failure(format!(
                            "display reconfiguration failed to enumerate displays: {error}"
                        ));
                        return;
                    }
                };
                broker
                    .lock()
                    .expect("frame broker lock poisoned")
                    .publish_displays(displays.clone());
                match start_streams(backend, &displays, broker, failure) {
                    Ok(new_streams) => *streams = new_streams,
                    Err(error) => {
                        failure(format!(
                            "stream restart after display change failed: {error}"
                        ));
                        return;
                    }
                }
            }
        }
    }
}

fn start_streams(
    backend: &mut impl StreamBackend,
    displays: &[DisplayGeometry],
    broker: &Arc<Mutex<FrameBroker>>,
    failure: &FailureSink,
) -> Result<Vec<Box<dyn ActiveStream>>, String> {
    let mut streams = Vec::with_capacity(displays.len());
    for display in displays {
        let sink: FrameSink = {
            let broker = broker.clone();
            Arc::new(move |frame: FrameData| {
                broker
                    .lock()
                    .expect("frame broker lock poisoned")
                    .publish_frame(Arc::new(frame));
            })
        };
        let stream = backend.start_stream(display, sink, failure.clone())?;
        streams.push(stream);
    }
    Ok(streams)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use crate::capture::geometry::RectPt;
    use crate::recording::testutil::wait_until;

    use super::*;

    fn display(id: u32) -> DisplayGeometry {
        DisplayGeometry {
            id,
            frame_pt: RectPt::new(f64::from(id) * 1000.0, 0.0, 100.0, 100.0),
            scale: 1.0,
        }
    }

    fn frame(display: &DisplayGeometry, ts_ns: u64) -> FrameData {
        FrameData {
            display: display.clone(),
            width_px: 100,
            height_px: 100,
            bytes_per_row: 400,
            ts_ns,
            pixels: vec![0; 40_000],
        }
    }

    #[derive(Default)]
    struct FakeState {
        display_sets: Mutex<Vec<Vec<DisplayGeometry>>>,
        started: Mutex<Vec<(u32, FrameSink)>>,
        stopped: AtomicU32,
    }

    struct FakeBackend {
        state: Arc<FakeState>,
    }

    struct FakeStream {
        state: Arc<FakeState>,
    }

    impl ActiveStream for FakeStream {
        fn stop(self: Box<Self>) {
            self.state.stopped.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl StreamBackend for FakeBackend {
        fn current_displays(&mut self) -> Result<Vec<DisplayGeometry>, String> {
            let mut sets = self.state.display_sets.lock().unwrap();
            if sets.len() > 1 {
                Ok(sets.remove(0))
            } else {
                Ok(sets[0].clone())
            }
        }

        fn start_stream(
            &mut self,
            display: &DisplayGeometry,
            sink: FrameSink,
            _failure: FailureSink,
        ) -> Result<Box<dyn ActiveStream>, String> {
            self.state
                .started
                .lock()
                .unwrap()
                .push((display.id, sink));
            Ok(Box::new(FakeStream {
                state: self.state.clone(),
            }))
        }
    }

    fn harness(display_sets: Vec<Vec<DisplayGeometry>>) -> (Arc<FakeState>, Arc<Mutex<FrameBroker>>, StreamManager) {
        let state = Arc::new(FakeState {
            display_sets: Mutex::new(display_sets),
            ..FakeState::default()
        });
        let broker = Arc::new(Mutex::new(FrameBroker::new()));
        let failure: FailureSink = Arc::new(|error| panic!("unexpected failure: {error}"));
        let manager = StreamManager::start(
            FakeBackend {
                state: state.clone(),
            },
            broker.clone(),
            failure,
        )
        .unwrap();
        (state, broker, manager)
    }

    #[test]
    fn starts_one_stream_per_display_and_publishes_the_set() {
        let (state, broker, manager) = harness(vec![vec![display(1), display(2)]]);
        {
            let started = state.started.lock().unwrap();
            let ids: Vec<u32> = started.iter().map(|(id, _)| *id).collect();
            assert_eq!(ids, vec![1, 2]);
            // Frames flow through the sink into the broker.
            started[0].1(frame(&display(1), 1_000));
        }
        {
            let broker = broker.lock().unwrap();
            assert_eq!(broker.generation(), 1);
            assert_eq!(broker.displays().len(), 2);
            assert!(broker.snapshot(2_000).frame_for(1).is_some());
        }
        manager.stop();
        assert_eq!(state.stopped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_display_change_restarts_the_set_and_keeps_surviving_frames() {
        let (state, broker, manager) = harness(vec![
            vec![display(1), display(2)],
            vec![display(1), display(3)],
        ]);
        // Warm both displays, pin a lease.
        {
            let started = state.started.lock().unwrap();
            started[0].1(frame(&display(1), 1_000));
            started[1].1(frame(&display(2), 1_100));
        }
        let lease = broker.lock().unwrap().snapshot(1_200);

        manager.notify_reconfigured();
        assert!(
            wait_until(Duration::from_secs(5), || {
                broker.lock().unwrap().generation() == 2
            }),
            "manager did not publish the new generation",
        );
        // The outgoing generation's streams were stopped and replaced.
        assert!(wait_until(Duration::from_secs(5), || {
            state.started.lock().unwrap().len() == 4
        }));
        assert_eq!(state.stopped.load(Ordering::SeqCst), 2);
        {
            let broker = broker.lock().unwrap();
            let snapshot = broker.snapshot(2_000);
            // Surviving display keeps its outgoing-generation frame.
            assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, 1_000);
            // The new display has no frame yet (fail-stop path drives
            // events there); the removed display is out of the set.
            assert!(snapshot.frame_for(3).is_none());
            assert!(!broker.is_warm());
        }
        // The pre-change lease still holds the removed display's frame.
        assert_eq!(lease.frame_for(2).unwrap().ts_ns, 1_100);
        manager.stop();
        assert_eq!(state.stopped.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn an_empty_initial_display_set_refuses_to_start() {
        let state = Arc::new(FakeState {
            display_sets: Mutex::new(vec![vec![]]),
            ..FakeState::default()
        });
        let broker = Arc::new(Mutex::new(FrameBroker::new()));
        let failure: FailureSink = Arc::new(|_| {});
        let error = StreamManager::start(
            FakeBackend { state },
            broker,
            failure,
        )
        .err()
        .unwrap();
        assert!(error.contains("no active displays"), "got {error}");
    }
}
