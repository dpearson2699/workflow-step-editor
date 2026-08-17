//! TCC permission reporting and requesting for the capture pipeline.
//!
//! Ordering invariant (issue #6 decision 2, DEC-011): no Accessibility API
//! call may happen before Input Monitoring has been requested, because an
//! early Accessibility check suppresses the Input Monitoring prompt. An
//! out-of-order Accessibility request returns
//! [`PermissionStatus::BlockedByPrerequisite`] without touching the
//! Accessibility API.
//!
//! [`PermissionService`] is not internally synchronized. The Tauri command
//! layer wraps it in a mutex so permission operations are serialized and
//! concurrent commands cannot violate the request order.

#[cfg(target_os = "macos")]
pub mod macos;

use serde::{Deserialize, Serialize};

/// The three TCC permissions the capture pipeline needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    InputMonitoring,
    Accessibility,
    ScreenRecording,
}

/// Error for a permission kind name that is not one of the three kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidPermissionKind(String);

impl std::fmt::Display for InvalidPermissionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown permission kind: {}", self.0)
    }
}

impl std::error::Error for InvalidPermissionKind {}

impl std::str::FromStr for PermissionKind {
    type Err = InvalidPermissionKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "input_monitoring" => Ok(Self::InputMonitoring),
            "accessibility" => Ok(Self::Accessibility),
            "screen_recording" => Ok(Self::ScreenRecording),
            other => Err(InvalidPermissionKind(other.to_owned())),
        }
    }
}

/// User-facing status of one permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Granted,
    /// Not currently granted. Kinds whose native query is boolean
    /// (Accessibility, Screen Recording) cannot distinguish "never asked"
    /// from "declined"; both report `Denied` here.
    Denied,
    /// TCC has no record yet (first launch, before any request).
    NotRequested,
    /// Deliberately not checked or requested: the ordering prerequisite
    /// (Input Monitoring requested first) is not satisfied yet.
    BlockedByPrerequisite,
}

/// Raw answer from the operating system for one permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeStatus {
    Granted,
    Denied,
    NotDetermined,
}

impl From<NativeStatus> for PermissionStatus {
    fn from(native: NativeStatus) -> Self {
        match native {
            NativeStatus::Granted => Self::Granted,
            NativeStatus::Denied => Self::Denied,
            NativeStatus::NotDetermined => Self::NotRequested,
        }
    }
}

/// The native seam: one status query and one request path per kind.
///
/// The production implementation is [`macos::MacosPermissionSource`]; tests
/// substitute a fake at this seam.
pub trait PermissionSource {
    fn input_monitoring_status(&mut self) -> NativeStatus;
    fn request_input_monitoring(&mut self) -> NativeStatus;
    fn accessibility_status(&mut self) -> NativeStatus;
    fn request_accessibility(&mut self) -> NativeStatus;
    fn screen_recording_status(&mut self) -> NativeStatus;
    fn request_screen_recording(&mut self) -> NativeStatus;
}

/// Status of all three permissions, one field per kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PermissionReport {
    pub input_monitoring: PermissionStatus,
    pub accessibility: PermissionStatus,
    pub screen_recording: PermissionStatus,
}

/// Ordered permission aggregation over a [`PermissionSource`].
pub struct PermissionService<S> {
    source: S,
    /// True once this service has sent the native Input Monitoring request
    /// in this session. TCC state from earlier runs is derived from the
    /// native status instead.
    input_monitoring_requested: bool,
}

