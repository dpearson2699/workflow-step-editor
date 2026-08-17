//! The ListenOnly CGEventTap on a dedicated CFRunLoop thread.
//!
//! The callback is constant-bounded and nonblocking: it copies the
//! event's immutable fields (type, position, button, keycode, produced
//! characters, modifier flags, timestamp) into a [`TapEvent`] and hands
//! it to the pipeline's enqueue closure. It never touches the AX API,
//! the window list, or frame bytes.
//!
//! Tap health: `kCGEventTapDisabledBy*` callbacks report immediately;
//! [`TapHandle::is_enabled`] exposes the runtime `CGEventTapIsEnabled`
//! check the pipeline's health loop polls (DEC-001/DEC-007 — no
//! silent re-enable loops).

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use core_graphics::sys::CGEventRef;
use foreign_types::ForeignType;

use crate::capture::hostclock::{host_now_ns, normalize_event_timestamp_ns, timebase};
use crate::domain::schema::{KeyInfo, Modifier, MouseButton};

extern "C" {
    fn CGEventGetTimestamp(event: CGEventRef) -> u64;
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_len: libc::c_ulong,
        actual_len: *mut libc::c_ulong,
        buffer: *mut u16,
    );
    fn CGEventTapIsEnabled(tap: CFMachPortRef) -> bool;
}

/// One copied global input event, in host-clock nanoseconds and global
/// display points.
#[derive(Debug, Clone, PartialEq)]
pub enum TapInput {
    Click { button: MouseButton },
    KeyDown { key: KeyInfo },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TapEvent {
    pub input: TapInput,
    pub x: f64,
    pub y: f64,
    pub ts_ns: u64,
}

/// `CFMachPortRef` for the health check: CFMachPort is a thread-safe
/// immutable CF object; the handle retains it for the tap's lifetime.
struct SendPort(CFMachPortRef);
// SAFETY: the port is CFRetained by the owning `CGEventTap` on the tap
// thread, which outlives this handle; `CGEventTapIsEnabled` is a
// thread-safe read.
unsafe impl Send for SendPort {}
unsafe impl Sync for SendPort {}

/// `CFRunLoop` of the tap thread; `CFRunLoop::stop` is documented as
/// callable from any thread.
struct SendRunLoop(CFRunLoop);
// SAFETY: only `stop` crosses threads, which CFRunLoop supports.
unsafe impl Send for SendRunLoop {}

/// A cloneable runtime health probe over the tap's mach port. The
/// health-monitor thread polls `CGEventTapIsEnabled` through it without
/// borrowing the [`TapHandle`].
#[derive(Clone)]
pub struct TapHealthProbe {
    port: CFMachPortRef,
}
// SAFETY: the mach port is CFRetained by the owning tap for the probe's
// lifetime (the probe is dropped before `TapHandle::stop` joins the tap
// thread); `CGEventTapIsEnabled` is a thread-safe read.
unsafe impl Send for TapHealthProbe {}
unsafe impl Sync for TapHealthProbe {}

impl TapHealthProbe {
    /// The runtime `CGEventTapIsEnabled` health check.
    pub fn is_enabled(&self) -> bool {
        // SAFETY: see the `TapHealthProbe` safety note.
        unsafe { CGEventTapIsEnabled(self.port) }
    }
}

/// Handle over the running tap thread.
pub struct TapHandle {
    port: SendPort,
    runloop: SendRunLoop,
    thread: Option<JoinHandle<()>>,
}

impl TapHandle {
    /// The runtime `CGEventTapIsEnabled` health check.
    pub fn is_enabled(&self) -> bool {
        // SAFETY: see `SendPort`.
        unsafe { CGEventTapIsEnabled(self.port.0) }
    }

    /// A cloneable health probe for the monitor thread.
    pub fn health_probe(&self) -> TapHealthProbe {
        TapHealthProbe { port: self.port.0 }
    }

