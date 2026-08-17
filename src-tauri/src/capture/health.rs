//! The health/failure adapter between the capture threads and the
//! coordinator's emitter.
//!
//! Every failure source — tap disable, stream failure, permission loss,
//! queue saturation (DEC-009), and an event without any retained frame —
//! reports through [`EmitterGuard::fail`], which forwards exactly one
//! failure into the coordinator's single fail-stop path (DEC-007) no
//! matter how many sources race.
//!
//! [`EmitterGuard::close`] owns the `CapturePipeline::stop` quiescence
//! contract: it takes the emitter under the write lock, so it cannot
//! return while any emitter call still runs under a read lock on
//! another thread.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use crate::recording::pipeline::{CapturePacket, PacketEmitter};

#[derive(Debug)]
pub struct EmitterGuard {
    emitter: RwLock<Option<PacketEmitter>>,
    failed: AtomicBool,
}

impl EmitterGuard {
    pub fn new(emitter: PacketEmitter) -> Self {
        Self {
            emitter: RwLock::new(Some(emitter)),
            failed: AtomicBool::new(false),
        }
    }

    /// Emits one ordered packet. A packet after `close` is silently
    /// dropped (stale callback; the recording already finalized).
    pub fn packet(&self, packet: CapturePacket) {
        let emitter = self.emitter.read().expect("emitter lock poisoned");
        if let Some(emitter) = emitter.as_ref() {
            emitter.packet(packet);
        }
    }

    /// Reports a failure. Only the first reported failure is forwarded;
    /// the coordinator then runs its single fail-stop transition.
    pub fn fail(&self, error: impl Into<String>) {
        if self.failed.swap(true, Ordering::SeqCst) {
            return;
        }
        let emitter = self.emitter.read().expect("emitter lock poisoned");
        if let Some(emitter) = emitter.as_ref() {
            emitter.failed(error.into());
        }
    }

    pub fn has_failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    /// Quiesces the guard: blocks until no emitter call is in flight,
    /// then drops the emitter so later calls are no-ops.
    pub fn close(&self) {
        self.emitter
            .write()
            .expect("emitter lock poisoned")
            .take();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    use crate::recording::fake_pipeline::click_packet;
    use crate::recording::pipeline::PipelineEvent;

    use super::*;

    fn guarded() -> (Arc<EmitterGuard>, mpsc::Receiver<PipelineEvent>) {
        let (tx, rx) = mpsc::channel();
        let emitter = PacketEmitter::new(move |event| {
            let _ = tx.send(event);
        });
        (Arc::new(EmitterGuard::new(emitter)), rx)
    }

    #[test]
    fn racing_failure_sources_forward_exactly_one_fail_stop() {
        // Tap disable, stream failure, and permission loss racing from
        // separate threads yield one forwarded failure (DEC-007).
        let (guard, rx) = guarded();
        let sources = ["event tap disabled", "stream failed", "permission lost"];
        let handles: Vec<_> = sources
            .iter()
            .copied()
            .map(|error| {
                let guard = guard.clone();
                thread::spawn(move || guard.fail(error))
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        let events: Vec<PipelineEvent> = rx.try_iter().collect();
        assert_eq!(events.len(), 1, "got {events:?}");
        assert!(matches!(&events[0], PipelineEvent::Failed(error)
            if sources.contains(&error.as_str())));
        assert!(guard.has_failed());
    }

    #[test]
    fn packets_flow_until_close_then_drop_silently() {
        let (guard, rx) = guarded();
        guard.packet(click_packet());
        guard.close();
        guard.packet(click_packet());
        guard.fail("late failure");

        let events: Vec<PipelineEvent> = rx.try_iter().collect();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], PipelineEvent::Packet(_)));
    }
}
