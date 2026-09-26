use crate::db::local_cache::LocalCache;
use crate::models::{ChatMessage, ChatRoom};
use serde::Deserialize;
use tauri::State;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetMessagesRequest {
    pub room_id: String,
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: String,
    pub body: String,
    pub media_urls: Option<Vec<String>>,
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Get all chat rooms for the current user.
#[tauri::command]
pub async fn get_rooms(
    cache: State<'_, LocalCache>,
) -> Result<Vec<ChatRoom>, String> {
    info!("Getting chat rooms");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;
    let rooms = cache.get_chat_rooms(&user.id).map_err(|e| e.to_string())?;
    debug!(count = rooms.len(), "Retrieved chat rooms");
    Ok(rooms)
}

/// Get messages in a chat room.
#[tauri::command]
pub async fn get_messages(
    req: GetMessagesRequest,
    cache: State<'_, LocalCache>,
) -> Result<Vec<ChatMessage>, String> {
    info!(room_id = %req.room_id, "Getting messages");
    let limit = req.limit.unwrap_or(50);
    let messages = cache.get_chat_messages(&req.room_id, req.before, limit).map_err(|e| e.to_string())?;
    debug!(count = messages.len(), "Retrieved messages");
    Ok(messages)
}

/// Send a message to a chat room.
#[tauri::command]
#[instrument(skip(req))]
pub async fn send_message(
    req: SendMessageRequest,
    cache: State<'_, LocalCache>,
) -> Result<ChatMessage, String> {
    info!(room_id = %req.room_id, "Sending message");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        room_id: req.room_id,
        sender_id: user.id,
        body: req.body,
        media_urls: req.media_urls,
        read_by: vec![],
        created_at: chrono::Utc::now(),
    };

    cache.insert_chat_message(&message).map_err(|e| e.to_string())?;
    debug!(message_id = %message.id, "Message sent");
    Ok(message)
}
