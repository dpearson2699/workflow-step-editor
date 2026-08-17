//! One monotonic host clock for event and frame timestamps.
//!
//! The broker compares event timestamps against frame timestamps, so
//! both must live in one clock domain: nanoseconds on the mach host
//! clock (the monotonic since-boot clock).
//!
//! - `CGEventGetTimestamp` is documented as nanoseconds of the same
//!   since-boot clock. Because that unit cannot be verified without a
//!   live Input Monitoring grant, [`normalize_event_timestamp_ns`]
//!   defensively also considers the raw-mach-tick interpretation and
//!   deterministically picks whichever lands closer to "now".
//! - ScreenCaptureKit frame presentation timestamps are `CMTime` values
//!   on the host clock; [`frame_timestamp_ns`] converts their seconds.

/// The mach timebase ratio (`mach_timebase_info`): one mach tick equals
/// `numer / denom` nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timebase {
    pub numer: u32,
    pub denom: u32,
}

impl Timebase {
    pub fn ticks_to_ns(&self, ticks: u64) -> u64 {
        (u128::from(ticks) * u128::from(self.numer) / u128::from(self.denom.max(1))) as u64
    }
}

#[cfg(target_os = "macos")]
mod ffi {
    #[repr(C)]
    pub struct MachTimebaseInfo {
        pub numer: u32,
        pub denom: u32,
    }
    extern "C" {
        pub fn mach_absolute_time() -> u64;
        pub fn mach_timebase_info(info: *mut MachTimebaseInfo) -> libc::c_int;
    }
}

/// The process-wide mach timebase.
#[cfg(target_os = "macos")]
pub fn timebase() -> Timebase {
    let mut info = ffi::MachTimebaseInfo { numer: 0, denom: 0 };
    // SAFETY: plain out-parameter call.
    unsafe { ffi::mach_timebase_info(&mut info) };
    Timebase {
        numer: info.numer.max(1),
        denom: info.denom.max(1),
    }
}

/// Now, in host-clock nanoseconds since boot.
#[cfg(target_os = "macos")]
pub fn host_now_ns() -> u64 {
    // SAFETY: no arguments, returns a scalar.
    let ticks = unsafe { ffi::mach_absolute_time() };
    timebase().ticks_to_ns(ticks)
}

/// Normalizes a raw `CGEventGetTimestamp` value into host-clock
/// nanoseconds. The documented unit is nanoseconds; when interpreting
/// the raw value as mach ticks lands strictly closer to `now_ns`, that
/// interpretation wins. On identity timebases (Intel: 1/1) both
/// interpretations are equal, so the rule is a no-op there.
pub fn normalize_event_timestamp_ns(raw: u64, timebase: Timebase, now_ns: u64) -> u64 {
    let as_ns = raw;
    let as_ticks = timebase.ticks_to_ns(raw);
    if as_ns.abs_diff(now_ns) <= as_ticks.abs_diff(now_ns) {
        as_ns
    } else {
        as_ticks
    }
}

/// Converts a frame presentation timestamp (host-clock seconds) into
/// nanoseconds. Negative or non-finite values clamp to zero.
pub fn frame_timestamp_ns(pts_seconds: f64) -> u64 {
    if !pts_seconds.is_finite() || pts_seconds <= 0.0 {
        return 0;
    }
    (pts_seconds * 1_000_000_000.0).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Apple-silicon timebase: 125/3 (24 MHz tick).
    const APPLE_SILICON: Timebase = Timebase {
        numer: 125,
        denom: 3,
    };
    const IDENTITY: Timebase = Timebase { numer: 1, denom: 1 };

    #[test]
    fn nanosecond_timestamps_pass_through() {
        let now_ns = 1_000_000_000_000;
        // A raw value that already looks like nanoseconds near now.
        let raw = now_ns - 5_000_000;
        assert_eq!(
            normalize_event_timestamp_ns(raw, APPLE_SILICON, now_ns),
            raw,
        );
        assert_eq!(normalize_event_timestamp_ns(raw, IDENTITY, now_ns), raw);
    }

    #[test]
    fn tick_shaped_timestamps_convert_through_the_timebase() {
        let now_ns = 1_000_000_000_000;
        // A raw value in mach ticks: now / (125/3) ticks.
        let raw_ticks = now_ns * 3 / 125;
        let normalized = normalize_event_timestamp_ns(raw_ticks, APPLE_SILICON, now_ns);
        assert_eq!(normalized, APPLE_SILICON.ticks_to_ns(raw_ticks));
        assert!(normalized.abs_diff(now_ns) < 1_000);
    }

    #[test]
    fn frame_seconds_convert_to_nanoseconds() {
        assert_eq!(frame_timestamp_ns(1.5), 1_500_000_000);
        assert_eq!(frame_timestamp_ns(0.0), 0);
        assert_eq!(frame_timestamp_ns(-2.0), 0);
        assert_eq!(frame_timestamp_ns(f64::NAN), 0);
    }
}
