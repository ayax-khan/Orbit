use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use std::sync::Arc;

pub struct WebRtcPeer {
    pub peer_connection: Arc<RTCPeerConnection>,
}

impl WebRtcPeer {
    pub async fn new() -> Result<Self, String> {
        let mut m = webrtc::api::media_engine::MediaEngine::default();
        m.register_default_codecs().map_err(|e| e.to_string())?;
        
        let api = APIBuilder::new()
            .with_media_engine(m)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let pc = api.new_peer_connection(config).await.map_err(|e| e.to_string())?;

        Ok(Self {
            peer_connection: Arc::new(pc),
        })
    }
}
