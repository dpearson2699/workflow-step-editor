//! The single ordered capture worker behind the bounded queue.
//!
//! One worker thread drains jobs in FIFO order: resolve metadata,
//! select the display and the frame, assemble the packet (pure), and
//! emit through the guard. Ordering is structural — one thread, one
//! FIFO channel — so packets reach the coordinator in event order. A
//! packet-assembly failure (no retained frame, encode failure) reports
//! through the guard's single fail-stop.
//!
//! Frame selection is per event kind (DEC-001/DEC-002/DEC-006). A click
//! uses its pinned pre-event frame and never consults the live broker.
//! A key-down runs a bounded content-aware wait on this thread: it
//! waits, up to the event-anchored window deadline, for the oldest
//! retained frame on its display inside `(event_ts, event_ts + window]`
//! whose pixels inside the selected element crop differ from the pinned
//! pre-event frame's, and selects it as soon as it exists; at the
//! deadline it takes the newest in-window frame; when none exists, or
//! the candidate's display geometry differs from the event-time
//! display, the pinned frame is used. The wait runs on an injectable
//! [`WaitRuntime`] so tests drive a fake clock; the deadline is anchored
//! to the event timestamp, so a burst of key-downs on a static screen
//! shares one wait instead of stacking.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::capture::broker::{FrameBroker, FrameData};
use crate::capture::geometry::{CropPx, PointPt};
use crate::capture::health::EmitterGuard;
use crate::capture::packets::{
    build_packet, crop_pixels_differ, element_crop_px, element_info, select_pinned_frame,
    ResolvedMetadata, SelectedFrame,
};
use crate::capture::queue::{CaptureJob, JobReceiver, RawInput};
use crate::capture::resolver::MetadataResolver;

/// The clock and sleep the key-down post-event wait runs on. Production
/// supplies the mach host clock, `std::thread::sleep`, the DEC-002
/// window, and a short poll interval; tests supply a fake clock whose
/// `wait_for` advances time and publishes frames into the real broker.
pub trait WaitRuntime: Send {
    /// Now, in the same host-clock nanoseconds as event and frame
    /// timestamps.
    fn now_ns(&mut self) -> u64;

    /// Blocks the worker for `duration`.
    fn wait_for(&mut self, duration: Duration);

    /// The post-event window: the deadline is `event_ts + window_ns()`.
    fn window_ns(&self) -> u64;

    /// The re-query interval while waiting.
    fn poll_interval(&self) -> Duration;
}

/// Runs until the queue closes and is fully drained, so a stop never
/// silently discards an accepted event.
pub fn run_capture_worker(
    rx: JobReceiver,
    mut resolver: Box<dyn MetadataResolver>,
    broker: Arc<Mutex<FrameBroker>>,
    mut wait: impl WaitRuntime,
    guard: Arc<EmitterGuard>,
) {
    while let Some(job) = rx.recv() {
        let meta = match &job.input {
            RawInput::Click { .. } => resolver.resolve_click(PointPt { x: job.x, y: job.y }),
            RawInput::KeyDown { .. } => resolver.resolve_key_down(),
        };
        // The pinned pre-event frame for the selected display; its
        // absence remains the explicit fail-stop for every event kind.
        let pinned = match select_pinned_frame(&job, &meta) {
            Ok(pinned) => pinned,
            Err(error) => {
                guard.fail(error.to_string());
                continue;
            }
        };
        let selected = match &job.input {
            RawInput::Click { .. } => pinned,
            RawInput::KeyDown { .. } => {
                select_key_down_frame(&broker, &mut wait, &job, &meta, pinned)
            }
        };
        match build_packet(&job, &meta, &selected) {
            Ok(packet) => guard.packet(packet),
            Err(error) => guard.fail(error.to_string()),
        }
    }
}

/// The key-down rule (DEC-002/DEC-006): the content-aware post-event
/// frame on the pinned display, chosen by the bounded wait, when one
/// exists and matches the event-time display geometry (GA-006);
/// otherwise the pinned frame. The element crop compared is the same
/// crop `build_packet` cuts the element shot from on that display.
fn select_key_down_frame(
    broker: &Mutex<FrameBroker>,
    wait: &mut impl WaitRuntime,
    job: &CaptureJob,
    meta: &ResolvedMetadata,
    pinned: SelectedFrame,
) -> SelectedFrame {
    let element_crop = element_crop_px(&pinned.display, &element_info(job, meta, &pinned.display));
    match await_post_event_frame(broker, wait, &pinned, element_crop, job.ts_ns) {
        Some(frame) if frame.display == pinned.display => SelectedFrame {
            display: pinned.display,
            frame,
        },
        _ => pinned,
    }
}

