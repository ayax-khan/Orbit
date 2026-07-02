//! ORBIT Desktop Agent (host).
//!
//! Captures the screen, encodes it, and streams it to a remote client over a
//! WebRTC data channel while applying the client's mouse/keyboard input. The
//! backend acts as the signaling relay; the client is the WebRTC offerer and
//! this agent answers.

mod capture;
mod config;
mod encoder;
mod input;
mod network;

use std::sync::Arc;

use config::AgentConfig;
use network::{SignalingClient, WebRtcPeer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AgentConfig::from_env();
    tracing::info!(backend = %config.backend_host, "starting ORBIT desktop agent");

    if config.session_id.is_empty() {
        anyhow::bail!(
            "no session id configured. Set ORBIT_SESSION_ID to the session token \
             returned by the backend's /api/v1/sessions/create endpoint."
        );
    }

    // Build the peer connection with the configured ICE servers.
    let peer = Arc::new(
        WebRtcPeer::new(config.ice_servers.clone())
            .await
            .map_err(|e| anyhow::anyhow!("failed to create WebRTC peer: {e}"))?,
    );

    // Connect to the backend signaling relay and run until the session ends.
    let client = SignalingClient::new(&config.backend_host, peer, config.clone());
    client
        .connect(&config.session_id)
        .await
        .map_err(|e| anyhow::anyhow!("signaling failed: {e}"))?;

    Ok(())
}
