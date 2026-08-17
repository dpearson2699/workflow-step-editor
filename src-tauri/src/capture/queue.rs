//! The bounded capture queue between the ListenOnly tap callback and
//! the single ordered capture worker.
//!
//! The tap side never blocks: enqueue is `try_send` on a bounded
//! channel, and a full queue is an explicit saturation signal that the
//! caller maps to the DEC-009 capture-overloaded fail-stop. No event is
//! silently dropped or coalesced: every accepted event is processed in
//! order, and the first rejected event fail-stops the recording.
//!
//! Capacity derives from a retained-frame byte budget: each queued job
//! pins at most one frame set (one frame per display), so the queue may
//! retain at most `capacity` frame sets beyond the broker's own.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::sync::Arc;

use crate::capture::broker::FrameSnapshot;
use crate::domain::schema::{KeyInfo, MouseButton};

/// The retained-frame byte budget the queue capacity is chosen from.
pub const RETAINED_FRAME_BYTE_BUDGET: u64 = 768 * 1024 * 1024;
/// Capacity clamp: enough headroom for bursts, bounded memory.
pub const MIN_QUEUE_CAPACITY: usize = 4;
pub const MAX_QUEUE_CAPACITY: usize = 64;

/// The raw input facts one tap callback copies out of the event.
#[derive(Debug, Clone, PartialEq)]
pub enum RawInput {
    Click { button: MouseButton },
    KeyDown { key: KeyInfo },
}

/// One queued capture job: the copied event facts plus the pinned
/// eligible-frame snapshot.
#[derive(Debug)]
pub struct CaptureJob {
    pub input: RawInput,
    /// Event location in global display points.
    pub x: f64,
    pub y: f64,
    /// Event timestamp in host-clock nanoseconds.
    pub ts_ns: u64,
    pub snapshot: FrameSnapshot,
}

/// Queue capacity from the byte budget: how many frame sets of the
/// current display configuration fit in the budget, clamped into
/// `[MIN_QUEUE_CAPACITY, MAX_QUEUE_CAPACITY]`.
pub fn queue_capacity(budget_bytes: u64, frame_set_bytes: u64) -> usize {
    let fitting = (budget_bytes / frame_set_bytes.max(1)) as usize;
    fitting.clamp(MIN_QUEUE_CAPACITY, MAX_QUEUE_CAPACITY)
}

/// Shared queue-depth accounting; the maximum observed depth is
/// reported when the pipeline stops.
#[derive(Debug, Default)]
pub struct QueueDepth {
    current: AtomicUsize,
    max: AtomicUsize,
}

impl QueueDepth {
    fn on_enqueue(&self) {
        let depth = self.current.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
        self.max.fetch_max(depth, Ordering::SeqCst);
    }

    /// Reverses `on_enqueue` for a job the channel rejected.
    fn on_rejected(&self) {
        self.current.fetch_sub(1, Ordering::SeqCst);
    }

    fn on_dequeue(&self) {
        self.current.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn max_observed(&self) -> usize {
        self.max.load(Ordering::SeqCst)
    }
}

/// Enqueue failure: saturation is the DEC-009 signal; a closed queue
/// means the worker is gone (stop already in progress).
#[derive(Debug, PartialEq, Eq)]
pub enum EnqueueError {
    Saturated,
    Closed,
}

/// The tap-side sender.
#[derive(Clone)]
pub struct JobSender {
    tx: SyncSender<CaptureJob>,
    depth: Arc<QueueDepth>,
}

impl JobSender {
    /// Nonblocking enqueue.
    pub fn enqueue(&self, job: CaptureJob) -> Result<(), EnqueueError> {
        // Account the depth before publishing the job: the worker can
        // receive and decrement immediately after `try_send` returns,
        // so the increment must already be visible or the counter
        // underflows. A rejected job rolls its increment back (the
        // maximum may transiently include one rejected job at the
        // saturation boundary, which the fail-stop makes moot).
        self.depth.on_enqueue();
        match self.tx.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                self.depth.on_rejected();
                Err(EnqueueError::Saturated)
            }
            Err(TrySendError::Disconnected(_)) => {
                self.depth.on_rejected();
                Err(EnqueueError::Closed)
            }
        }
    }

    pub fn depth(&self) -> Arc<QueueDepth> {
        self.depth.clone()
    }
}

