//! The frame-set broker: timestamped frames per display on one
//! monotonic host clock.
//!
//! Streams publish frames; the tap callback pins an immutable
//! eligible-frame snapshot for the event's timestamp before the event
//! is enqueued, so a delayed worker can never lose the required
//! predecessor frame when the live broker advances (ADR 0001). The
//! capture worker additionally queries the live broker for a key-down's
//! post-event frame inside the bounded window
//! `(event_ts, event_ts + POST_EVENT_FRAME_WINDOW_NS]` (ADR 0001
//! amendment, DEC-002); the pinned snapshot stays the fallback.
//!
//! Display sets are published per generation and replace each other
//! atomically under the broker lock. Frames retained for a display
//! that survives a generation change stay available — an event during
//! warm-up uses the outgoing generation's newest frame — and existing
//! pinned leases (`Arc<FrameData>`) stay valid forever.

use std::collections::HashMap;
use std::sync::Arc;

use crate::capture::geometry::DisplayGeometry;

/// The bounded post-event window for key-down frame selection
/// (DEC-002): a key-down step uses the oldest retained frame whose
/// timestamp lies in `(event_ts, event_ts + 250 ms]`, else its pinned
/// pre-event frame. About 2.5 minimum frame intervals at ~10 fps.
pub const POST_EVENT_FRAME_WINDOW_NS: u64 = 250_000_000;

/// One captured frame: the pixels plus the display geometry they were
/// captured under (which may be an older generation than the current
/// display set).
#[derive(Debug)]
pub struct FrameData {
    pub display: DisplayGeometry,
    pub width_px: u32,
    pub height_px: u32,
    pub bytes_per_row: usize,
    /// Host-clock nanoseconds of the frame's presentation time.
    pub ts_ns: u64,
    /// Tightly packed BGRA pixels (`bytes_per_row * height_px` bytes).
    pub pixels: Vec<u8>,
}

/// The retained frames of one display: the newest frame plus its
/// predecessor, so an event whose timestamp precedes the newest frame
/// still finds its pre-event frame.
#[derive(Debug, Default)]
struct RetainedFrames {
    newest: Option<Arc<FrameData>>,
    previous: Option<Arc<FrameData>>,
}

impl RetainedFrames {
    /// The newest retained frame not later than `event_ts_ns`. When
    /// every retained frame is newer than the event (a callback delayed
    /// past two frame intervals), the oldest retained frame is the
    /// closest available approximation and is selected; its age then
    /// clamps to zero. Equal timestamps are eligible ("not later").
    fn eligible(&self, event_ts_ns: u64) -> Option<Arc<FrameData>> {
        if let Some(newest) = &self.newest {
            if newest.ts_ns <= event_ts_ns {
                return Some(newest.clone());
            }
        }
        if let Some(previous) = &self.previous {
            if previous.ts_ns <= event_ts_ns {
                return Some(previous.clone());
            }
        }
        self.previous.clone().or_else(|| self.newest.clone())
    }

    /// The retained frame with the smallest timestamp inside
    /// `(event_ts_ns, deadline_ns]`, or `None`. Both retained slots are
    /// compared (`previous` is not assumed older than `newest`). A
    /// frame equal to the event is not eligible; a frame equal to the
    /// deadline is.
    fn oldest_in_window(&self, event_ts_ns: u64, deadline_ns: u64) -> Option<Arc<FrameData>> {
        [&self.newest, &self.previous]
            .into_iter()
            .flatten()
            .filter(|frame| frame.ts_ns > event_ts_ns && frame.ts_ns <= deadline_ns)
            .min_by_key(|frame| frame.ts_ns)
            .cloned()
    }
}

/// An immutable snapshot pinned at event time: the display set current
/// at the event plus, per display, the eligible frame lease.
#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub event_ts_ns: u64,
    pub displays: Arc<Vec<DisplayGeometry>>,
    frames: HashMap<u32, Arc<FrameData>>,
}

