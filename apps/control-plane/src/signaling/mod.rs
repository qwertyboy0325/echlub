use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const MAX_MESSAGE_BYTES: usize = 16_384;
const MAX_PEERS_PER_SESSION: usize = 2;
const MAX_SESSION_ID_LEN: usize = 64;
const MAX_PEER_ID_LEN: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalingMessage {
    Join {
        peer_id: String,
    },
    Leave {
        peer_id: String,
    },
    Relay {
        from: String,
        payload: serde_json::Value,
    },
    Error {
        code: String,
        message: String,
    },
    PeerJoined {
        peer_id: String,
    },
    PeerLeft {
        peer_id: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct SignalingQuery {
    pub session_id: String,
    pub peer_id: String,
}

#[derive(Clone, Default)]
pub struct SignalingState {
    inner: Arc<Mutex<HashMap<String, SessionRoom>>>,
}

struct SessionRoom {
    peers: HashMap<String, tokio::sync::broadcast::Sender<String>>,
}

pub fn validate_origin(headers: &HeaderMap) -> bool {
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => {
            origin.starts_with("http://localhost")
                || origin.starts_with("http://127.0.0.1")
                || origin.starts_with("https://localhost")
                || origin.starts_with("https://127.0.0.1")
        }
        None => true,
    }
}

pub fn validate_ids(session_id: &str, peer_id: &str) -> Result<(), (StatusCode, String)> {
    if session_id.is_empty() || session_id.len() > MAX_SESSION_ID_LEN {
        return Err((StatusCode::BAD_REQUEST, "invalid session_id".into()));
    }
    if peer_id.is_empty() || peer_id.len() > MAX_PEER_ID_LEN {
        return Err((StatusCode::BAD_REQUEST, "invalid peer_id".into()));
    }
    let valid_chars = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    if !session_id.chars().all(valid_chars) || !peer_id.chars().all(valid_chars) {
        return Err((
            StatusCode::BAD_REQUEST,
            "id contains invalid characters".into(),
        ));
    }
    Ok(())
}

pub async fn signaling_ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    Query(query): Query<SignalingQuery>,
    State(state): State<SignalingState>,
    headers: HeaderMap,
) -> Response {
    if !validate_origin(&headers) {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    if let Err((status, msg)) = validate_ids(&query.session_id, &query.peer_id) {
        return (status, msg).into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(socket: WebSocket, state: SignalingState, query: SignalingQuery) {
    let session_id = query.session_id;
    let peer_id = query.peer_id;

    let mut rx = match join_room(&state, &session_id, &peer_id) {
        Ok(rx) => rx,
        Err(msg) => {
            let (mut sender, _) = socket.split();
            let err = SignalingMessage::Error {
                code: "session_full".into(),
                message: msg,
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&err).unwrap().into()))
                .await;
            return;
        }
    };

    let (mut sender, mut receiver) = socket.split();

    let join_msg = SignalingMessage::Join {
        peer_id: peer_id.clone(),
    };
    if sender
        .send(Message::Text(
            serde_json::to_string(&join_msg).unwrap().into(),
        ))
        .await
        .is_err()
    {
        remove_peer(&state, &session_id, &peer_id);
        return;
    }

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if text.len() > MAX_MESSAGE_BYTES {
                            continue;
                        }
                        relay_message(&state, &session_id, &peer_id, text.as_ref());
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            relay = rx.recv() => {
                match relay {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    remove_peer(&state, &session_id, &peer_id);
}

fn join_room(
    state: &SignalingState,
    session_id: &str,
    peer_id: &str,
) -> Result<tokio::sync::broadcast::Receiver<String>, String> {
    let mut rooms = state.inner.lock().unwrap();
    let room = rooms
        .entry(session_id.to_string())
        .or_insert_with(|| SessionRoom {
            peers: HashMap::new(),
        });

    if room.peers.len() >= MAX_PEERS_PER_SESSION && !room.peers.contains_key(peer_id) {
        return Err(format!("max {MAX_PEERS_PER_SESSION} peers per session"));
    }

    let (tx, rx) = tokio::sync::broadcast::channel(32);
    room.peers.insert(peer_id.to_string(), tx.clone());

    let joined = SignalingMessage::PeerJoined {
        peer_id: peer_id.to_string(),
    };
    let msg = serde_json::to_string(&joined).unwrap();
    for (other_id, other_tx) in &room.peers {
        if other_id != peer_id {
            let _ = other_tx.send(msg.clone());
        }
    }

    Ok(rx)
}

fn relay_message(state: &SignalingState, session_id: &str, from: &str, text: &str) {
    let payload: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
    let relay = SignalingMessage::Relay {
        from: from.to_string(),
        payload,
    };
    let msg = serde_json::to_string(&relay).unwrap();

    let rooms = state.inner.lock().unwrap();
    if let Some(room) = rooms.get(session_id) {
        for (peer_id, tx) in &room.peers {
            if peer_id != from {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

fn remove_peer(state: &SignalingState, session_id: &str, peer_id: &str) {
    let mut rooms = state.inner.lock().unwrap();
    if let Some(room) = rooms.get_mut(session_id) {
        room.peers.remove(peer_id);
        let leave = SignalingMessage::PeerLeft {
            peer_id: peer_id.to_string(),
        };
        let msg = serde_json::to_string(&leave).unwrap();
        for tx in room.peers.values() {
            let _ = tx.send(msg.clone());
        }
        if room.peers.is_empty() {
            rooms.remove(session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ids_rejects_empty() {
        assert!(validate_ids("", "peer1").is_err());
        assert!(validate_ids("session1", "").is_err());
    }

    #[test]
    fn validate_ids_accepts_valid() {
        assert!(validate_ids("session-1", "peer_a").is_ok());
    }

    #[test]
    fn max_peers_is_two() {
        assert_eq!(MAX_PEERS_PER_SESSION, 2);
    }
}
