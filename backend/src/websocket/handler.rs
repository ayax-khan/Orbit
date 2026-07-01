use axum::{
    extract::{ws::{Message, WebSocket}, WebSocketUpgrade, Path},
    response::IntoResponse,
};
use futures_util::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>;
static CLIENTS: once_cell::sync::Lazy<Clients> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id))
}

async fn handle_socket(mut socket: WebSocket, session_id: String) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    CLIENTS.lock().unwrap().insert(session_id.clone(), tx);

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Task to send messages from MPSC to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() { break; }
        }
    });

    // Task to receive messages from WebSocket and relay
    while let Some(msg) = ws_rx.next().await {
        if let Ok(Message::Text(text)) = msg {
            // Simple relay: broadcast to all in session (or implement peer-to-peer logic)
            let clients = CLIENTS.lock().unwrap();
            for (id, tx) in clients.iter() {
                if id == &session_id {
                    let _ = tx.send(Message::Text(text.clone()));
                }
            }
        }
    }
    
    CLIENTS.lock().unwrap().remove(&session_id);
    send_task.abort();
}