impl FrameSnapshot {
    /// The pinned frame for a display, or `None` when the display
    /// retains no frame at all (explicit fail-stop path, DEC-007).
    pub fn frame_for(&self, display_id: u32) -> Option<&Arc<FrameData>> {
        self.frames.get(&display_id)
    }

    /// `frame_age_ms` for a pinned frame: the event-to-frame timestamp
    /// comparison on the shared host clock, clamped at zero.
    pub fn frame_age_ms(&self, frame: &FrameData) -> u64 {
        self.event_ts_ns.saturating_sub(frame.ts_ns) / 1_000_000
    }

    #[cfg(test)]
    pub fn for_test(
        event_ts_ns: u64,
        displays: Vec<DisplayGeometry>,
        frames: Vec<Arc<FrameData>>,
    ) -> Self {
        Self {
            event_ts_ns,
            displays: Arc::new(displays),
            frames: frames
                .into_iter()
                .map(|frame| (frame.display.id, frame))
                .collect(),
        }
    }
}

/// The live broker. Shared behind a mutex; every operation is
/// constant-bounded in the number of displays, so the tap callback's
/// snapshot never blocks on frame-sized work.
#[derive(Debug, Default)]
pub struct FrameBroker {
    generation: u64,
    displays: Arc<Vec<DisplayGeometry>>,
    frames: HashMap<u32, RetainedFrames>,
}

impl FrameBroker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn displays(&self) -> Arc<Vec<DisplayGeometry>> {
        self.displays.clone()
    }

    /// Publishes a new display-set generation atomically. Frames
    /// retained for displays that survive the change carry over (the
    /// outgoing generation's newest frame keeps serving events during
    /// warm-up); frames of removed displays are dropped from the live
    /// set, while already-pinned leases stay valid.
    pub fn publish_displays(&mut self, displays: Vec<DisplayGeometry>) -> u64 {
        self.generation += 1;
        self.frames.retain(|id, _| displays.iter().any(|d| d.id == *id));
        self.displays = Arc::new(displays);
        self.generation
    }

    /// Publishes one frame for its display.
    pub fn publish_frame(&mut self, frame: Arc<FrameData>) {
        let retained = self.frames.entry(frame.display.id).or_default();
        retained.previous = retained.newest.take();
        retained.newest = Some(frame);
    }

    /// True once every display of the current set retains at least one
    /// frame (first-frame warm-up gate before the tap enables).
    pub fn is_warm(&self) -> bool {
        !self.displays.is_empty()
            && self.displays.iter().all(|d| {
                self.frames
                    .get(&d.id)
                    .is_some_and(|retained| retained.newest.is_some())
            })
    }

    /// Pins the eligible-frame snapshot for an event timestamp.
    pub fn snapshot(&self, event_ts_ns: u64) -> FrameSnapshot {
        let mut frames = HashMap::with_capacity(self.displays.len());
        for display in self.displays.iter() {
            if let Some(frame) = self
                .frames
                .get(&display.id)
                .and_then(|retained| retained.eligible(event_ts_ns))
            {
                frames.insert(display.id, frame);
            }
        }
        FrameSnapshot {
            event_ts_ns,
            displays: self.displays.clone(),
            frames,
        }
    }

    /// The key-down post-event query (DEC-002): the oldest retained
    /// frame on `display_id` whose timestamp lies in
    /// `(event_ts_ns, deadline_ns]`, or `None` when the display retains
    /// no such frame. Pure and constant-bounded; the caller owns the
    /// bounded wait and never holds the broker lock while waiting.
    pub fn post_event_frame(
        &self,
        display_id: u32,
        event_ts_ns: u64,
        deadline_ns: u64,
    ) -> Option<Arc<FrameData>> {
        self.frames
            .get(&display_id)
            .and_then(|retained| retained.oldest_in_window(event_ts_ns, deadline_ns))
    }
}

#[cfg(test)]
mod tests {
    use crate::capture::geometry::RectPt;