/// The worker-side receiver; receiving keeps FIFO order.
pub struct JobReceiver {
    rx: Receiver<CaptureJob>,
    depth: Arc<QueueDepth>,
}

impl JobReceiver {
    /// Blocks for the next job; `None` once every sender is dropped and
    /// the queue is drained.
    pub fn recv(&self) -> Option<CaptureJob> {
        let job = self.rx.recv().ok()?;
        self.depth.on_dequeue();
        Some(job)
    }
}

/// Builds the bounded queue pair.
pub fn capture_queue(capacity: usize) -> (JobSender, JobReceiver) {
    let (tx, rx) = std::sync::mpsc::sync_channel(capacity);
    let depth = Arc::new(QueueDepth::default());
    (
        JobSender {
            tx,
            depth: depth.clone(),
        },
        JobReceiver { rx, depth },
    )
}

#[cfg(test)]
mod tests {
    use crate::capture::broker::FrameSnapshot;
    use crate::domain::schema::MouseButton;

    use super::*;

    fn job(ts_ns: u64) -> CaptureJob {
        CaptureJob {
            input: RawInput::Click {
                button: MouseButton::Left,
            },
            x: 10.0,
            y: 20.0,
            ts_ns,
            snapshot: FrameSnapshot::for_test(ts_ns, vec![], vec![]),
        }
    }

    #[test]
    fn capacity_comes_from_the_byte_budget_with_clamping() {
        // One Retina 3456x2234 BGRA frame set is ~30.9 MB.
        let frame_set = 3456_u64 * 2234 * 4;
        assert_eq!(queue_capacity(RETAINED_FRAME_BYTE_BUDGET, frame_set), 26);
        // A huge frame set clamps up to the minimum.
        assert_eq!(queue_capacity(RETAINED_FRAME_BYTE_BUDGET, u64::MAX / 2), 4);
        // A tiny frame set clamps down to the maximum.
        assert_eq!(queue_capacity(RETAINED_FRAME_BYTE_BUDGET, 1), 64);
        // A zero frame set must not divide by zero.
        assert_eq!(queue_capacity(RETAINED_FRAME_BYTE_BUDGET, 0), 64);
    }

    #[test]
    fn saturation_is_an_explicit_error_and_earlier_jobs_survive_in_order() {
        // DEC-009: with the worker stalled, the queue accepts exactly
        // `capacity` jobs, then reports saturation instead of blocking
        // or silently dropping.
        let (tx, rx) = capture_queue(2);
        tx.enqueue(job(1)).unwrap();
        tx.enqueue(job(2)).unwrap();
        assert_eq!(tx.enqueue(job(3)), Err(EnqueueError::Saturated));

        // Every accepted job is still there, in order.
        assert_eq!(rx.recv().unwrap().ts_ns, 1);
        assert_eq!(rx.recv().unwrap().ts_ns, 2);
        // The rejected job was never accepted; after the drain the
        // queue accepts again.
        tx.enqueue(job(4)).unwrap();
        assert_eq!(rx.recv().unwrap().ts_ns, 4);
    }

    #[test]
    fn a_closed_queue_reports_closed() {
        let (tx, rx) = capture_queue(2);
        drop(rx);
        assert_eq!(tx.enqueue(job(1)), Err(EnqueueError::Closed));
    }

    #[test]
    fn depth_tracks_the_observed_maximum() {
        let (tx, rx) = capture_queue(8);
        let depth = tx.depth();
        tx.enqueue(job(1)).unwrap();
        tx.enqueue(job(2)).unwrap();
        tx.enqueue(job(3)).unwrap();
        rx.recv().unwrap();
        tx.enqueue(job(4)).unwrap();
        assert_eq!(depth.max_observed(), 3);
    }
}
