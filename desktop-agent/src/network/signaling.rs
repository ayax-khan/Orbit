use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::env;

pub struct SignalingClient {
    pub socket_url: String,
}

impl SignalingClient {
    pub fn new(url: &str) -> Self {
        Self { socket_url: url.to_owned() }
    }

    pub async fn connect(&self) -> Result<(), String> {
        let (ws_stream, _) = connect_async(&self.socket_url).await
            .map_err(|e| e.to_string())?;
        println!("WebSocket connected to: {}", self.socket_url);
        
        let (mut write, mut read) = ws_stream.split();

        // Basic read loop for signaling messages (SDP/ICE)
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => println!("Received signaling message: {}", text),
                    Ok(Message::Close(_)) => break,
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
