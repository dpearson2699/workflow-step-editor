//! The ScreenCaptureKit stream backend: one continuous `SCStream` per
//! display, delivering BGRA frames into the broker.
//!
//! The output/delegate object copies each screen sample buffer's pixels
//! out on its dispatch queue and publishes an owned [`FrameData`]; a
//! stream stop-with-error reports through the failure sink. The
//! `objc2` bindings avoid the Swift runtime entirely, so the signed app
//! links only against system frameworks.

use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use dispatch2::{DispatchQueue, DispatchQueueAttr, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, AnyThread, DefinedClass};
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{
    kCVPixelFormatType_32BGRA, CVPixelBuffer, CVPixelBufferGetBaseAddress,
    CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight, CVPixelBufferGetWidth,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};
use objc2_core_media::CMTime;
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};
use objc2_screen_capture_kit::{
    SCContentFilter, SCDisplay, SCShareableContent, SCStream, SCStreamConfiguration,
    SCStreamDelegate, SCStreamOutput, SCStreamOutputType,
};

use crate::capture::broker::FrameData;
use crate::capture::geometry::DisplayGeometry;
use crate::capture::hostclock::frame_timestamp_ns;
use crate::capture::macos::displays::{enumerate_displays, shareable_content};
use crate::capture::streams::{ActiveStream, FailureSink, FrameSink, StreamBackend};

struct HandlerIvars {
    display: DisplayGeometry,
    frames: FrameSink,
    failure: FailureSink,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements; no Drop impl.
    #[unsafe(super(NSObject))]
    #[name = "WseCaptureStreamHandler"]
    #[ivars = HandlerIvars]
    struct StreamHandler;

    unsafe impl NSObjectProtocol for StreamHandler {}

    unsafe impl SCStreamOutput for StreamHandler {
        #[unsafe(method(stream:didOutputSampleBuffer:ofType:))]
        unsafe fn stream_did_output_sample_buffer_of_type(
            &self,
            _stream: &SCStream,
            sample_buffer: &CMSampleBuffer,
            kind: SCStreamOutputType,
        ) {
            if kind != SCStreamOutputType::Screen {
                return;
            }
            if let Some(frame) = copy_frame(sample_buffer, &self.ivars().display) {
                (self.ivars().frames)(frame);
            }
        }
    }

    unsafe impl SCStreamDelegate for StreamHandler {
        #[unsafe(method(stream:didStopWithError:))]
        unsafe fn stream_did_stop_with_error(&self, _stream: &SCStream, error: &NSError) {
            (self.ivars().failure)(format!(
                "screen capture stream stopped: {}",
                error.localizedDescription()
            ));
        }
    }
);

impl StreamHandler {
    fn new(display: DisplayGeometry, frames: FrameSink, failure: FailureSink) -> Retained<Self> {
        let this = Self::alloc().set_ivars(HandlerIvars {
            display,
            frames,
            failure,
        });
        unsafe { msg_send![super(this), init] }
    }
}

/// Copies one screen sample buffer's pixels into an owned frame. `None`
/// for idle frames without an image buffer or on a lock failure.
fn copy_frame(sample_buffer: &CMSampleBuffer, display: &DisplayGeometry) -> Option<FrameData> {
    let pts = unsafe { sample_buffer.presentation_time_stamp() };
    let ts_ns = frame_timestamp_ns(seconds(pts));
    let image = unsafe { sample_buffer.image_buffer() }?;
    // `CVPixelBuffer` is a type alias for `CVImageBuffer`.
    let pixel_buffer: &CVPixelBuffer = &image;
    // SAFETY: a read-only lock over a valid pixel buffer.
    if unsafe { CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) } != 0
    {
        return None;
    }
    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);
    let base = CVPixelBufferGetBaseAddress(pixel_buffer);
    let pixels = if base.is_null() || width == 0 || height == 0 {
        None
    } else {
        // SAFETY: the buffer is locked read-only for this scope; the
        // copied Vec outlives the unlock.
        Some(unsafe { std::slice::from_raw_parts(base.cast::<u8>(), bytes_per_row * height) }.to_vec())
    };
    // SAFETY: balances the lock above.
    unsafe { CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
    let pixels = pixels?;
    Some(FrameData {
        display: display.clone(),
        width_px: width as u32,
        height_px: height as u32,
        bytes_per_row,
        ts_ns,
        pixels,
    })
}

