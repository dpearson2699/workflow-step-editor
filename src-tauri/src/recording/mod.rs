//! Platform-independent recording orchestration: the persistence seam, the
//! capture-pipeline boundary, the live-channel envelope, and the recording
//! coordinator.

pub mod channel;
pub mod clock;
pub mod coordinator;
pub mod fake_pipeline;
pub mod pipeline;
pub mod store;

#[cfg(test)]
pub(crate) mod testutil;
