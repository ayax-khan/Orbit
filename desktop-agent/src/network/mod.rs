//! Networking: WebRTC peer, backend signaling, and data-channel wiring.

pub mod datachannel;
pub mod signaling;
pub mod webrtc;

pub use signaling::SignalingClient;
pub use webrtc::WebRtcPeer;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("webrtc setup failed: {0}")]
    Setup(String),
    #[error("signaling error: {0}")]
    Signaling(String),
    #[error("data channel error: {0}")]
    DataChannel(String),
}
