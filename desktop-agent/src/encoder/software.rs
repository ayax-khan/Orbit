//! Portable software encoder (MVP fallback).
//!
//! The v1.0 MVP ships a raw-frame passthrough: it prepends a small header
//! (dimensions) and forwards the BGRA pixel buffer over the data channel,
//! which the browser client renders onto a canvas. This keeps the full
//! capture -> encode -> transport -> render pipeline working on every
//! platform without a hardware encoder. A real x264/OpenH264 backend plugs in
//! behind the same [`Encoder`] trait without touching the rest of the agent.

use super::{EncodeError, Encoder};
use crate::capture::Frame;

/// Magic bytes identifying a raw ORBIT frame payload ("ORB1").
pub const RAW_FRAME_MAGIC: &[u8; 4] = b"ORB1";

pub struct PassthroughEncoder {
    frame_count: u64,
}

impl PassthroughEncoder {
    pub fn new() -> Self {
        Self { frame_count: 0 }
    }
}

impl Default for PassthroughEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for PassthroughEncoder {
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, EncodeError> {
        if frame.data.len() < frame.expected_len() {
            return Err(EncodeError::Encode(format!(
                "frame buffer too small: got {}, expected {}",
                frame.data.len(),
                frame.expected_len()
            )));
        }

        // Header: magic(4) + width(4 LE) + height(4 LE) + seq(8 LE) then pixels.
        let mut out = Vec::with_capacity(20 + frame.data.len());
        out.extend_from_slice(RAW_FRAME_MAGIC);
        out.extend_from_slice(&frame.width.to_le_bytes());
        out.extend_from_slice(&frame.height.to_le_bytes());
        out.extend_from_slice(&self.frame_count.to_le_bytes());
        out.extend_from_slice(&frame.data);

        self.frame_count = self.frame_count.wrapping_add(1);
        Ok(out)
    }

    fn name(&self) -> &'static str {
        "software-passthrough"
    }
}
