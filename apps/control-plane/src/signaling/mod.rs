use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::validate_origin;

const MAX_MESSAGE_BYTES: usize = 16_384;
const MAX_PEERS_PER_SESSION: usize = 2;
const MAX_SESSION_ID_LEN: usize = 64;
const MAX_PEER_ID_LEN: usize = 64;
const ALLOWED_RELAY_TYPES: &[&str] = &["description", "ice_candidate", "control", "relay"];

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

#[derive(Clone)]
pub struct SignalingState {
    inner: Arc<Mutex<HashMap<String, SessionRoom>>>,
    extra_origins: Arc<Vec<String>>,
}

struct SessionRoom {
    peers: HashMap<String, tokio::sync::broadcast::Sender<String>>,
}

impl SignalingState {
    pub fn new(extra_origins: Vec<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            extra_origins: Arc::new(extra_origins),
        }
    }

    pub fn default_with_origins(extra_origins: Vec<String>) -> Self {
        Self::new(extra_origins)
    }
}

impl Default for SignalingState {
    fn default() -> Self {
        Self::new(vec![])
    }
}

pub fn validate_origin_header(headers: &HeaderMap, extra_origins: &[String]) -> bool {
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => validate_origin(origin, extra_origins),
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

pub fn validate_relay_envelope(text: &str) -> Result<serde_json::Value, &'static str> {
    if text.len() > MAX_MESSAGE_BYTES {
        return Err("oversized payload");
    }
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|_| "malformed relay envelope")?;
    let msg_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("missing relay type")?;
    if !ALLOWED_RELAY_TYPES.contains(&msg_type) {
        return Err("unsupported relay type");
    }
    if msg_type == "relay" {
        let payload = value.get("payload").ok_or("malformed relay envelope")?;
        let inner_type = payload
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or("missing relay type")?;
        if !["description", "ice_candidate", "control", "offer", "answer"].contains(&inner_type) {
            return Err("unsupported relay type");
        }
        return Ok(payload.clone());
    }
    Ok(value)
}

pub async fn signaling_ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    Query(query): Query<SignalingQuery>,
    State(state): State<SignalingState>,
    headers: HeaderMap,
) -> Response {
    if !validate_origin_header(&headers, &state.extra_origins) {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    if let Err((status, msg)) = validate_ids(&query.session_id, &query.peer_id) {
        return (status, msg).into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

#[derive(Debug)]
struct JoinResult {
    rx: tokio::sync::broadcast::Receiver<String>,
    existing_peer_ids: Vec<String>,
}

async fn handle_socket(socket: WebSocket, state: SignalingState, query: SignalingQuery) {
    let session_id = query.session_id;
    let peer_id = query.peer_id;

    let join_result = match join_room(&state, &session_id, &peer_id) {
        Ok(result) => result,
        Err(code) => {
            let (mut sender, _) = socket.split();
            let err = SignalingMessage::Error {
                code: code.to_string(),
                message: code.to_string(),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&err).unwrap().into()))
                .await;
            return;
        }
    };

    let mut rx = join_result.rx;
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

    for existing_peer_id in join_result.existing_peer_ids {
        let existing_joined = SignalingMessage::PeerJoined {
            peer_id: existing_peer_id,
        };
        if sender
            .send(Message::Text(
                serde_json::to_string(&existing_joined).unwrap().into(),
            ))
            .await
            .is_err()
        {
            remove_peer(&state, &session_id, &peer_id);
            return;
        }
    }

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match validate_relay_envelope(text.as_ref()) {
                            Ok(payload) => relay_message(&state, &session_id, &peer_id, payload),
                            Err(code) => {
                                let err = SignalingMessage::Error {
                                    code: code.to_string(),
                                    message: code.to_string(),
                                };
                                let _ = sender.send(Message::Text(serde_json::to_string(&err).unwrap().into())).await;
                            }
                        }
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
) -> Result<JoinResult, &'static str> {
    let mut rooms = state.inner.lock().unwrap();
    let room = rooms
        .entry(session_id.to_string())
        .or_insert_with(|| SessionRoom {
            peers: HashMap::new(),
        });

    if room.peers.contains_key(peer_id) {
        return Err("duplicate peer");
    }

    if room.peers.len() >= MAX_PEERS_PER_SESSION {
        return Err("session_full");
    }

    let existing_peer_ids: Vec<String> = room.peers.keys().cloned().collect();
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

    Ok(JoinResult {
        rx,
        existing_peer_ids,
    })
}

fn relay_message(state: &SignalingState, session_id: &str, from: &str, payload: serde_json::Value) {
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
    fn validate_relay_rejects_oversized() {
        let big = "a".repeat(MAX_MESSAGE_BYTES + 1);
        assert_eq!(
            validate_relay_envelope(&big).unwrap_err(),
            "oversized payload"
        );
    }

    #[test]
    fn validate_relay_accepts_description() {
        let payload = validate_relay_envelope(
            r#"{"type":"relay","payload":{"type":"description","sdp":{"type":"offer","sdp":"x"}}}"#,
        );
        assert!(payload.is_ok());
    }

    #[test]
    fn max_peers_is_two() {
        assert_eq!(MAX_PEERS_PER_SESSION, 2);
    }

    #[tokio::test]
    async fn peer_a_first_notifies_joiner_of_existing_peer_b() {
        let state = SignalingState::default();
        let session = "session-1";

        let join_b = join_room(&state, session, "peer_b").expect("peer_b joins");
        assert!(join_b.existing_peer_ids.is_empty());
        let mut rx_b = join_b.rx;

        let join_a = join_room(&state, session, "peer_a").expect("peer_a joins");
        assert_eq!(join_a.existing_peer_ids, vec!["peer_b"]);

        let msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx_b.recv())
            .await
            .expect("timed out waiting for peer_joined")
            .expect("broadcast closed");
        let parsed: SignalingMessage = serde_json::from_str(&msg).unwrap();
        match parsed {
            SignalingMessage::PeerJoined { peer_id } => assert_eq!(peer_id, "peer_a"),
            other => panic!("expected peer_joined, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn peer_b_first_notifies_peer_a_of_existing_peer_b() {
        let state = SignalingState::default();
        let session = "session-1";

        let join_b = join_room(&state, session, "peer_b").expect("peer_b joins");
        assert!(join_b.existing_peer_ids.is_empty());

        let join_a = join_room(&state, session, "peer_a").expect("peer_a joins");
        assert_eq!(join_a.existing_peer_ids, vec!["peer_b"]);
    }

    #[test]
    fn duplicate_join_rejected() {
        let state = SignalingState::default();
        let session = "session-1";
        assert!(join_room(&state, session, "peer_a").is_ok());
        assert_eq!(
            join_room(&state, session, "peer_a").unwrap_err(),
            "duplicate peer"
        );
    }

    #[test]
    fn third_peer_rejected_and_room_cleaned_after_leave() {
        let state = SignalingState::default();
        let session = "session-1";
        let join_a = join_room(&state, session, "peer_a").expect("peer_a joins");
        let join_b = join_room(&state, session, "peer_b").expect("peer_b joins");
        assert_eq!(
            join_room(&state, session, "peer_c").unwrap_err(),
            "session_full"
        );
        drop(join_a.rx);
        drop(join_b.rx);
        remove_peer(&state, session, "peer_a");
        remove_peer(&state, session, "peer_b");
        assert!(state.inner.lock().unwrap().get(session).is_none());
    }
}
