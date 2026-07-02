//! WebRTC peer wrapper for the desktop agent (the "host" side).

use std::sync::Arc;

use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

use super::NetworkError;

/// Thin wrapper around an [`RTCPeerConnection`] configured with the
/// STUN/TURN servers provided by the backend.
pub struct WebRtcPeer {
    pub peer_connection: Arc<RTCPeerConnection>,
}

impl WebRtcPeer {
    /// Build a peer connection using the given ICE server URLs.
    pub async fn new(ice_urls: Vec<String>) -> Result<Self, NetworkError> {
        let mut media_engine = webrtc::api::media_engine::MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|e| NetworkError::Setup(e.to_string()))?;

        let api = APIBuilder::new().with_media_engine(media_engine).build();

        let ice_servers = if ice_urls.is_empty() {
            vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }]
        } else {
            vec![RTCIceServer {
                urls: ice_urls,
                ..Default::default()
            }]
        };

        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        let pc = api
            .new_peer_connection(config)
            .await
            .map_err(|e| NetworkError::Setup(e.to_string()))?;

        Ok(Self {
            peer_connection: Arc::new(pc),
        })
    }
}
