//! Window and Accessibility (AX) metadata resolution.
//!
//! Clicks (DEC-001/DEC-011): a `CGWindowListCopyWindowInfo` hit test at
//! the click point resolves the window, and the system-wide
//! `AXUIElementCopyElementAtPosition` resolves the element.
//!
//! Key-downs (DEC-008): the system-wide focused application's focused
//! window resolves the window, and `AXFocusedUIElement` resolves the
//! element; the frontmost application name comes from the focused
//! application element.
//!
//! Every AX call has a short messaging timeout so an unresponsive app
//! cannot stall the single capture worker; a timeout degrades to the
//! fixed-size fallback rather than blocking.

use std::ffi::c_void;
use std::ptr;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedApplicationAttribute, kAXFocusedUIElementAttribute,
    kAXFocusedWindowAttribute, kAXPositionAttribute, kAXRoleAttribute, kAXSizeAttribute,
    kAXTitleAttribute, kAXValueTypeCGPoint, kAXValueTypeCGSize, AXUIElementCopyAttributeValue,
    AXUIElementCopyElementAtPosition, AXUIElementCreateSystemWide, AXUIElementGetPid,
    AXUIElementRef, AXUIElementSetMessagingTimeout, AXValueGetValue, AXValueRef,
};
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation_sys::base::{CFRelease, CFTypeRef};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::window::{
    copy_window_info, kCGWindowBounds, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
    kCGWindowListOptionOnScreenOnly, kCGWindowName, kCGWindowOwnerName, kCGWindowOwnerPID,
};

use crate::capture::geometry::{PointPt, RectPt};
use crate::capture::packets::{ResolvedElement, ResolvedMetadata, ResolvedWindow};
use crate::capture::resolver::MetadataResolver;

/// The AX messaging timeout: bounded so one unresponsive application
/// cannot stall the ordered capture worker.
const AX_MESSAGING_TIMEOUT_SECS: f32 = 0.25;
/// `kCGNullWindowID`.
const NULL_WINDOW_ID: u32 = 0;
/// Normal application window level; overlays (menu bar, dock, cursor)
/// live at other layers and are skipped for the hit test.
const NORMAL_WINDOW_LAYER: i64 = 0;

/// The production resolver. Holds the retained system-wide AX element
/// with a bounded messaging timeout.
pub struct MacosResolver {
    system_wide: AxElement,
}

impl Default for MacosResolver {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: the resolver is created on, and only ever used by, the single
// capture worker thread; its retained AX element never crosses threads
// concurrently. `MetadataResolver` requires `Send` so the coordinator
// can move the boxed resolver onto that worker thread once.
unsafe impl Send for MacosResolver {}

impl MacosResolver {
    pub fn new() -> Self {
        // SAFETY: creates the system-wide element (never null).
        let raw = unsafe { AXUIElementCreateSystemWide() };
        // SAFETY: raw is a valid retained AX element.
        unsafe { AXUIElementSetMessagingTimeout(raw, AX_MESSAGING_TIMEOUT_SECS) };
        Self {
            system_wide: AxElement(raw),
        }
    }
}

impl MetadataResolver for MacosResolver {
    fn resolve_click(&mut self, point: PointPt) -> ResolvedMetadata {
        let window = window_at_point(point);
        let element = ax_element_at_point(&self.system_wide, point).and_then(element_metadata);
        let frontmost_app = focused_application_name(&self.system_wide)
            .or_else(|| window.as_ref().map(|window| window.app.clone()));
        ResolvedMetadata {
            window,
            element,
            frontmost_app,
        }
    }

    fn resolve_key_down(&mut self) -> ResolvedMetadata {
        let focused_app = element_attribute(&self.system_wide, kAXFocusedApplicationAttribute);
        let frontmost_app = focused_app.as_ref().and_then(|app| string_attribute(app, kAXTitleAttribute));

        let window = focused_app
            .as_ref()
            .and_then(|app| element_attribute(app, kAXFocusedWindowAttribute))
            .and_then(|window| window_metadata(&window, frontmost_app.as_deref()));

        let element = element_attribute(&self.system_wide, kAXFocusedUIElementAttribute)
            .and_then(element_metadata);

        ResolvedMetadata {
            window,
            element,
            frontmost_app,
        }
    }
}

/// A retained `AXUIElementRef`, released on drop.
struct AxElement(AXUIElementRef);

impl Drop for AxElement {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the element was retained under the create rule.
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

/// Copies an attribute value as a generic `CFType`.
fn copy_attribute(element: &AxElement, attribute: &str) -> Option<CFType> {
    let key = CFString::new(attribute);
    let mut value: CFTypeRef = ptr::null();
    // SAFETY: valid element and attribute string; out-parameter.
    let status = unsafe {
        AXUIElementCopyAttributeValue(element.0, key.as_concrete_TypeRef(), &mut value)
    };
    if status == kAXErrorSuccess && !value.is_null() {
        // SAFETY: value is retained under the create rule.
        Some(unsafe { CFType::wrap_under_create_rule(value) })
    } else {
        None
    }
}

/// A string-valued attribute.
fn string_attribute(element: &AxElement, attribute: &str) -> Option<String> {
    copy_attribute(element, attribute)?
        .downcast::<CFString>()
        .map(|string| string.to_string())
        .filter(|string| !string.trim().is_empty())
}

/// An element-valued attribute (a nested `AXUIElementRef`).
fn element_attribute(element: &AxElement, attribute: &str) -> Option<AxElement> {
    let value = copy_attribute(element, attribute)?;
    // The value is itself an AXUIElementRef; take ownership of the
    // create-rule retain by leaking the CFType wrapper.
    let raw = value.as_CFTypeRef() as AXUIElementRef;
    std::mem::forget(value);
    Some(AxElement(raw))
}

/// A point-valued AX attribute (`AXPosition`).
fn point_attribute(element: &AxElement, attribute: &str) -> Option<CGPoint> {
    let value = copy_attribute(element, attribute)?;
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let mut point = CGPoint::new(0.0, 0.0);
    // SAFETY: the buffer matches the requested CGPoint type.
    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGPoint,
            (&mut point as *mut CGPoint).cast::<c_void>(),
        )
    };
    ok.then_some(point)
}

