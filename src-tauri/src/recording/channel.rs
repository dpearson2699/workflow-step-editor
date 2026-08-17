//! The typed live-channel envelope the UI consumes during a recording.
//!
//! Tagged `step` items stream one per captured event, followed by exactly
//! one terminal `stopped` or `failed` item (terminal-last). A channel
//! disconnect never interrupts disk persistence: emission failures are
//! ignored by the recording worker.

use serde::{Deserialize, Serialize};

use crate::domain::schema::Step;

/// One item on the live capture channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEnvelope {
    /// One parsed step. Published only after the event line and all three
    /// screenshots are committed to disk.
    Step { step: Step },
    /// Terminal: the recording stopped and the manifest is saved.
    Stopped { workflow_id: String },
    /// Terminal: the recording fail-stopped. Committed events and shots
    /// are preserved and the manifest is saved with the committed steps.
    Failed { workflow_id: String, error: String },
}

impl LiveEnvelope {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped { .. } | Self::Failed { .. })
    }
}

/// The recording worker's outbound seam. Production wraps the Tauri IPC
/// channel; tests substitute recording or failing sinks.
pub trait StepSink: Send {
    /// Delivery is best-effort: the worker ignores errors so persistence
    /// never depends on the UI connection.
    fn emit(&self, item: LiveEnvelope) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_serializes_with_snake_case_tags() {
        let stopped = serde_json::to_value(LiveEnvelope::Stopped {
            workflow_id: "w1".into(),
        })
        .unwrap();
        assert_eq!(stopped["type"], "stopped");
        assert_eq!(stopped["workflow_id"], "w1");

        let failed = serde_json::to_value(LiveEnvelope::Failed {
            workflow_id: "w1".into(),
            error: "tap disabled".into(),
        })
        .unwrap();
        assert_eq!(failed["type"], "failed");
        assert_eq!(failed["error"], "tap disabled");
    }
}
