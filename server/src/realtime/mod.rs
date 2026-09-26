use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::WebSocket;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;
use crate::types::{ChatMessage, Incident, LocationPing};

pub mod alerts;
pub mod chat;
pub mod location;

/// Type alias for the circle membership map.
pub type CircleMembers = Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>;

/// Messages that can be broadcast through the hub.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HubMessage {
    LocationPing(LocationPing),
    ChatMessage(ChatMessage),
    Alert(Incident),
    Presence {
        user_id: Uuid,
        circle_id: Uuid,
        online: bool,
    },
}

/// WebSocket hub for real-time fan-out messaging.
#[derive(Clone)]
pub struct RealtimeHub {
    pub members: CircleMembers,
    pub tx: broadcast::Sender<HubMessage>,
}

impl RealtimeHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1024);
        Self {
            members: Arc::new(Mutex::new(HashMap::new())),
            tx,
        }
    }

    /// Add a user to a circle's member list.
    pub async fn join_circle(&self, circle_id: Uuid, user_id: Uuid) {
        let mut members = self.members.lock().await;
        members.entry(circle_id).or_default().push(user_id);
        tracing::info!("User {} joined circle {}", user_id, circle_id);
    }

    /// Remove a user from a circle's member list.
    pub async fn leave_circle(&self, circle_id: Uuid, user_id: Uuid) {
        let mut members = self.members.lock().await;
        if let Some(users) = members.get_mut(&circle_id) {
            users.retain(|&id| id != user_id);
        }
        tracing::info!("User {} left circle {}", user_id, circle_id);
    }

    /// Get the list of users in a circle.
    pub async fn get_circle_members(&self, circle_id: Uuid) -> Vec<Uuid> {
        let members = self.members.lock().await;
        members.get(&circle_id).cloned().unwrap_or_default()
    }

    /// Broadcast a message to all subscribers.
    pub fn broadcast(&self, msg: HubMessage) {
        let _ = self.tx.send(msg);
    }
}

impl Default for RealtimeHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-to-server message types.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { circle_id: Uuid },
    Leave { circle_id: Uuid },
    LocationPing { latitude: f64, longitude: f64 },
    ChatMessage { room_id: Uuid, body: String },
    Sos { latitude: f64, longitude: f64 },
}

/// Handle a WebSocket connection from a client.
pub async fn handle_connection(mut socket: WebSocket, hub: RealtimeHub, user_id: Uuid) {
    tracing::info!("WebSocket connection established for user {}", user_id);

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WebSocket recv error for user {}: {}", user_id, e);
                break;
            }
        };

        let text = match msg.into_text() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Invalid WebSocket message from user {}: {}", user_id, e);
                continue;
            }
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to parse message from user {}: {}", user_id, e);
                continue;
            }
        };

        match client_msg {
            ClientMessage::Join { circle_id } => {
                hub.join_circle(circle_id, user_id).await;
                hub.broadcast(HubMessage::Presence {
                    user_id,
                    circle_id,
                    online: true,
                });
            }
            ClientMessage::Leave { circle_id } => {
                hub.leave_circle(circle_id, user_id).await;
                hub.broadcast(HubMessage::Presence {
                    user_id,
                    circle_id,
                    online: false,
                });
            }
            ClientMessage::LocationPing { latitude, longitude } => {
                location::handle_location_ping(&hub, user_id, latitude, longitude).await;
            }
            ClientMessage::ChatMessage { room_id, body } => {
                chat::handle_chat_message(&hub, user_id, room_id, body).await;
            }
            ClientMessage::Sos { latitude, longitude } => {
                alerts::handle_sos(&hub, user_id, latitude, longitude).await;
            }
        }
    }

    tracing::info!("WebSocket connection closed for user {}", user_id);
}