/// A size-valued AX attribute (`AXSize`).
fn size_attribute(element: &AxElement, attribute: &str) -> Option<CGSize> {
    let value = copy_attribute(element, attribute)?;
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let mut size = CGSize::new(0.0, 0.0);
    // SAFETY: the buffer matches the requested CGSize type.
    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGSize,
            (&mut size as *mut CGSize).cast::<c_void>(),
        )
    };
    ok.then_some(size)
}

/// The element's global point frame from `AXPosition` + `AXSize`.
fn element_frame(element: &AxElement) -> Option<RectPt> {
    let position = point_attribute(element, kAXPositionAttribute)?;
    let size = size_attribute(element, kAXSizeAttribute)?;
    Some(RectPt::new(position.x, position.y, size.width, size.height))
}

/// The element's pid.
fn element_pid(element: &AxElement) -> Option<i32> {
    let mut pid: i32 = 0;
    // SAFETY: valid element; out-parameter.
    let status = unsafe { AXUIElementGetPid(element.0, &mut pid) };
    (status == kAXErrorSuccess).then_some(pid)
}

/// Builds the resolved element from an AX element: role, title, frame.
/// Without a frame there is nothing to crop, so the caller falls back.
fn element_metadata(element: AxElement) -> Option<ResolvedElement> {
    let frame_pt = element_frame(&element)?;
    Some(ResolvedElement {
        role: string_attribute(&element, kAXRoleAttribute),
        title: string_attribute(&element, kAXTitleAttribute),
        frame_pt,
    })
}

/// The system-wide element at a point.
fn ax_element_at_point(system_wide: &AxElement, point: PointPt) -> Option<AxElement> {
    let mut element: AXUIElementRef = ptr::null_mut();
    // SAFETY: valid system-wide element; out-parameter.
    let status = unsafe {
        AXUIElementCopyElementAtPosition(
            system_wide.0,
            point.x as f32,
            point.y as f32,
            &mut element,
        )
    };
    (status == kAXErrorSuccess && !element.is_null()).then_some(AxElement(element))
}

/// The focused application's name.
fn focused_application_name(system_wide: &AxElement) -> Option<String> {
    let app = element_attribute(system_wide, kAXFocusedApplicationAttribute)?;
    string_attribute(&app, kAXTitleAttribute)
}

/// Builds a resolved window from an AX window element (key-down path).
fn window_metadata(window: &AxElement, app_name: Option<&str>) -> Option<ResolvedWindow> {
    let bounds_pt = element_frame(window)?;
    let title = string_attribute(window, kAXTitleAttribute).unwrap_or_default();
    let pid = element_pid(window).unwrap_or(0);
    Some(ResolvedWindow {
        app: app_name.unwrap_or("Unknown").to_owned(),
        title,
        pid,
        bounds_pt,
    })
}

/// The topmost normal window whose bounds contain the click point
/// (`CGWindowListCopyWindowInfo`, front-to-back order).
fn window_at_point(point: PointPt) -> Option<ResolvedWindow> {
    let info = copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        NULL_WINDOW_ID,
    )?;
    // The array is ordered front-to-back; the first containing window
    // at the normal window layer is the hit.
    for index in 0..info.len() {
        let Some(item) = info.get(index) else {
            continue;
        };
        let dict = unsafe {
            CFDictionary::<CFString, CFType>::wrap_under_get_rule((*item).cast())
        };
        if window_layer(&dict) != NORMAL_WINDOW_LAYER {
            continue;
        }
        let Some(bounds) = window_bounds(&dict) else {
            continue;
        };
        let bounds_pt = RectPt::new(
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        );
        if !bounds_pt.contains(point) {
            continue;
        }
        return Some(ResolvedWindow {
            app: dict_string(&dict, unsafe { kCGWindowOwnerName })
                .unwrap_or_else(|| "Unknown".to_owned()),
            title: dict_string(&dict, unsafe { kCGWindowName }).unwrap_or_default(),
            pid: dict_i64(&dict, unsafe { kCGWindowOwnerPID }).unwrap_or(0) as i32,
            bounds_pt,
        });
    }
    None
}

fn window_layer(dict: &CFDictionary<CFString, CFType>) -> i64 {
    dict_i64(dict, unsafe { kCGWindowLayer }).unwrap_or(i64::MIN)
}

fn window_bounds(dict: &CFDictionary<CFString, CFType>) -> Option<CGRect> {
    let key = unsafe { CFString::wrap_under_get_rule(kCGWindowBounds) };
    let value = dict.find(&key)?;
    // The bounds value is a `CGRect` dictionary representation; only the
    // untyped `CFDictionary` implements `ConcreteCFType`.
    let bounds_dict = value.downcast::<CFDictionary>()?;
    CGRect::from_dict_representation(&bounds_dict)
}

fn dict_string(dict: &CFDictionary<CFString, CFType>, key: CFStringRef) -> Option<String> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    let value = dict.find(&key)?;
    value
        .downcast::<CFString>()
        .map(|string| string.to_string())
        .filter(|string| !string.trim().is_empty())
}

fn dict_i64(dict: &CFDictionary<CFString, CFType>, key: CFStringRef) -> Option<i64> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    let value = dict.find(&key)?;
    value.downcast::<CFNumber>().and_then(|number| number.to_i64())
}
