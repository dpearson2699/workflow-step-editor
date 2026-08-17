//! The single ordered capture worker behind the bounded queue.
//!
//! One worker thread drains jobs in FIFO order: resolve metadata,
//! assemble the packet (pure), and emit through the guard. Ordering is
//! structural — one thread, one FIFO channel — so packets reach the
//! coordinator in event order. A packet-assembly failure (no retained
//! frame, encode failure) reports through the guard's single fail-stop.

use std::sync::Arc;

use crate::capture::health::EmitterGuard;
use crate::capture::geometry::PointPt;
use crate::capture::packets::build_packet;
use crate::capture::queue::{JobReceiver, RawInput};
use crate::capture::resolver::MetadataResolver;

/// Runs until the queue closes and is fully drained, so a stop never
/// silently discards an accepted event.
pub fn run_capture_worker(
    rx: JobReceiver,
    mut resolver: Box<dyn MetadataResolver>,
    guard: Arc<EmitterGuard>,
) {
    while let Some(job) = rx.recv() {
        let meta = match &job.input {
            RawInput::Click { .. } => resolver.resolve_click(PointPt { x: job.x, y: job.y }),
            RawInput::KeyDown { .. } => resolver.resolve_key_down(),
        };
        match build_packet(&job, &meta) {
            Ok(packet) => guard.packet(packet),
            Err(error) => guard.fail(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    use crate::capture::broker::{FrameBroker, FrameData};
    use crate::capture::geometry::{DisplayGeometry, RectPt};
    use crate::capture::packets::{ResolvedMetadata, ResolvedWindow};
    use crate::capture::queue::{capture_queue, CaptureJob, RawInput};
    use crate::domain::schema::MouseButton;
    use crate::recording::pipeline::{PacketEmitter, PipelineEvent};

    use super::*;

    struct FixedResolver;

    impl MetadataResolver for FixedResolver {
        fn resolve_click(&mut self, point: PointPt) -> ResolvedMetadata {
            ResolvedMetadata {
                window: Some(ResolvedWindow {
                    app: "TextEdit".into(),
                    title: "Untitled".into(),
                    pid: 871,
                    bounds_pt: RectPt::new(point.x - 50.0, point.y - 50.0, 200.0, 150.0),
                }),
                element: None,
                frontmost_app: Some("TextEdit".into()),
            }
        }

        fn resolve_key_down(&mut self) -> ResolvedMetadata {
            ResolvedMetadata::default()
        }
    }

    fn display() -> DisplayGeometry {
        DisplayGeometry {
            id: 1,
            frame_pt: RectPt::new(0.0, 0.0, 400.0, 300.0),
            scale: 1.0,
        }
    }

    fn broker_with_frame(ts_ns: u64) -> FrameBroker {
        let display = display();
        let mut broker = FrameBroker::new();
        broker.publish_displays(vec![display.clone()]);
        broker.publish_frame(Arc::new(FrameData {
            width_px: display.width_px(),
            height_px: display.height_px(),
            bytes_per_row: display.width_px() as usize * 4,
            ts_ns,
            pixels: vec![10; display.width_px() as usize * 4 * display.height_px() as usize],
            display,
        }));
        broker
    }

    #[test]
    fn drains_jobs_in_order_and_emits_ordered_packets() {
        let broker = broker_with_frame(1_000_000);
        let (tx, rx) = capture_queue(8);
        let (event_tx, event_rx) = mpsc::channel();
        let guard = Arc::new(crate::capture::health::EmitterGuard::new(
            PacketEmitter::new(move |event| {
                let _ = event_tx.send(event);
            }),
        ));

        for i in 0..3_u32 {
            tx.enqueue(CaptureJob {
                input: RawInput::Click {
                    button: MouseButton::Left,
                },
                x: 100.0 + f64::from(i),
                y: 100.0,
                ts_ns: 2_000_000 + u64::from(i),
                snapshot: broker.snapshot(2_000_000 + u64::from(i)),
            })
            .unwrap();
        }
        drop(tx);

        let worker = thread::spawn(move || {
            run_capture_worker(rx, Box::new(FixedResolver), guard)
        });
        worker.join().unwrap();

        let events: Vec<PipelineEvent> = event_rx.try_iter().collect();
        assert_eq!(events.len(), 3);
        let xs: Vec<f64> = events
            .iter()
            .map(|event| match event {
                PipelineEvent::Packet(packet) => packet.pos.x,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(xs, vec![100.0, 101.0, 102.0]);
    }

    #[test]
    fn a_job_without_a_retained_frame_fail_stops_through_the_guard() {
        let display = display();
        let mut broker = FrameBroker::new();
        broker.publish_displays(vec![display]);
        let (tx, rx) = capture_queue(8);
        let (event_tx, event_rx) = mpsc::channel();
        let guard = Arc::new(crate::capture::health::EmitterGuard::new(
            PacketEmitter::new(move |event| {
                let _ = event_tx.send(event);
            }),
        ));

        tx.enqueue(CaptureJob {
            input: RawInput::Click {
                button: MouseButton::Left,
            },
            x: 100.0,
            y: 100.0,
            ts_ns: 2_000_000,
            snapshot: broker.snapshot(2_000_000),
        })
        .unwrap();
        drop(tx);
        run_capture_worker(rx, Box::new(FixedResolver), guard);

        let events: Vec<PipelineEvent> = event_rx.try_iter().collect();
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], PipelineEvent::Failed(error)
            if error.contains("no retained pre-event frame")));
    }
}
