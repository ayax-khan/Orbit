pub mod hardware;
pub mod software;

pub use hardware::detect_available_encoders;
pub use software::PassthroughEncoder;

use crate::capture::Frame;

/// A video encoder turns a raw captured [`Frame`] into an encoded byte stream
/// (e.g. an H.264 access unit). The MVP ships a passthrough encoder; hardware
/// (NVENC/QSV/AMF) and software (x264) backends plug in behind this trait.
pub trait Encoder: Send {
    /// Encode a single frame into a self-contained payload for transmission.
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, EncodeError>;

    /// Name of the active encoder backend, for logging.
    fn name(&self) -> &'static str;
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("encoder initialization failed: {0}")]
    Init(String),
    #[error("frame encoding failed: {0}")]
    Encode(String),
}

/// Select the best available encoder. Prefers a detected hardware encoder and
/// falls back to the portable software/passthrough encoder.
pub fn create_encoder() -> Box<dyn Encoder> {
    let available = detect_available_encoders();
    if let Some(hw) = available.into_iter().next() {
        if let Ok(enc) = hardware::create_hardware_encoder(hw) {
            return enc;
        }
    }
    tracing::info!("using software/passthrough encoder");
    Box::new(PassthroughEncoder::new())
}
