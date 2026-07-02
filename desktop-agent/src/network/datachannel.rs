//! Wires a WebRTC data channel to the agent's I/O:
//! - inbound messages are decoded as [`InputEvent`]s and applied to the host,
//! - an outbound task captures the screen, encodes it, and streams frames.

use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::Mutex;
use webrtc::data_channel::RTCDataChannel;

use crate::capture::{self, FpsController, ScreenCapturer};
use crate::config::AgentConfig;
use crate::encoder::{self, Encoder};
use crate::input::InputProcessor;

use super::NetworkError;

/// Attach input handling and screen streaming to a data channel.
///
/// Inbound text/binary messages are treated as JSON input events. When the
/// channel opens, a background task starts pushing encoded frames to the peer.
pub fn attach(data_channel: Arc<RTCDataChannel>, config: AgentConfig) {
    // Shared input processor; created lazily so a failure to init the input
    // backend does not prevent the (view-only) stream from working.
    let input: Arc<Mutex<Option<InputProcessor>>> = match InputProcessor::new() {
        Ok(p) => Arc::new(Mutex::new(Some(p))),
        Err(e) => {
            tracing::warn!("input backend unavailable ({e}); running view-only");
            Arc::new(Mutex::new(None))
        }
    };

    // Inbound: apply remote input events.
    let input_for_msg = input.clone();
    data_channel.on_message(Box::new(move |msg| {
        let input = input_for_msg.clone();
        Box::pin(async move {
            let text = String::from_utf8_lossy(&msg.data).to_string();
            let mut guard = input.lock().await;
            if let Some(processor) = guard.as_mut() {
                if let Err(e) = processor.process_json(&text) {
                    tracing::debug!("failed to apply input event: {e}");
                }
            }
        })
    }));

    // Outbound: on open, start the capture/encode/send loop.
    let dc_for_open = data_channel.clone();
    data_channel.on_open(Box::new(move || {
        let dc = dc_for_open.clone();
        let config = config.clone();
        Box::pin(async move {
            tracing::info!("data channel open; starting screen stream");
            tokio::spawn(async move {
                if let Err(e) = stream_frames(dc, config).await {
                    tracing::error!("screen stream ended: {e}");
                }
            });
        })
    }));
}

/// Capture -> encode -> send loop, rate-limited by the adaptive FPS controller.
async fn stream_frames(
    data_channel: Arc<RTCDataChannel>,
    config: AgentConfig,
) -> Result<(), NetworkError> {
    use webrtc::data_channel::data_channel_state::RTCDataChannelState;

    let mut capturer: Box<dyn ScreenCapturer> =
        capture::create_capturer(config.capture_width, config.capture_height);
    let mut encoder: Box<dyn Encoder> = encoder::create_encoder();
    let mut fps = FpsController::new(config.min_fps, config.max_fps);

    tracing::info!(
        capturer = capturer.backend_name(),
        encoder = encoder.name(),
        "streaming pipeline initialized"
    );

    loop {
        // Stop when the peer closes the channel.
        if data_channel.ready_state() != RTCDataChannelState::Open {
            tracing::info!("data channel no longer open; stopping stream");
            return Ok(());
        }

        match capturer.next_frame() {
            Ok(Some(frame)) => match encoder.encode(&frame) {
                Ok(payload) => {
                    if let Err(e) = data_channel.send(&Bytes::from(payload)).await {
                        return Err(NetworkError::DataChannel(e.to_string()));
                    }
                }
                Err(e) => tracing::debug!("encode error: {e}"),
            },
            Ok(None) => {} // no new frame this tick
            Err(e) => tracing::debug!("capture error: {e}"),
        }

        // Adaptive pacing. A real implementation feeds measured RTT here; we
        // pace to the current target FPS.
        fps.adjust(0);
        tokio::time::sleep(fps.frame_interval()).await;
    }
}
