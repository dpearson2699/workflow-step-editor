//! The injected wall-clock seam. Production wires [`SystemClock`]; tests
//! use a fixed clock so default names, folder names, and manifest
//! timestamps are deterministic.

use chrono::{DateTime, SecondsFormat, Utc};

/// One injected wall-clock source.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The production clock: reads the ambient system time on every call.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Event timestamp format: RFC 3339 UTC with millisecond precision
/// (`2026-08-16T22:31:05.123Z`).
pub fn event_timestamp(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Manifest `created_at` format: RFC 3339 UTC with second precision
/// (`2026-08-16T22:31:00Z`).
pub fn manifest_timestamp(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Readable workflow-folder prefix (`2026-08-16-223105`).
pub fn folder_prefix(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%d-%H%M%S").to_string()
}

/// The default workflow name when the caller passes none
/// (`2026-08-16 22:31:05`, issue #7 decision 5).
pub fn default_workflow_name(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn formats_match_the_schema_examples() {
        let instant = Utc.with_ymd_and_hms(2026, 8, 16, 22, 31, 5).unwrap()
            + chrono::Duration::milliseconds(123);
        assert_eq!(event_timestamp(instant), "2026-08-16T22:31:05.123Z");
        assert_eq!(manifest_timestamp(instant), "2026-08-16T22:31:05Z");
        assert_eq!(folder_prefix(instant), "2026-08-16-223105");
        assert_eq!(default_workflow_name(instant), "2026-08-16 22:31:05");
    }
}
