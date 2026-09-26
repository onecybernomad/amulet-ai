use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::ChatMessage;

/// Create a new chat room for a circle.
pub async fn create_room(pool: &PgPool, circle_id: Uuid, name: &str) -> Result<Uuid> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO chat_rooms (circle_id, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(circle_id)
    .bind(name)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created chat room: id={}, name={}", row.0, name);
    Ok(row.0)
}

/// Get all chat rooms for a circle.
pub async fn get_rooms(pool: &PgPool, circle_id: Uuid) -> Result<Vec<(Uuid, String, chrono::DateTime<chrono::Utc>)>> {
    let rooms = sqlx::query_as::<_, (Uuid, String, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT id, name, created_at
        FROM chat_rooms
        WHERE circle_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(circle_id)
    .fetch_all(pool)
    .await?;

    Ok(rooms)
}

/// Get recent messages from a chat room.
pub async fn get_messages(
    pool: &PgPool,
    room_id: Uuid,
    limit: i64,
) -> Result<Vec<ChatMessage>> {
    let messages = sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT id, room_id, sender_id, body, created_at
        FROM chat_messages
        WHERE room_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(room_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

/// Insert a new chat message.
pub async fn insert_message(
    pool: &PgPool,
    room_id: Uuid,
    sender_id: Uuid,
    body: &str,
) -> Result<ChatMessage> {
    let message = sqlx::query_as::<_, ChatMessage>(
        r#"
        INSERT INTO chat_messages (room_id, sender_id, body)
        VALUES ($1, $2, $3)
        RETURNING id, room_id, sender_id, body, created_at
        "#,
    )
    .bind(room_id)
    .bind(sender_id)
    .bind(body)
    .fetch_one(pool)
    .await?;

    tracing::info!("Inserted chat message: id={}", message.id);
    Ok(message)
}

/// Mark messages as read by a user up to a certain message ID.
pub async fn mark_read(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    last_read_message_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO chat_read_receipts (room_id, user_id, last_read_message_id, read_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (room_id, user_id)
        DO UPDATE SET last_read_message_id = $3, read_at = NOW()
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(last_read_message_id)
    .execute(pool)
    .await?;

    tracing::info!("Marked messages read for user {} in room {}", user_id, room_id);
    Ok(())
}
