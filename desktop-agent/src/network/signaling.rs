use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use super::webrtc::WebRtcPeer;
use std::sync::Arc;

pub struct SignalingClient {
    pub server_url: String,
    pub peer: Arc<WebRtcPeer>,
}

impl SignalingClient {
    pub fn new(host: &str, peer: Arc<WebRtcPeer>) -> Self {
        Self { 
            server_url: format!("ws://{}/ws", host),
            peer
        }
    }

    pub async fn connect(&self, session_id: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.server_url, session_id);
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| e.to_string())?;
        println!("WebSocket connected to: {}", url);
        
        let (mut write, mut read) = ws_stream.split();
        let peer = self.peer.clone();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
// ... existing imports ...
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

// ... inside tokio::spawn block ...
                if let Ok(Message::Text(text)) = message {
                    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                    
                    if v["type"] == "offer" {
                        // 1. Set Remote
                        let start = std::time::Instant::now();
                        let offer = RTCSessionDescription::offer(v["sdp"]["sdp"].as_str().unwrap().to_owned());
                        peer.peer_connection.set_remote_description(offer).await.unwrap();
                        
                        // 2. Create Answer
                        let answer = peer.peer_connection.create_answer(None).await.unwrap();
                        peer.peer_connection.set_local_description(answer.clone()).await.unwrap();
                        
                        // 3. Send Answer
                        let msg = serde_json::json!({ "type": "answer", "sdp": answer });
                        write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.unwrap();
                        
                        println!("Handshake complete in: {:?}", start.elapsed());
                    } else if v["type"] == "ice" {
                        // Handle ICE
                        let candidate = serde_json::from_value(v["candidate"].clone()).unwrap();
                        peer.peer_connection.add_ice_candidate(candidate).await.unwrap();
                    }
                }
// ...
            }
        });

        Ok(())
    }
}
