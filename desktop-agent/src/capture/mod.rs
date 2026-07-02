pub mod frame;
pub mod fps_controller;

#[cfg(windows)]
pub mod dxgi;

pub use frame::Frame;
pub use fps_controller::FpsController;

/// Common interface implemented by every screen capture backend. This lets the
/// rest of the agent stay backend-agnostic (DXGI on Windows, a portable
/// fallback elsewhere), matching the spec's capture-pipeline design.
pub trait ScreenCapturer: Send {
    /// Acquire the next frame. Returns `Ok(None)` if no new frame is available
    /// yet (e.g. the desktop has not changed), and `Err` on a fatal error.
    fn next_frame(&mut self) -> Result<Option<Frame>, CaptureError>;

    /// Human-readable name of the active backend, for logging.
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("capture backend unavailable: {0}")]
    Unavailable(String),
    #[error("frame acquisition failed: {0}")]
    Acquire(String),
}

/// Create the best available capturer for the current platform. On Windows we
/// try DXGI (GPU-accelerated Desktop Duplication) and fall back to a portable
/// generator if initialization fails; on other platforms only the fallback is
/// available (useful for development and CI).
pub fn create_capturer(width: u32, height: u32) -> Box<dyn ScreenCapturer> {
    #[cfg(windows)]
    {
        match dxgi::DxgiCapturer::new() {
            Ok(cap) => {
                tracing::info!("using DXGI Desktop Duplication capturer");
                return Box::new(cap);
            }
            Err(e) => {
                tracing::warn!("DXGI unavailable ({e}); using fallback capturer");
            }
        }
    }

    Box::new(FallbackCapturer::new(width, height))
}

/// A portable capturer that produces blank frames. It keeps the pipeline fully
/// functional on non-Windows platforms and when no capture backend initializes,
/// so the WebRTC/data-channel path can still be exercised end to end.
pub struct FallbackCapturer {
    width: u32,
    height: u32,
}

impl FallbackCapturer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl ScreenCapturer for FallbackCapturer {
    fn next_frame(&mut self) -> Result<Option<Frame>, CaptureError> {
        let len = (self.width as usize) * (self.height as usize) * 4;
        Ok(Some(Frame::new(self.width, self.height, vec![0u8; len])))
    }

    fn backend_name(&self) -> &'static str {
        "fallback"
    }
}
