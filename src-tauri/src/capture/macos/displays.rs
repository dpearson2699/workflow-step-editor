//! Display enumeration and reconfiguration observation.
//!
//! Enumeration uses `SCShareableContent` for the point frames and one
//! `SCContentFilter` per display for the point-to-pixel scale, ordering
//! the main display first (the pipeline's `None`-selection fallback).
//! Reconfiguration is observed on a dedicated CFRunLoop thread through
//! `CGDisplayRegisterReconfigurationCallback`; the settled-state
//! callback signals the stream manager to restart the stream set.

use std::ffi::c_void;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use block2::RcBlock;
use core_foundation::runloop::CFRunLoop;
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_foundation::{NSArray, NSError};
use objc2_screen_capture_kit::{SCContentFilter, SCDisplay, SCShareableContent};

use crate::capture::geometry::{DisplayGeometry, RectPt};

extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayRegisterReconfigurationCallback(
        callback: ReconfigCallback,
        user_info: *mut c_void,
    ) -> i32;
    fn CGDisplayRemoveReconfigurationCallback(
        callback: ReconfigCallback,
        user_info: *mut c_void,
    ) -> i32;
}

type ReconfigCallback = unsafe extern "C" fn(display: u32, flags: u32, user_info: *mut c_void);

/// `kCGDisplayBeginConfigurationFlag`: the "about to change" phase. We
/// signal only on the settled phase.
const BEGIN_CONFIGURATION_FLAG: u32 = 1;

/// Fetches the current display set synchronously, main display first.
pub fn enumerate_displays() -> Result<Vec<DisplayGeometry>, String> {
    let content = shareable_content()?;
    let displays = unsafe { content.displays() };
    let main_id = unsafe { CGMainDisplayID() };

    let mut geometries: Vec<DisplayGeometry> = displays
        .iter()
        .map(|display| display_geometry(&display))
        .collect();
    if geometries.is_empty() {
        return Err("ScreenCaptureKit reported no active displays".to_owned());
    }
    // Main display first; the pipeline uses `displays.first()` as its
    // null-window key-down fallback.
    geometries.sort_by_key(|geometry| geometry.id != main_id);
    Ok(geometries)
}

fn display_geometry(display: &SCDisplay) -> DisplayGeometry {
    let frame = unsafe { display.frame() };
    let filter = unsafe {
        SCContentFilter::initWithDisplay_excludingWindows(
            SCContentFilter::alloc(),
            display,
            &NSArray::new(),
        )
    };
    let scale = f64::from(unsafe { filter.pointPixelScale() }).max(1.0);
    DisplayGeometry {
        id: unsafe { display.displayID() },
        frame_pt: RectPt::new(
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            frame.size.height,
        ),
        scale,
    }
}

/// Synchronous `SCShareableContent` fetch behind the async completion
/// handler.
pub(crate) fn shareable_content() -> Result<Retained<SCShareableContent>, String> {
    let (tx, rx) = mpsc::channel::<Result<Retained<SCShareableContent>, String>>();
    let block = RcBlock::new(move |content: *mut SCShareableContent, error: *mut NSError| {
        let result = if content.is_null() {
            let message = if error.is_null() {
                "screen capture content is unavailable".to_owned()
            } else {
                unsafe { (*error).localizedDescription() }.to_string()
            };
            Err(message)
        } else {
            match unsafe { Retained::retain(content) } {
                Some(content) => Ok(content),
                None => Err("screen capture content was null".to_owned()),
            }
        };
        let _ = tx.send(result);
    });
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|_| "timed out fetching screen capture content".to_owned())?
}

/// Trampoline state: the boxed signal closure, reached through the
/// callback's `user_info` pointer.
struct ObserverState {
    signal: Box<dyn Fn() + Send + Sync>,
}

unsafe extern "C" fn reconfig_trampoline(_display: u32, flags: u32, user_info: *mut c_void) {
    // Signal only on the settled phase, not the "about to change" one.
    if flags & BEGIN_CONFIGURATION_FLAG != 0 {
        return;
    }
    if user_info.is_null() {
        return;
    }
    let state = unsafe { &*(user_info as *const ObserverState) };
    (state.signal)();
}

/// Owns a dedicated CFRunLoop thread that observes display
/// reconfiguration and forwards the settled-state signal.
pub struct DisplayReconfigurationObserver {
    runloop: SendRunLoop,
    thread: Option<JoinHandle<()>>,
    // Kept alive for the callback's `user_info`; freed after the
    // callback is removed.
    state: *mut ObserverState,
}

/// `CFRunLoop` handle crossing to the drop thread; only `stop` is
/// called cross-thread, which CFRunLoop supports.
struct SendRunLoop(CFRunLoop);
// SAFETY: only `stop` is used across threads.
unsafe impl Send for SendRunLoop {}

impl DisplayReconfigurationObserver {
    /// Registers the reconfiguration callback on a fresh run-loop
    /// thread. `signal` runs on that thread and must be nonblocking
    /// (it only sends the stream manager's restart command).
    pub fn start(signal: impl Fn() + Send + Sync + 'static) -> Result<Self, String> {
        let state = Box::into_raw(Box::new(ObserverState {
            signal: Box::new(signal),
        }));
        let (ready_tx, ready_rx) = mpsc::channel::<Result<SendRunLoop, String>>();
        let state_addr = state as usize;
        let thread = std::thread::Builder::new()
            .name("capture-display-observer".into())
            .spawn(move || {
                let user_info = state_addr as *mut c_void;
                // SAFETY: registering with a live run loop on this
                // thread; `user_info` outlives the registration.
                let status = unsafe {
                    CGDisplayRegisterReconfigurationCallback(reconfig_trampoline, user_info)
                };
                if status != 0 {
                    let _ = ready_tx.send(Err(format!(
                        "could not register the display reconfiguration callback (error {status})"
                    )));
                    return;
                }
                let runloop = CFRunLoop::get_current();
                let _ = ready_tx.send(Ok(SendRunLoop(runloop)));
                CFRunLoop::run_current();
                // SAFETY: symmetric removal with the same fn + user_info.
                unsafe {
                    CGDisplayRemoveReconfigurationCallback(reconfig_trampoline, user_info);
                }
            })
            .map_err(|error| {
                // Reclaim the leaked state on spawn failure.
                unsafe { drop(Box::from_raw(state)) };
                format!("could not spawn the display-observer thread: {error}")
            })?;

        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(runloop)) => Ok(Self {
                runloop,
                thread: Some(thread),
                state,
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                unsafe { drop(Box::from_raw(state)) };
                Err(error)
            }
            Err(_) => Err("the display observer did not report readiness".to_owned()),
        }
    }
}

impl Drop for DisplayReconfigurationObserver {
    fn drop(&mut self) {
        self.runloop.0.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        // The callback is removed on the observer thread before it
        // exits; the state box is now unreachable and can be freed.
        if !self.state.is_null() {
            unsafe { drop(Box::from_raw(self.state)) };
            self.state = std::ptr::null_mut();
        }
    }
}

/// The observer holds a raw pointer that is only dereferenced on its
/// own thread and freed after that thread joins.
unsafe impl Send for DisplayReconfigurationObserver {}

/// Wraps a plain reconfiguration signal into an `Arc` the stream
/// manager can share.
pub type ReconfigureSignal = Arc<dyn Fn() + Send + Sync>;