    /// Stops the run loop and joins the tap thread; the tap is
    /// destroyed on its own thread. Idempotent by construction (the
    /// join handle is taken once).
    pub fn stop(mut self) {
        self.runloop.0.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts the ListenOnly session tap on a dedicated thread. `on_event`
/// must be constant-bounded and nonblocking — the pipeline's
/// snapshot-and-enqueue closure. Returns once the tap is created,
/// enabled, and verified enabled; a tap that cannot be created or
/// immediately reports disabled (Input Monitoring revoked between the
/// permission gate and here) is a start error.
pub fn start_event_tap(
    on_event: impl Fn(TapEvent) + Send + 'static,
    on_disabled: impl Fn(String) + Send + Sync + 'static,
) -> Result<TapHandle, String> {
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(SendPort, SendRunLoop), String>>();

    let thread = std::thread::Builder::new()
        .name("capture-event-tap".into())
        .spawn(move || {
            let tap_timebase = timebase();
            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::LeftMouseDown,
                    CGEventType::RightMouseDown,
                    CGEventType::OtherMouseDown,
                ],
                move |_proxy, event_type, event| {
                    match event_type {
                        CGEventType::TapDisabledByTimeout
                        | CGEventType::TapDisabledByUserInput => {
                            on_disabled(format!(
                                "event tap disabled by the system ({event_type:?})"
                            ));
                        }
                        _ => {
                            if let Some(tap_event) =
                                copy_event(event_type, event, tap_timebase)
                            {
                                on_event(tap_event);
                            }
                        }
                    }
                    CallbackResult::Keep
                },
            );
            let tap = match tap {
                Ok(tap) => tap,
                Err(()) => {
                    let _ = ready_tx.send(Err(
                        "could not create the ListenOnly event tap (Input Monitoring)".to_owned(),
                    ));
                    return;
                }
            };
            let source = match tap.mach_port().create_runloop_source(0) {
                Ok(source) => source,
                Err(()) => {
                    let _ = ready_tx
                        .send(Err("could not create the event-tap run-loop source".to_owned()));
                    return;
                }
            };
            let runloop = CFRunLoop::get_current();
            runloop.add_source(&source, unsafe { kCFRunLoopCommonModes });
            tap.enable();
            let port = tap.mach_port().as_concrete_TypeRef();
            // SAFETY: reading the enabled flag of a live tap.
            if !unsafe { CGEventTapIsEnabled(port) } {
                let _ = ready_tx.send(Err(
                    "the event tap was created but the system reports it disabled; \
                     Input Monitoring is not effective for this process"
                        .to_owned(),
                ));
                return;
            }
            if ready_tx.send(Ok((SendPort(port), SendRunLoop(runloop)))).is_err() {
                // The caller timed out and abandoned this tap: return
                // instead of entering the run loop, so the tap (and the
                // enqueue closure's job sender) cannot outlive the
                // failed start and wedge the capture worker.
                return;
            }
            CFRunLoop::run_current();
            // The tap (and its port retain) drops here, on its thread.
        })
        .map_err(|error| format!("could not spawn the event-tap thread: {error}"))?;

    match ready_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok((port, runloop))) => Ok(TapHandle {
            port,
            runloop,
            thread: Some(thread),
        }),
        Ok(Err(error)) => {
            let _ = thread.join();
            Err(error)
        }
        Err(_) => Err("the event-tap thread did not report readiness".to_owned()),
    }
}

/// Copies the immutable fields of one tap event. Returns `None` for
/// event types outside the capture set.
fn copy_event(
    event_type: CGEventType,
    event: &core_graphics::event::CGEvent,
    tap_timebase: crate::capture::hostclock::Timebase,
) -> Option<TapEvent> {
    let input = match event_type {
        CGEventType::LeftMouseDown => TapInput::Click {
            button: MouseButton::Left,
        },
        CGEventType::RightMouseDown => TapInput::Click {
            button: MouseButton::Right,
        },
        CGEventType::OtherMouseDown => TapInput::Click {
            button: MouseButton::Middle,
        },
        CGEventType::KeyDown => TapInput::KeyDown {
            key: KeyInfo {
                key_code: event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                    as u16,
                chars: produced_characters(event),
                modifiers: held_modifiers(event.get_flags()),
            },
        },
        _ => return None,
    };
    let location = event.location();
    // SAFETY: the event reference is valid for the callback's duration.
    let raw_ts = unsafe { CGEventGetTimestamp(event.as_ptr()) };
    Some(TapEvent {
        input,
        x: location.x,
        y: location.y,
        ts_ns: normalize_event_timestamp_ns(raw_ts, tap_timebase, host_now_ns()),
    })
}

/// The characters the key-down produced; empty for non-character keys.
/// Control characters and the macOS function-key private-use range
/// (U+F700..=U+F8FF: arrows, F-keys, forward delete) count as
/// non-character keys per the schema.
fn produced_characters(event: &core_graphics::event::CGEvent) -> String {
    let mut buffer = [0_u16; 8];
    let mut actual: libc::c_ulong = 0;
    // SAFETY: fixed-size out buffer; the API truncates to max_len.
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event.as_ptr(),
            buffer.len() as libc::c_ulong,
            &mut actual,
            buffer.as_mut_ptr(),
        );
    }
    let produced = String::from_utf16_lossy(&buffer[..(actual as usize).min(buffer.len())]);
    let non_character = produced.chars().any(|ch| {
        ch.is_control() || ('\u{f700}'..='\u{f8ff}').contains(&ch)
    });
    if produced.is_empty() || non_character {
        String::new()
    } else {
        produced
    }
}

/// Maps held `CGEventFlags` to the schema modifiers in the canonical
/// Fn, Ctrl, Opt, Shift, Cmd, CapsLock order.
fn held_modifiers(flags: CGEventFlags) -> Vec<Modifier> {
    let mapping = [
        (CGEventFlags::CGEventFlagSecondaryFn, Modifier::Fn),
        (CGEventFlags::CGEventFlagControl, Modifier::Control),
        (CGEventFlags::CGEventFlagAlternate, Modifier::Option),
        (CGEventFlags::CGEventFlagShift, Modifier::Shift),
        (CGEventFlags::CGEventFlagCommand, Modifier::Command),
        (CGEventFlags::CGEventFlagAlphaShift, Modifier::CapsLock),
    ];
    mapping
        .into_iter()
        .filter(|(flag, _)| flags.contains(*flag))
        .map(|(_, modifier)| modifier)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_flags_map_in_canonical_order() {
        let flags = CGEventFlags::CGEventFlagCommand
            | CGEventFlags::CGEventFlagShift
            | CGEventFlags::CGEventFlagSecondaryFn;
        assert_eq!(
            held_modifiers(flags),
            vec![Modifier::Fn, Modifier::Shift, Modifier::Command],
        );
        assert_eq!(held_modifiers(CGEventFlags::empty()), vec![]);
    }
}
