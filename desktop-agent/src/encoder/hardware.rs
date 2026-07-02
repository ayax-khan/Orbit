//! Hardware encoder detection and selection.
//!
//! Per the spec, ORBIT prefers a GPU H.264 encoder (NVIDIA NVENC, Intel Quick
//! Sync, AMD AMF) and falls back to software. The v1.0 MVP does not bundle the
//! proprietary encoder SDKs, so detection currently reports no hardware
//! encoders and the pipeline uses the software fallback. Real detection plugs
//! in here (querying adapters / probing the encoder SDKs) and returns a boxed
//! [`Encoder`] implementation, keeping the selection logic unchanged.

use super::{EncodeError, Encoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants are constructed once real GPU detection lands
pub enum HardwareEncoderType {
    NvidiaNvEnc,
    IntelQuickSync,
    AmdAmf,
}

impl HardwareEncoderType {
    pub fn label(self) -> &'static str {
        match self {
            HardwareEncoderType::NvidiaNvEnc => "NVIDIA NVENC",
            HardwareEncoderType::IntelQuickSync => "Intel Quick Sync",
            HardwareEncoderType::AmdAmf => "AMD AMF",
        }
    }
}

/// Probe the system for usable hardware H.264 encoders, in priority order.
///
/// Returns an empty list on platforms/builds without a hardware encoder SDK,
/// which makes [`super::create_encoder`] fall back to software encoding.
pub fn detect_available_encoders() -> Vec<HardwareEncoderType> {
    tracing::debug!("probing for hardware H.264 encoders");
    // No hardware encoder SDK is bundled in v1.0; report none so the caller
    // uses the portable software encoder.
    Vec::new()
}

/// Instantiate a hardware encoder of the given type.
///
/// Not yet implemented in v1.0: returns an [`EncodeError::Init`] so the caller
/// gracefully falls back to the software encoder.
pub fn create_hardware_encoder(
    encoder_type: HardwareEncoderType,
) -> Result<Box<dyn Encoder>, EncodeError> {
    Err(EncodeError::Init(format!(
        "{} hardware encoding is not available in this build",
        encoder_type.label()
    )))
}
