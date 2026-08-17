//! Native macOS TCC permission source.
//!
//! One status query and one request path per kind:
//! - Input Monitoring: `IOHIDCheckAccess` / `IOHIDRequestAccess`
//!   (`kIOHIDRequestTypeListenEvent`, the ListenOnly CGEventTap access).
//! - Accessibility: `AXIsProcessTrusted` /
//!   `AXIsProcessTrustedWithOptions(kAXTrustedCheckOptionPrompt)`.
//! - Screen Recording: `CGPreflightScreenCaptureAccess` /
//!   `CGRequestScreenCaptureAccess`.
//!
//! The ordering invariant (no Accessibility call before the Input
//! Monitoring request) lives in [`super::PermissionService`], not here.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};

use super::{NativeStatus, PermissionSource};

/// `kIOHIDRequestTypeListenEvent` — Input Monitoring access.
const IOHID_REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
/// `kIOHIDAccessTypeGranted`.
const IOHID_ACCESS_TYPE_GRANTED: u32 = 0;
/// `kIOHIDAccessTypeDenied`.
const IOHID_ACCESS_TYPE_DENIED: u32 = 1;

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDCheckAccess(request_type: u32) -> u32;
    fn IOHIDRequestAccess(request_type: u32) -> bool;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Returns a `Boolean` (`unsigned char`); nonzero means trusted.
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// Production [`PermissionSource`] backed by the macOS TCC APIs.
pub struct MacosPermissionSource;

impl PermissionSource for MacosPermissionSource {
    fn input_monitoring_status(&mut self) -> NativeStatus {
        match unsafe { IOHIDCheckAccess(IOHID_REQUEST_TYPE_LISTEN_EVENT) } {
            IOHID_ACCESS_TYPE_GRANTED => NativeStatus::Granted,
            IOHID_ACCESS_TYPE_DENIED => NativeStatus::Denied,
            // kIOHIDAccessTypeUnknown: TCC has no record yet.
            _ => NativeStatus::NotDetermined,
        }
    }

    fn request_input_monitoring(&mut self) -> NativeStatus {
        if unsafe { IOHIDRequestAccess(IOHID_REQUEST_TYPE_LISTEN_EVENT) } {
            NativeStatus::Granted
        } else {
            // The prompt may still be on screen; report the settled TCC
            // answer (Denied, or NotDetermined while undecided).
            self.input_monitoring_status()
        }
    }

    fn accessibility_status(&mut self) -> NativeStatus {
        // The Accessibility API has no not-determined query; "not trusted"
        // reports as Denied.
        if unsafe { AXIsProcessTrusted() } != 0 {
            NativeStatus::Granted
        } else {
            NativeStatus::Denied
        }
    }

    fn request_accessibility(&mut self) -> NativeStatus {
        let trusted = unsafe {
            let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
            let options = CFDictionary::from_CFType_pairs(&[(
                key.as_CFType(),
                CFBoolean::true_value().as_CFType(),
            )]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) != 0
        };
        if trusted {
            NativeStatus::Granted
        } else {
            // The system prompt points the user at System Settings; trust
            // stays false until the user toggles it there.
            NativeStatus::Denied
        }
    }

    fn screen_recording_status(&mut self) -> NativeStatus {
        if unsafe { CGPreflightScreenCaptureAccess() } {
            NativeStatus::Granted
        } else {
            NativeStatus::Denied
        }
    }

    fn request_screen_recording(&mut self) -> NativeStatus {
        if unsafe { CGRequestScreenCaptureAccess() } {
            NativeStatus::Granted
        } else {
            NativeStatus::Denied
        }
    }
}
