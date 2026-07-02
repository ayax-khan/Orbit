use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Each session holds a map of connected peers (client + host). Messages from
/// one peer are relayed to every *other* peer in the same session. This is the
/// signaling channel used to exchange SDP offers/answers and ICE candidates.
type PeerSender = mpsc::UnboundedSender<Message>;
type Session = HashMap<String, PeerSender>;
type Rooms = Arc<Mutex<HashMap<String, Session>>>;

static ROOMS: once_cell::sync::Lazy<Rooms> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn ws_handler(ws: WebSocketUpgrade, Path(session_id): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id))
}

async fn handle_socket(socket: WebSocket, session_id: String) {
    let peer_id = Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register this peer in the room.
    {
        let mut rooms = ROOMS.lock().unwrap();
        rooms
            .entry(session_id.clone())
            .or_default()
            .insert(peer_id.clone(), tx);
    }
    tracing::info!(%session_id, %peer_id, "peer joined signaling session");

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Forward queued messages to this peer's socket.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Relay inbound messages to the *other* peers in the same session.
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => relay_to_others(&session_id, &peer_id, Message::Text(text)),
            Message::Binary(bin) => relay_to_others(&session_id, &peer_id, Message::Binary(bin)),
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup on disconnect.
    {
        let mut rooms = ROOMS.lock().unwrap();
        if let Some(session) = rooms.get_mut(&session_id) {
            session.remove(&peer_id);
            if session.is_empty() {
                rooms.remove(&session_id);
            }
        }
    }
    send_task.abort();
    tracing::info!(%session_id, %peer_id, "peer left signaling session");
}

fn relay_to_others(session_id: &str, sender_id: &str, msg: Message) {
    let rooms = ROOMS.lock().unwrap();
    if let Some(session) = rooms.get(session_id) {
        for (id, tx) in session.iter() {
            if id != sender_id {
                let _ = tx.send(msg.clone());
            }
        }
    }
}