/// Bounded content-aware wait on the pinned display inside
/// `(event_ts_ns, event_ts_ns + window]` (DEC-006). Each pass clones the
/// in-window candidates out of the broker (oldest first) and, outside
/// the lock, returns the oldest candidate of the pinned display's
/// geometry whose pixels inside `element_crop` differ from the pinned
/// frame's. Once the deadline has passed, the pass returns the newest
/// in-window candidate instead (or `None`); the pass after the last
/// wait is the final query. Otherwise it sleeps `min(poll, remaining)`
/// and re-queries. The total requested wait never exceeds the remaining
/// window; a job that arrives after its deadline queries once and never
/// waits. The broker lock is never held across a wait or a compare.
/// Frames are immutable and arrive in timestamp order, so a candidate
/// is compared once: later passes skip every candidate whose timestamp
/// is not newer than the newest one already compared.
fn await_post_event_frame(
    broker: &Mutex<FrameBroker>,
    wait: &mut impl WaitRuntime,
    pinned: &SelectedFrame,
    element_crop: CropPx,
    event_ts_ns: u64,
) -> Option<Arc<FrameData>> {
    let deadline_ns = event_ts_ns.saturating_add(wait.window_ns());
    let poll = wait.poll_interval();
    let mut compared_through_ts_ns = event_ts_ns;
    loop {
        let candidates = broker
            .lock()
            .expect("frame broker lock poisoned")
            .post_event_frames(pinned.display.id, event_ts_ns, deadline_ns);
        let changed = candidates.iter().find(|frame| {
            if frame.ts_ns <= compared_through_ts_ns {
                return false;
            }
            compared_through_ts_ns = frame.ts_ns;
            frame.display == pinned.display
                && crop_pixels_differ(frame, &pinned.frame, element_crop)
        });
        if changed.is_some() {
            return changed.cloned();
        }
        let now_ns = wait.now_ns();
        if now_ns >= deadline_ns {
            return candidates.last().cloned();
        }
        let remaining = Duration::from_nanos(deadline_ns.saturating_sub(now_ns));
        wait.wait_for(poll.min(remaining));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crate::capture::broker::{FrameBroker, FrameData, POST_EVENT_FRAME_WINDOW_NS};
    use crate::capture::geometry::{DisplayGeometry, RectPt};
    use crate::capture::packets::{ResolvedElement, ResolvedMetadata, ResolvedWindow};
    use crate::capture::queue::{capture_queue, CaptureJob, JobSender, RawInput};
    use crate::domain::schema::{KeyInfo, MouseButton};
    use crate::recording::pipeline::{CapturePacket, PacketEmitter, PacketInput, PipelineEvent};

    use super::*;

    const WINDOW_NS: u64 = POST_EVENT_FRAME_WINDOW_NS;
    const POLL: Duration = Duration::from_millis(5);
    const MS: u64 = 1_000_000;

    /// Deterministic resolver: clicks resolve a window around the
    /// point; key-downs return the configured metadata.
    #[derive(Default)]
    struct FixedResolver {
        key_down: ResolvedMetadata,
    }

    impl MetadataResolver for FixedResolver {
        fn resolve_click(&mut self, point: PointPt) -> ResolvedMetadata {
            ResolvedMetadata {
                window: Some(ResolvedWindow {
                    app: "TextEdit".into(),
                    title: "Untitled".into(),
                    pid: 871,
                    bounds_pt: RectPt::new(point.x - 5.0, point.y - 5.0, 20.0, 15.0),
                }),
                element: None,
                frontmost_app: Some("TextEdit".into()),
            }
        }

        fn resolve_key_down(&mut self) -> ResolvedMetadata {
            self.key_down.clone()
        }
    }

    /// What a scripted `wait_for` does once the fake clock reaches
    /// `at_ns`.
    enum ScriptStep {
        Publish(Arc<FrameData>),
        /// Publishes a new display-set generation (geometry change).
        PublishDisplays(Vec<DisplayGeometry>),
        /// Drops the queue's last sender while the worker waits.
        CloseSender,
    }

    /// The injected wait runtime: a fake host clock whose `wait_for`
    /// advances time, records the requested duration, and runs the
    /// scripted broker publications that fall due. No real sleeping.
    struct ScriptedRuntime {
        now_ns: u64,
        broker: Arc<Mutex<FrameBroker>>,
        script: Vec<(u64, ScriptStep)>,
        sender: Option<JobSender>,
        waits: Arc<Mutex<Vec<Duration>>>,
    }

    impl ScriptedRuntime {
        fn new(broker: Arc<Mutex<FrameBroker>>, now_ns: u64) -> Self {
            Self {
                now_ns,
                broker,
                script: Vec::new(),
                sender: None,
                waits: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn at(mut self, at_ns: u64, step: ScriptStep) -> Self {
            self.script.push((at_ns, step));
            self
        }

        fn waits(&self) -> Arc<Mutex<Vec<Duration>>> {
            self.waits.clone()
        }

        fn run_due_steps(&mut self) {
            let now_ns = self.now_ns;
            let (due, pending): (Vec<_>, Vec<_>) = self
                .script
                .drain(..)
                .partition(|(at_ns, _)| *at_ns <= now_ns);
            self.script = pending;
            for (_, step) in due {
                match step {
                    ScriptStep::Publish(frame) => self.broker.lock().unwrap().publish_frame(frame),
                    ScriptStep::PublishDisplays(displays) => {
                        self.broker.lock().unwrap().publish_displays(displays);
                    }
                    ScriptStep::CloseSender => drop(self.sender.take()),
                }
            }
        }
    }

    impl WaitRuntime for ScriptedRuntime {
        fn now_ns(&mut self) -> u64 {
            self.now_ns
        }

        fn wait_for(&mut self, duration: Duration) {
            self.waits.lock().unwrap().push(duration);
            self.now_ns += duration.as_nanos() as u64;
            self.run_due_steps();
        }

        fn window_ns(&self) -> u64 {
            WINDOW_NS
        }

        fn poll_interval(&self) -> Duration {
            POLL
        }
    }

    fn display(id: u32) -> DisplayGeometry {
        DisplayGeometry {
            id,
            frame_pt: RectPt::new(f64::from(id - 1) * 40.0, 0.0, 40.0, 30.0),
            scale: 1.0,
        }
    }

    /// A frame whose every pixel is BGRA `(b, g, r, 255)`; PNG shots
    /// decode to RGB `(r, g, b)`.
    fn frame(display: &DisplayGeometry, ts_ns: u64, bgr: [u8; 3]) -> Arc<FrameData> {
        let width = display.width_px();
        let height = display.height_px();
        Arc::new(FrameData {
            display: display.clone(),
            width_px: width,
            height_px: height,
            bytes_per_row: width as usize * 4,
            ts_ns,
            pixels: [bgr[0], bgr[1], bgr[2], 255].repeat(width as usize * height as usize),
        })
    }

    /// The focused text element on display 1, in global points: the
    /// element crop is pixels x 10..22, y 10..18.
    const ELEMENT_PT: RectPt = RectPt {
        x: 10.0,
        y: 10.0,
        w: 12.0,
        h: 8.0,
    };

    /// A key-down resolved to a focused window covering display 1 and
    /// the [`ELEMENT_PT`] text element inside it.
    fn element_resolver() -> FixedResolver {
        FixedResolver {
            key_down: ResolvedMetadata {
                window: Some(ResolvedWindow {
                    app: "TextEdit".into(),
                    title: "Untitled".into(),
                    pid: 871,
                    bounds_pt: RectPt::new(0.0, 0.0, 40.0, 30.0),
                }),
                element: Some(ResolvedElement {
                    role: Some("AXTextArea".into()),
                    title: None,
                    frame_pt: ELEMENT_PT,
                }),
                frontmost_app: Some("TextEdit".into()),
            },
        }
    }

    /// A frame of `base` pixels whose [`ELEMENT_PT`] region is painted
    /// `element` (a title-only repaint has `element == PINNED_BGR`; a
    /// glyph paint changes it). Its full shot's first pixel identifies
    /// the frame; its element shot's first pixel shows the element.
    fn frame_with_element(
        display: &DisplayGeometry,
        ts_ns: u64,
        base: [u8; 3],
        element: [u8; 3],
    ) -> Arc<FrameData> {
        let mut data = Arc::try_unwrap(frame(display, ts_ns, base)).unwrap();
        let width = display.width_px() as usize;
        for y in 10..18_usize {
            for x in 10..22_usize {
                let at = y * width * 4 + x * 4;
                data.pixels[at..at + 3].copy_from_slice(&element);
            }
        }
        Arc::new(data)
    }

    const PINNED_BGR: [u8; 3] = [10, 20, 30];
    const POST_BGR: [u8; 3] = [200, 150, 100];
    const OTHER_BGR: [u8; 3] = [1, 2, 3];
    const LATER_BGR: [u8; 3] = [90, 60, 40];

    fn rgb_of(bgr: [u8; 3]) -> [u8; 3] {
        [bgr[2], bgr[1], bgr[0]]
    }

    /// Asserts the packet's full shot comes from the frame with `base`
    /// pixels and its element shot shows `element` pixels.
    fn assert_full_and_element(packet: &CapturePacket, base: [u8; 3], element: [u8; 3]) {
        assert_eq!(first_pixel(&packet.shots.full), rgb_of(base));
        assert_eq!(first_pixel(&packet.shots.element), rgb_of(element));
    }

    fn first_pixel(png_bytes: &[u8]) -> [u8; 3] {
        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut buffer).unwrap();
        [buffer[0], buffer[1], buffer[2]]
    }

    /// Asserts every shot of the packet was cut from a frame with
    /// `bgr` pixels.
    fn assert_shots_from(packet: &CapturePacket, bgr: [u8; 3]) {
        for shot in [
            &packet.shots.full,
            &packet.shots.window,
            &packet.shots.element,
        ] {
            assert_eq!(first_pixel(shot), rgb_of(bgr));
        }
    }

    /// A warm broker: display 1 (and optionally 2) with one pinned
    /// pre-event frame at `pinned_ts`.
    fn warm_broker(displays: &[DisplayGeometry], pinned_ts: u64) -> Arc<Mutex<FrameBroker>> {
        let mut broker = FrameBroker::new();
        broker.publish_displays(displays.to_vec());
        for display in displays {
            broker.publish_frame(frame(display, pinned_ts, PINNED_BGR));
        }
        Arc::new(Mutex::new(broker))
    }

    fn key_job(broker: &Mutex<FrameBroker>, ts_ns: u64) -> CaptureJob {
        CaptureJob {
            input: RawInput::KeyDown {
                key: KeyInfo {
                    key_code: 4,
                    chars: "h".into(),
                    modifiers: vec![],
                },
            },
            x: 0.0,
            y: 0.0,
            ts_ns,
            snapshot: broker.lock().unwrap().snapshot(ts_ns),
        }
    }

    fn click_job(broker: &Mutex<FrameBroker>, ts_ns: u64, x: f64) -> CaptureJob {
        CaptureJob {
            input: RawInput::Click {
                button: MouseButton::Left,
            },
            x,
            y: 10.0,
            ts_ns,
            snapshot: broker.lock().unwrap().snapshot(ts_ns),
        }
    }

    fn guarded() -> (Arc<EmitterGuard>, mpsc::Receiver<PipelineEvent>) {
        let (event_tx, event_rx) = mpsc::channel();
        let guard = Arc::new(EmitterGuard::new(PacketEmitter::new(move |event| {
            let _ = event_tx.send(event);
        })));
        (guard, event_rx)
    }

    /// Runs the worker to completion on the current thread and returns
    /// the emitted events.
    fn run(
        rx: crate::capture::queue::JobReceiver,
        resolver: FixedResolver,
        broker: Arc<Mutex<FrameBroker>>,
        runtime: ScriptedRuntime,
    ) -> Vec<PipelineEvent> {
        let (guard, event_rx) = guarded();
        run_capture_worker(rx, Box::new(resolver), broker, runtime, guard);
        event_rx.try_iter().collect()
    }

    fn packets(events: &[PipelineEvent]) -> Vec<&CapturePacket> {
        events
            .iter()
            .map(|event| match event {
                PipelineEvent::Packet(packet) => packet.as_ref(),
                other => panic!("unexpected {other:?}"),
            })
            .collect()
    }

    fn total(waits: &Mutex<Vec<Duration>>) -> Duration {
        waits.lock().unwrap().iter().sum()
    }

    const EVENT: u64 = 1_000 * MS;

    #[test]
    fn drains_jobs_in_order_and_emits_ordered_packets() {
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        for i in 0..3_u32 {
            tx.enqueue(click_job(
                &broker,
                EVENT + u64::from(i),
                10.0 + f64::from(i),
            ))
            .unwrap();
        }
        drop(tx);
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + MS);
        let (guard, event_rx) = guarded();

        let worker = thread::spawn(move || {
            run_capture_worker(
                rx,
                Box::new(FixedResolver::default()),
                broker,
                runtime,
                guard,
            )
        });
        worker.join().unwrap();

        let events: Vec<PipelineEvent> = event_rx.try_iter().collect();
        let xs: Vec<f64> = packets(&events).iter().map(|packet| packet.pos.x).collect();
        assert_eq!(xs, vec![10.0, 11.0, 12.0]);
    }

    #[test]
    fn a_job_without_a_retained_frame_fail_stops_through_the_guard() {
        let broker = Arc::new(Mutex::new(FrameBroker::new()));
        broker.lock().unwrap().publish_displays(vec![display(1)]);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(click_job(&broker, EVENT, 10.0)).unwrap();
        // A key-down without a pinned frame fail-stops before any wait.
        tx.enqueue(key_job(&broker, EVENT + MS)).unwrap();
        drop(tx);
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + MS);
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        // Only the first failure is forwarded (DEC-007).
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], PipelineEvent::Failed(error)
            if error.contains("no retained pre-event frame")));
        assert!(waits.lock().unwrap().is_empty());
    }

    #[test]
    fn a_title_only_repaint_is_skipped_for_the_first_frame_that_changes_the_element() {
        // GA-007/DEC-006: the first keystroke's dirty-state title repaint
        // at +30 ms changes pixels only outside the focused element; the
        // glyph frame at +100 ms changes pixels inside it. The glyph
        // frame is selected and the wait ends on that frame, not at the
        // deadline.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        // The worker picks the job up 5 ms after the event.
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 5 * MS)
            .at(
                EVENT + 30 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 30 * MS,
                    OTHER_BGR,
                    PINNED_BGR,
                )),
            )
            .at(
                EVENT + 100 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 100 * MS,
                    POST_BGR,
                    POST_BGR,
                )),
            );
        let waits = runtime.waits();

        let events = run(rx, element_resolver(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_full_and_element(packets[0], POST_BGR, POST_BGR);
        assert_eq!(packets[0].frame_age_ms, 0);
        // Polled in short steps up to the glyph frame; well short of the
        // remaining window.
        assert!(waits.lock().unwrap().iter().all(|wait| *wait <= POLL));
        assert_eq!(total(&waits), Duration::from_millis(95));
    }

    #[test]
    fn the_oldest_frame_that_changes_the_element_wins_over_a_later_one() {
        // GA-009 (fast typing): the frame at +30 ms already shows the
        // glyph inside the element; a later in-window frame with more
        // changes must not supersede it, and the wait ends at +30 ms.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 5 * MS)
            .at(
                EVENT + 30 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 30 * MS,
                    POST_BGR,
                    POST_BGR,
                )),
            )
            .at(
                EVENT + 100 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 100 * MS,
                    LATER_BGR,
                    LATER_BGR,
                )),
            );
        let waits = runtime.waits();

        let events = run(rx, element_resolver(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_full_and_element(packets[0], POST_BGR, POST_BGR);
        assert_eq!(packets[0].frame_age_ms, 0);
        assert_eq!(total(&waits), Duration::from_millis(25));
    }

    #[test]
    fn no_element_change_before_the_deadline_selects_the_newest_in_window_frame() {
        // Every in-window frame is identical to the pinned frame inside
        // the element crop (repaints elsewhere only): the worker waits
        // out the window and the final query takes the newest one.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 5 * MS)
            .at(
                EVENT + 30 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 30 * MS,
                    OTHER_BGR,
                    PINNED_BGR,
                )),
            )
            .at(
                EVENT + 100 * MS,
                ScriptStep::Publish(frame_with_element(
                    &displays[0],
                    EVENT + 100 * MS,
                    LATER_BGR,
                    PINNED_BGR,
                )),
            );
        let waits = runtime.waits();

        let events = run(rx, element_resolver(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_full_and_element(packets[0], LATER_BGR, PINNED_BGR);
        assert_eq!(packets[0].frame_age_ms, 0);
        // The whole remaining window was waited out, in short steps.
        assert_eq!(total(&waits), Duration::from_nanos(WINDOW_NS - 5 * MS));
        assert!(waits.lock().unwrap().iter().all(|wait| *wait <= POLL));
    }

    #[test]
    fn a_single_early_in_window_frame_that_changes_the_element_is_selected_at_once() {
        // Without a resolved element the fallback crop covers the whole
        // small display, so a uniformly repainted post frame counts as
        // an element change and is selected as soon as it exists.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        // The worker picks the job up 2 ms after the event; the stream
        // publishes one post frame 60 ms after the event.
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 2 * MS).at(
            EVENT + 60 * MS,
            ScriptStep::Publish(frame(&displays[0], EVENT + 60 * MS, POST_BGR)),
        );
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_shots_from(packets[0], POST_BGR);
        assert_eq!(packets[0].frame_age_ms, 0);
        // Polled in 5 ms steps from +2 ms; the poll that reaches +62 ms
        // sees the frame. Never past the remaining window.
        let waits = waits.lock().unwrap();
        assert!(waits.iter().all(|wait| *wait <= POLL));
        assert_eq!(waits.iter().sum::<Duration>(), Duration::from_millis(60));
    }

    #[test]
    fn a_silent_key_down_falls_back_to_the_pinned_frame_at_the_deadline() {
        // No frame arrives inside the window: the worker actively
        // reaches the deadline on the fake clock, emits the pinned
        // frame, and never fails.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        // A frame just after the deadline must not be selected either.
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 2 * MS).at(
            EVENT + WINDOW_NS + MS,
            ScriptStep::Publish(frame(&displays[0], EVENT + WINDOW_NS + MS, POST_BGR)),
        );
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_shots_from(packets[0], PINNED_BGR);
        assert_eq!(packets[0].frame_age_ms, 5);
        // The total requested wait is exactly the remaining window.
        assert_eq!(total(&waits), Duration::from_nanos(WINDOW_NS - 2 * MS));
        assert!(waits.lock().unwrap().iter().all(|wait| *wait <= POLL));
    }

    #[test]
    fn a_late_job_takes_a_retained_in_window_frame_without_waiting() {
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        // The in-window frame is retained before the worker even runs,
        // and the worker runs after the deadline.
        broker
            .lock()
            .unwrap()
            .publish_frame(frame(&displays[0], EVENT + 80 * MS, POST_BGR));
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + WINDOW_NS + 50 * MS);
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_shots_from(packets[0], POST_BGR);
        assert_eq!(packets[0].frame_age_ms, 0);
        assert!(waits.lock().unwrap().is_empty());
    }

    #[test]
    fn a_late_job_with_only_an_after_deadline_frame_uses_the_pinned_frame_immediately() {
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        broker.lock().unwrap().publish_frame(frame(
            &displays[0],
            EVENT + WINDOW_NS + 10 * MS,
            POST_BGR,
        ));
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + WINDOW_NS + 50 * MS);
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_shots_from(packets[0], PINNED_BGR);
        assert!(waits.lock().unwrap().is_empty());
    }

    #[test]
    fn several_key_downs_before_one_frame_share_it_without_stacking_waits() {
        // Three key-downs 20 ms apart, then one changed frame 140 ms
        // after the first: the first job waits for it and the later jobs
        // find it retained (and changed) without waiting a full window
        // each.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        for i in 0..3_u64 {
            tx.enqueue(key_job(&broker, EVENT + i * 20 * MS)).unwrap();
        }
        drop(tx);
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 45 * MS).at(
            EVENT + 140 * MS,
            ScriptStep::Publish(frame(&displays[0], EVENT + 140 * MS, POST_BGR)),
        );
        let waits = runtime.waits();

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 3);
        for packet in &packets {
            assert_shots_from(packet, POST_BGR);
            assert_eq!(packet.frame_age_ms, 0);
        }
        // Only the first job waited (45 ms to 140 ms); the rest were
        // served from the retained frame.
        assert_eq!(total(&waits), Duration::from_millis(95));
    }

    #[test]
    fn a_click_between_key_downs_keeps_its_pinned_frame_and_emitter_order() {
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        tx.enqueue(click_job(&broker, EVENT + 10 * MS, 12.0))
            .unwrap();
        tx.enqueue(key_job(&broker, EVENT + 20 * MS)).unwrap();
        drop(tx);
        // A changed broker frame arrives before the worker starts and a
        // second one during the last key-down's wait.
        broker
            .lock()
            .unwrap()
            .publish_frame(frame(&displays[0], EVENT + 5 * MS, POST_BGR));
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 25 * MS).at(
            EVENT + 110 * MS,
            ScriptStep::Publish(frame(&displays[0], EVENT + 110 * MS, OTHER_BGR)),
        );

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 3);
        // Emitter order equals enqueue order.
        assert!(matches!(packets[0].input, PacketInput::KeyDown { .. }));
        assert!(matches!(packets[1].input, PacketInput::Click { .. }));
        assert!(matches!(packets[2].input, PacketInput::KeyDown { .. }));
        // First key-down: the retained +5 ms frame already changed the
        // element, so it is selected without waiting.
        assert_shots_from(packets[0], POST_BGR);
        // The click keeps its pinned pre-event pixels although the
        // broker retains newer frames.
        assert_shots_from(packets[1], PINNED_BGR);
        assert_eq!(packets[1].frame_age_ms, 15);
        // Second key-down at +20 ms: the +5 ms frame is not later than
        // the event, so it waits for the changed +110 ms frame.
        assert_shots_from(packets[2], OTHER_BGR);
    }

    #[test]
    fn a_key_down_on_a_secondary_display_ignores_a_primary_post_frame() {
        let displays = [display(1), display(2)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        // The focused window sits on display 2.
        let resolver = FixedResolver {
            key_down: ResolvedMetadata {
                window: Some(ResolvedWindow {
                    app: "TextEdit".into(),
                    title: "Untitled".into(),
                    pid: 871,
                    bounds_pt: RectPt::new(45.0, 5.0, 20.0, 15.0),
                }),
                ..Default::default()
            },
        };
        // A changed frame on display 1 neither ends the wait nor gets
        // selected for display 2.
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 2 * MS).at(
            EVENT + 100 * MS,
            ScriptStep::Publish(frame(&displays[0], EVENT + 100 * MS, POST_BGR)),
        );
        let waits = runtime.waits();

        let events = run(rx, resolver, broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets[0].display_id, 2);
        assert_shots_from(packets[0], PINNED_BGR);
        assert_eq!(total(&waits), Duration::from_nanos(WINDOW_NS - 2 * MS));
    }

    #[test]
    fn a_post_frame_with_changed_display_geometry_falls_back_to_the_pinned_frame() {
        // GA-006: the display set is republished with a new geometry
        // for the same display ID inside the window; the only candidate
        // frame carries that geometry, so it is neither compared nor
        // selected and the pinned frame is used.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        drop(tx);
        let moved = DisplayGeometry {
            frame_pt: RectPt::new(100.0, 0.0, 40.0, 30.0),
            ..display(1)
        };
        let runtime = ScriptedRuntime::new(broker.clone(), EVENT + 2 * MS)
            .at(
                EVENT + 20 * MS,
                ScriptStep::PublishDisplays(vec![moved.clone()]),
            )
            .at(
                EVENT + 100 * MS,
                ScriptStep::Publish(frame(&moved, EVENT + 100 * MS, POST_BGR)),
            );

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_shots_from(packets[0], PINNED_BGR);
    }

    #[test]
    fn a_key_down_accepted_before_the_sender_closes_still_gets_its_frame_and_the_worker_exits() {
        // Orderly stop: the tap closes the sender while the worker waits
        // for a key-down's frame; the streams keep publishing, the job
        // emits with the post frame, and the worker exits on the drain.
        let displays = [display(1)];
        let broker = warm_broker(&displays, EVENT - 5 * MS);
        let (tx, rx) = capture_queue(8);
        tx.enqueue(key_job(&broker, EVENT)).unwrap();
        let mut runtime = ScriptedRuntime::new(broker.clone(), EVENT + 2 * MS)
            .at(EVENT + 20 * MS, ScriptStep::CloseSender)
            .at(
                EVENT + 100 * MS,
                ScriptStep::Publish(frame(&displays[0], EVENT + 100 * MS, POST_BGR)),
            );
        runtime.sender = Some(tx);

        let events = run(rx, FixedResolver::default(), broker, runtime);

        let packets = packets(&events);
        assert_eq!(packets.len(), 1);
        assert_shots_from(packets[0], POST_BGR);
    }
}