    use super::*;

    fn display(id: u32) -> DisplayGeometry {
        DisplayGeometry {
            id,
            frame_pt: RectPt::new(0.0, 0.0, 100.0, 100.0),
            scale: 1.0,
        }
    }

    fn frame(id: u32, ts_ns: u64) -> Arc<FrameData> {
        Arc::new(FrameData {
            display: display(id),
            width_px: 100,
            height_px: 100,
            bytes_per_row: 400,
            ts_ns,
            pixels: vec![0; 400 * 100],
        })
    }

    fn broker_with(displays: Vec<u32>) -> FrameBroker {
        let mut broker = FrameBroker::new();
        broker.publish_displays(displays.into_iter().map(display).collect());
        broker
    }

    #[test]
    fn selects_the_newest_frame_not_later_than_the_event() {
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, 1_000));
        broker.publish_frame(frame(1, 2_000));

        // Event after both frames: the newest wins.
        let snapshot = broker.snapshot(2_500);
        assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, 2_000);
        // Event between the frames: the predecessor wins.
        let snapshot = broker.snapshot(1_500);
        assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, 1_000);
    }

    #[test]
    fn an_equal_timestamp_is_eligible() {
        // Clock-domain conversion can land a frame and an event on the
        // same nanosecond; "not later" includes equality and the age is
        // zero.
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, 1_000));
        broker.publish_frame(frame(1, 2_000));

        let snapshot = broker.snapshot(2_000);
        let selected = snapshot.frame_for(1).unwrap();
        assert_eq!(selected.ts_ns, 2_000);
        assert_eq!(snapshot.frame_age_ms(selected), 0);
    }

    #[test]
    fn a_delayed_callback_still_finds_its_pre_event_frame() {
        // The event fired at t=1500 but its callback ran after the
        // t=2000 frame arrived: selection must go to the t=1000 frame.
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, 1_000));
        broker.publish_frame(frame(1, 2_000));

        let snapshot = broker.snapshot(1_500);
        let selected = snapshot.frame_for(1).unwrap();
        assert_eq!(selected.ts_ns, 1_000);
        assert_eq!(snapshot.frame_age_ms(selected), 0);

        // A 600 ms old pre-event frame reports exactly 600.
        let snapshot = broker.snapshot(2_000 + 600_000_000);
        assert_eq!(snapshot.frame_age_ms(snapshot.frame_for(1).unwrap()), 600);
    }

    #[test]
    fn an_event_older_than_every_retained_frame_takes_the_oldest() {
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, 5_000));
        broker.publish_frame(frame(1, 6_000));

        let snapshot = broker.snapshot(1_000);
        let selected = snapshot.frame_for(1).unwrap();
        assert_eq!(selected.ts_ns, 5_000);
        // Age clamps to zero instead of going negative.
        assert_eq!(snapshot.frame_age_ms(selected), 0);
    }

    #[test]
    fn advancing_the_broker_after_a_snapshot_keeps_the_pinned_predecessor() {
        // The broker-advance race: the snapshot pins its frame before
        // the enqueue; later frames must not replace it.
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, 1_000));

        let snapshot = broker.snapshot(1_500);
        broker.publish_frame(frame(1, 2_000));
        broker.publish_frame(frame(1, 3_000));

        assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, 1_000);
        // The live broker itself has advanced.
        assert_eq!(broker.snapshot(3_500).frame_for(1).unwrap().ts_ns, 3_000);
    }

    #[test]
    fn display_set_replacement_keeps_surviving_frames_and_live_leases() {
        let mut broker = broker_with(vec![1, 2]);
        broker.publish_frame(frame(1, 1_000));
        broker.publish_frame(frame(2, 1_100));
        let lease = broker.snapshot(1_200);

        // Display 2 leaves, display 3 arrives.
        let generation = broker.publish_displays(vec![display(1), display(3)]);
        assert_eq!(generation, 2);

        // The surviving display still serves its outgoing-generation
        // frame during warm-up.
        let snapshot = broker.snapshot(1_300);
        assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, 1_000);
        // The new display retains nothing yet: explicit fail-stop path.
        assert!(snapshot.frame_for(3).is_none());
        assert!(!broker.is_warm());
        // The pre-replacement lease still holds the removed display's
        // frame.
        assert_eq!(lease.frame_for(2).unwrap().ts_ns, 1_100);
    }

    #[test]
    fn warm_up_requires_a_frame_on_every_current_display() {
        let mut broker = broker_with(vec![1, 2]);
        assert!(!broker.is_warm());
        broker.publish_frame(frame(1, 1_000));
        assert!(!broker.is_warm());
        broker.publish_frame(frame(2, 1_050));
        assert!(broker.is_warm());
    }

    #[test]
    fn an_empty_display_set_is_never_warm() {
        let broker = FrameBroker::new();
        assert!(!broker.is_warm());
    }

    // --- Key-down post-event window query (DEC-002) ---

    const EVENT: u64 = 1_000_000_000;
    const DEADLINE: u64 = EVENT + POST_EVENT_FRAME_WINDOW_NS;

    #[test]
    fn post_event_query_takes_the_oldest_of_two_in_window_frames() {
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, EVENT + 100_000_000));
        broker.publish_frame(frame(1, EVENT + 200_000_000));

        let selected = broker.post_event_frame(1, EVENT, DEADLINE).unwrap();
        assert_eq!(selected.ts_ns, EVENT + 100_000_000);
    }

    #[test]
    fn post_event_query_excludes_a_frame_equal_to_the_event() {
        // The pinned pre-event rule treats equality as "not later"; the
        // post-event window is open at the event, so the same frame is
        // never both the pre-event and the post-event candidate.
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, EVENT));
        assert!(broker.post_event_frame(1, EVENT, DEADLINE).is_none());
    }

    #[test]
    fn post_event_query_includes_a_frame_equal_to_the_deadline() {
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, DEADLINE));
        assert_eq!(
            broker.post_event_frame(1, EVENT, DEADLINE).unwrap().ts_ns,
            DEADLINE,
        );
    }

    #[test]
    fn post_event_query_ignores_frames_after_the_deadline() {
        // A late worker must not pick a frame taken after the window,
        // even when it is the only retained frame later than the event.
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, EVENT - 50_000_000));
        broker.publish_frame(frame(1, DEADLINE + 1));
        assert!(broker.post_event_frame(1, EVENT, DEADLINE).is_none());
    }

    #[test]
    fn post_event_query_is_per_display() {
        let mut broker = broker_with(vec![1, 2]);
        broker.publish_frame(frame(1, EVENT - 10_000_000));
        broker.publish_frame(frame(2, EVENT + 30_000_000));
        assert!(broker.post_event_frame(1, EVENT, DEADLINE).is_none());
        assert_eq!(
            broker.post_event_frame(2, EVENT, DEADLINE).unwrap().ts_ns,
            EVENT + 30_000_000,
        );
        // A display that retains nothing at all yields `None`, too.
        assert!(broker.post_event_frame(3, EVENT, DEADLINE).is_none());
    }

    #[test]
    fn post_event_query_leaves_the_pinned_click_selection_unchanged() {
        // The pre-event snapshot for the same event still pins the
        // newest frame not later than the event (AC-002 click rule).
        let mut broker = broker_with(vec![1]);
        broker.publish_frame(frame(1, EVENT - 10_000_000));
        broker.publish_frame(frame(1, EVENT + 40_000_000));
        let snapshot = broker.snapshot(EVENT);
        assert_eq!(snapshot.frame_for(1).unwrap().ts_ns, EVENT - 10_000_000);
        assert_eq!(
            broker.post_event_frame(1, EVENT, DEADLINE).unwrap().ts_ns,
            EVENT + 40_000_000,
        );
    }
}
