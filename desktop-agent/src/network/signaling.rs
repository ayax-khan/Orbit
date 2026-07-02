//! Backend signaling client for the desktop agent (host / answerer).
//!
//! The client (Chrome extension) is the offerer. This agent connects to the
//! backend WebSocket relay for the session, then:
//!
//! 1. receives the SDP `offer`, sets it as the remote description,
//! 2. creates an `answer`, sets it locally and sends it back,
//! 3. exchanges ICE candidates in both directions.
//!
//! Incoming data channels are handed to [`crate::network::datachannel::attach`].

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::datachannel;
use super::webrtc::WebRtcPeer;
use super::NetworkError;
use crate::config::AgentConfig;

pub struct SignalingClient {
    server_url: String,
    peer: Arc<WebRtcPeer>,
    config: AgentConfig,
}

impl SignalingClient {
    /// `host` is the backend authority, e.g. `127.0.0.1:8080`.
    pub fn new(host: &str, peer: Arc<WebRtcPeer>, config: AgentConfig) -> Self {
        Self {
            server_url: format!("ws://{host}/ws"),
            peer,
            config,
        }
    }

    /// Connect to the session room and run the signaling loop until the socket
    /// closes.
    pub async fn connect(&self, session_id: &str) -> Result<(), NetworkError> {
        let url = format!("{}/{}", self.server_url, session_id);
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;
        tracing::info!("signaling connected to {url}");

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Outbound channel so async callbacks (ICE) can push messages to the
        // single WebSocket writer.
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

        // Forward locally-gathered ICE candidates to the peer via signaling.
        {
            let out_tx = out_tx.clone();
            self.peer
                .peer_connection
                .on_ice_candidate(Box::new(move |candidate| {
                    let out_tx = out_tx.clone();
                    Box::pin(async move {
                        if let Some(c) = candidate {
                            if let Ok(init) = c.to_json() {
                                let msg = serde_json::json!({ "type": "ice", "candidate": init });
                                let _ = out_tx.send(msg.to_string());
                            }
                        }
                    })
                }));
        }

        // Attach capture/input wiring to any data channel the client opens.
        {
            let config = self.config.clone();
            self.peer
                .peer_connection
                .on_data_channel(Box::new(move |dc| {
                    tracing::info!(label = %dc.label(), "incoming data channel");
                    datachannel::attach(dc, config.clone());
                    Box::pin(async {})
                }));
        }

        // Writer task: drain outbound queue to the socket.
        let writer = tokio::spawn(async move {
            while let Some(text) = out_rx.recv().await {
                if ws_write.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        });

        // Reader loop: process signaling messages.
        while let Some(Ok(message)) = ws_read.next().await {
            let Message::Text(text) = message else {
                if matches!(message, Message::Close(_)) {
                    break;
                }
                continue;
            };

            let value: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!("ignoring malformed signaling message: {e}");
                    continue;
                }
            };

            match value.get("type").and_then(|t| t.as_str()) {
                Some("offer") => {
                    if let Err(e) = self.handle_offer(&value, &out_tx).await {
                        tracing::error!("failed to handle offer: {e}");
                    }
                }
                Some("ice") => {
                    if let Err(e) = self.handle_ice(&value).await {
                        tracing::debug!("failed to add ICE candidate: {e}");
                    }
                }
                other => tracing::debug!(?other, "unhandled signaling message"),
            }
        }

        writer.abort();
        tracing::info!("signaling session ended");
        Ok(())
    }

    async fn handle_offer(
        &self,
        value: &serde_json::Value,
        out_tx: &mpsc::UnboundedSender<String>,
    ) -> Result<(), NetworkError> {
        // The client sends { type: "offer", sdp: <RTCSessionDescriptionInit> }.
        let sdp = value
            .get("sdp")
            .and_then(|s| s.get("sdp"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| NetworkError::Signaling("offer missing sdp".into()))?;

        let offer = RTCSessionDescription::offer(sdp.to_owned())
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;
        let pc = &self.peer.peer_connection;

        pc.set_remote_description(offer)
            .await
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;

        let answer = pc
            .create_answer(None)
            .await
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;
        pc.set_local_description(answer.clone())
            .await
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;

        let msg = serde_json::json!({ "type": "answer", "sdp": answer });
        out_tx
            .send(msg.to_string())
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;
        tracing::info!("sent SDP answer");
        Ok(())
    }

    async fn handle_ice(&self, value: &serde_json::Value) -> Result<(), NetworkError> {
        let candidate = value
            .get("candidate")
            .cloned()
            .ok_or_else(|| NetworkError::Signaling("ice message missing candidate".into()))?;

        let init: RTCIceCandidateInit = serde_json::from_value(candidate)
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;

        self.peer
            .peer_connection
            .add_ice_candidate(init)
            .await
            .map_err(|e| NetworkError::Signaling(e.to_string()))?;
        Ok(())
    }
}