fn seconds(time: CMTime) -> f64 {
    // Avoids the deprecated free function; the value/timescale are the
    // host-clock presentation time.
    if time.timescale == 0 {
        return 0.0;
    }
    time.value as f64 / f64::from(time.timescale)
}

/// A running per-display stream: the stream plus the handler kept alive
/// for the stream's lifetime.
struct MacosStream {
    stream: Retained<SCStream>,
    _handler: Retained<StreamHandler>,
    _queue: DispatchRetained<DispatchQueue>,
}

// SAFETY: the stream, its handler, and its dispatch queue are moved from
// the pipeline's start thread onto the stream-manager control thread
// exactly once and are thereafter used only from that thread (and from
// ScreenCaptureKit's own dispatch queue, which the objects are designed
// for). No concurrent Rust-side access to these handles occurs.
unsafe impl Send for MacosStream {}

impl ActiveStream for MacosStream {
    fn stop(self: Box<Self>) {
        let (tx, rx) = mpsc::channel::<()>();
        let block = RcBlock::new(move |_error: *mut NSError| {
            let _ = tx.send(());
        });
        unsafe {
            self.stream
                .stopCaptureWithCompletionHandler(Some(&block));
        }
        // Bounded wait so a wedged stop cannot hang the manager thread.
        let _ = rx.recv_timeout(Duration::from_secs(5));
    }
}

/// The production stream backend.
pub struct MacosStreamBackend;

impl StreamBackend for MacosStreamBackend {
    fn current_displays(&mut self) -> Result<Vec<DisplayGeometry>, String> {
        enumerate_displays()
    }

    fn start_stream(
        &mut self,
        display: &DisplayGeometry,
        sink: FrameSink,
        failure: FailureSink,
    ) -> Result<Box<dyn ActiveStream>, String> {
        let sc_display = find_sc_display(display.id)?;
        let filter = unsafe {
            SCContentFilter::initWithDisplay_excludingWindows(
                SCContentFilter::alloc(),
                &sc_display,
                &NSArray::new(),
            )
        };
        let config = unsafe { SCStreamConfiguration::new() };
        unsafe {
            config.setWidth(display.width_px() as usize);
            config.setHeight(display.height_px() as usize);
            config.setPixelFormat(kCVPixelFormatType_32BGRA);
            // ~10 fps: enough to keep a fresh pre-event frame without a
            // heavy standing capture cost.
            config.setMinimumFrameInterval(CMTime::new(1, 10));
            config.setQueueDepth(5);
            config.setShowsCursor(true);
        }

        let handler = StreamHandler::new(display.clone(), sink, failure);
        let delegate = ProtocolObject::from_ref(&*handler);
        let stream = unsafe {
            SCStream::initWithFilter_configuration_delegate(
                SCStream::alloc(),
                &filter,
                &config,
                Some(delegate),
            )
        };
        let queue = DispatchQueue::new(
            &format!("com.dpearson.workflow-step-editor.capture.{}", display.id),
            DispatchQueueAttr::SERIAL,
        );
        let output = ProtocolObject::from_ref(&*handler);
        unsafe {
            stream
                .addStreamOutput_type_sampleHandlerQueue_error(
                    output,
                    SCStreamOutputType::Screen,
                    Some(&queue),
                )
                .map_err(|error| {
                    format!("could not add the capture output for display {}: {error}", display.id)
                })?;
        }

        start_capture_blocking(&stream, display.id)?;
        Ok(Box::new(MacosStream {
            stream,
            _handler: handler,
            _queue: queue,
        }))
    }
}

fn start_capture_blocking(stream: &SCStream, display_id: u32) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<Option<String>>();
    let block = RcBlock::new(move |error: *mut NSError| {
        let result = if error.is_null() {
            None
        } else {
            Some(unsafe { (*error).localizedDescription() }.to_string())
        };
        let _ = tx.send(result);
    });
    unsafe { stream.startCaptureWithCompletionHandler(Some(&block)) };
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(None) => Ok(()),
        Ok(Some(error)) => Err(format!(
            "could not start the capture stream for display {display_id}: {error}"
        )),
        Err(_) => Err(format!(
            "starting the capture stream for display {display_id} timed out"
        )),
    }
}

/// Finds the live `SCDisplay` for a display id.
fn find_sc_display(display_id: u32) -> Result<Retained<SCDisplay>, String> {
    let content: Retained<SCShareableContent> = shareable_content()?;
    unsafe { content.displays() }
        .iter()
        .find(|display| unsafe { display.displayID() } == display_id)
        .ok_or_else(|| format!("display {display_id} is no longer available"))
}