impl<S: PermissionSource> PermissionService<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            input_monitoring_requested: false,
        }
    }

    /// True once Input Monitoring has been requested: either by this
    /// service in this session, or in an earlier run recorded by TCC
    /// (the native status is no longer `NotDetermined`).
    fn prerequisite_satisfied(&self, input_monitoring: NativeStatus) -> bool {
        self.input_monitoring_requested || input_monitoring != NativeStatus::NotDetermined
    }

    /// Reports all three kinds. Queries Input Monitoring first; queries
    /// Accessibility only when the ordering prerequisite is satisfied.
    pub fn check_all(&mut self) -> PermissionReport {
        let input_monitoring = self.source.input_monitoring_status();
        let accessibility = if self.prerequisite_satisfied(input_monitoring) {
            self.source.accessibility_status().into()
        } else {
            PermissionStatus::BlockedByPrerequisite
        };
        PermissionReport {
            input_monitoring: input_monitoring.into(),
            accessibility,
            screen_recording: self.source.screen_recording_status().into(),
        }
    }

    /// Sends the native request for `kind` and returns the resulting
    /// status. An Accessibility request whose ordering prerequisite is not
    /// satisfied returns [`PermissionStatus::BlockedByPrerequisite`]
    /// without touching the Accessibility API (DEC-011).
    pub fn request(&mut self, kind: PermissionKind) -> PermissionStatus {
        match kind {
            PermissionKind::InputMonitoring => {
                self.input_monitoring_requested = true;
                self.source.request_input_monitoring().into()
            }
            PermissionKind::Accessibility => {
                let input_monitoring = self.source.input_monitoring_status();
                if self.prerequisite_satisfied(input_monitoring) {
                    self.source.request_accessibility().into()
                } else {
                    PermissionStatus::BlockedByPrerequisite
                }
            }
            PermissionKind::ScreenRecording => self.source.request_screen_recording().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    /// Every observable call to the native seam, in order.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Call {
        InputMonitoringStatus,
        InputMonitoringRequest,
        AccessibilityStatus,
        AccessibilityRequest,
        ScreenRecordingStatus,
        ScreenRecordingRequest,
    }

    /// Fake native seam that records every call and serves fixed statuses.
    struct FakeSource {
        calls: Vec<Call>,
        input_monitoring: NativeStatus,
        accessibility: NativeStatus,
        screen_recording: NativeStatus,
    }

    impl FakeSource {
        fn first_launch() -> Self {
            Self {
                calls: Vec::new(),
                input_monitoring: NativeStatus::NotDetermined,
                accessibility: NativeStatus::Denied,
                screen_recording: NativeStatus::Denied,
            }
        }

        fn with_input_monitoring(status: NativeStatus) -> Self {
            Self {
                input_monitoring: status,
                ..Self::first_launch()
            }
        }

        fn accessibility_calls(&self) -> Vec<Call> {
            self.calls
                .iter()
                .copied()
                .filter(|call| {
                    matches!(call, Call::AccessibilityStatus | Call::AccessibilityRequest)
                })
                .collect()
        }
    }

    impl PermissionSource for FakeSource {
        fn input_monitoring_status(&mut self) -> NativeStatus {
            self.calls.push(Call::InputMonitoringStatus);
            self.input_monitoring
        }

        fn request_input_monitoring(&mut self) -> NativeStatus {
            self.calls.push(Call::InputMonitoringRequest);
            self.input_monitoring
        }

        fn accessibility_status(&mut self) -> NativeStatus {
            self.calls.push(Call::AccessibilityStatus);
            self.accessibility
        }

        fn request_accessibility(&mut self) -> NativeStatus {
            self.calls.push(Call::AccessibilityRequest);
            self.accessibility
        }

        fn screen_recording_status(&mut self) -> NativeStatus {
            self.calls.push(Call::ScreenRecordingStatus);
            self.screen_recording
        }

        fn request_screen_recording(&mut self) -> NativeStatus {
            self.calls.push(Call::ScreenRecordingRequest);
            self.screen_recording
        }
    }

    #[test]
    fn check_queries_input_monitoring_before_accessibility() {
        let mut service =
            PermissionService::new(FakeSource::with_input_monitoring(NativeStatus::Granted));

        service.check_all();

        assert_eq!(
            service.source.calls,
            vec![
                Call::InputMonitoringStatus,
                Call::AccessibilityStatus,
                Call::ScreenRecordingStatus,
            ],
        );
    }

    #[test]
    fn first_launch_reports_not_requested_and_blocks_accessibility() {
        let mut service = PermissionService::new(FakeSource::first_launch());

        let report = service.check_all();

        assert_eq!(
            report,
            PermissionReport {
                input_monitoring: PermissionStatus::NotRequested,
                accessibility: PermissionStatus::BlockedByPrerequisite,
                screen_recording: PermissionStatus::Denied,
            },
        );
        assert_eq!(service.source.accessibility_calls(), vec![]);
    }

    #[test]
    fn out_of_order_accessibility_request_is_blocked_without_touching_ax() {
        let mut service = PermissionService::new(FakeSource::first_launch());

        let status = service.request(PermissionKind::Accessibility);

        assert_eq!(status, PermissionStatus::BlockedByPrerequisite);
        assert_eq!(service.source.accessibility_calls(), vec![]);
    }

    #[test]
    fn accessibility_unblocks_after_in_session_input_monitoring_request() {
        // TCC still reports NotDetermined while the prompt is on screen;
        // the in-session request alone must satisfy the prerequisite.
        let mut service = PermissionService::new(FakeSource::first_launch());

        service.request(PermissionKind::InputMonitoring);
        let status = service.request(PermissionKind::Accessibility);

        assert_eq!(status, PermissionStatus::Denied);
        assert_eq!(
            service.source.accessibility_calls(),
            vec![Call::AccessibilityRequest],
        );
    }

    #[test]
    fn accessibility_unblocks_when_earlier_run_already_requested_input_monitoring() {
        let mut service =
            PermissionService::new(FakeSource::with_input_monitoring(NativeStatus::Denied));

        let status = service.request(PermissionKind::Accessibility);

        assert_eq!(status, PermissionStatus::Denied);
        assert_eq!(
            service.source.accessibility_calls(),
            vec![Call::AccessibilityRequest],
        );
    }

    #[test]
    fn each_kind_uses_exactly_its_own_native_request_path() {
        let mut service =
            PermissionService::new(FakeSource::with_input_monitoring(NativeStatus::Granted));

        service.request(PermissionKind::InputMonitoring);
        service.request(PermissionKind::ScreenRecording);
        service.request(PermissionKind::Accessibility);

        let requests: Vec<Call> = service
            .source
            .calls
            .iter()
            .copied()
            .filter(|call| {
                matches!(
                    call,
                    Call::InputMonitoringRequest
                        | Call::AccessibilityRequest
                        | Call::ScreenRecordingRequest
                )
            })
            .collect();
        assert_eq!(
            requests,
            vec![
                Call::InputMonitoringRequest,
                Call::ScreenRecordingRequest,
                Call::AccessibilityRequest,
            ],
        );
    }

    #[test]
    fn invalid_kind_name_is_rejected() {
        let error = PermissionKind::from_str("full_disk_access").unwrap_err();

        assert_eq!(
            error.to_string(),
            "unknown permission kind: full_disk_access",
        );
    }

    #[test]
    fn kind_names_parse_to_their_kinds() {
        assert_eq!(
            PermissionKind::from_str("input_monitoring").unwrap(),
            PermissionKind::InputMonitoring,
        );
        assert_eq!(
            PermissionKind::from_str("accessibility").unwrap(),
            PermissionKind::Accessibility,
        );
        assert_eq!(
            PermissionKind::from_str("screen_recording").unwrap(),
            PermissionKind::ScreenRecording,
        );
    }
}
