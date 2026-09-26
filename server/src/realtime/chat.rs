use uuid::Uuid;
use super::{HubMessage, RealtimeHub};

/// Handle an incoming chat_message message from a client.
/// Validates the message, stores in DB, and broadcasts to room members.
pub async fn handle_chat_message(
    hub: &RealtimeHub,
    user_id: Uuid,
    room_id: Uuid,
    body: String,
) {
    // Validate message body
    let body = body.trim().to_string();
    if body.is_empty() {
        tracing::warn!("Empty chat message from user {}", user_id);
        return;
    }
    if body.len() > 4096 {
        tracing::warn!("Chat message too long from user {}: {} chars", user_id, body.len());
        return;
    }

    tracing::debug!("Chat message from user {} in room {}: {}", user_id, room_id, body);

    // Create the message entity
    let message = crate::types::ChatMessage {
        id: Uuid::new_v4(),
        room_id,
        sender_id: user_id,
        body,
        created_at: chrono::Utc::now(),
    };

    // Broadcast the message
    hub.broadcast(HubMessage::ChatMessage(message.clone()));
    tracing::debug!("Broadcasted chat message to room {}", room_id);
}
