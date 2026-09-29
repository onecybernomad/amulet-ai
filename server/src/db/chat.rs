use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::ChatMessage;

/// Create a new chat room for a circle.
pub async fn create_room(pool: &SqlitePool, circle_id: Uuid, name: &str) -> Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO chat_rooms (circle_id, name)
        VALUES (?, ?)
        RETURNING id
        "#,
    )
    .bind(circle_id.to_string())
    .bind(name)
    .fetch_one(pool)
    .await?;

    let id: Uuid = Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?;
    tracing::info!("Created chat room: id={}, name={}", id, name);
    Ok(id)
}

/// Get all chat rooms for a circle.
pub async fn get_rooms(pool: &SqlitePool, circle_id: Uuid) -> Result<Vec<(Uuid, String, chrono::DateTime<chrono::Utc>)>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, created_at
        FROM chat_rooms
        WHERE circle_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(circle_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut rooms = Vec::new();
    for row in rows {
        let id: Uuid = Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?;
        let name: String = row.try_get("name")?;
        let created_at: chrono::DateTime<chrono::Utc> = crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?;
        rooms.push((id, name, created_at));
    }

    Ok(rooms)
}

/// Get recent messages from a chat room.
pub async fn get_messages(
    pool: &SqlitePool,
    room_id: Uuid,
    limit: i64,
) -> Result<Vec<ChatMessage>> {
    let rows = sqlx::query(
        r#"
        SELECT id, room_id, sender_id, body, created_at
        FROM chat_messages
        WHERE room_id = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(room_id.to_string())
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(ChatMessage {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            room_id: Uuid::parse_str(&row.try_get::<String, _>("room_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            sender_id: Uuid::parse_str(&row.try_get::<String, _>("sender_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            body: row.try_get("body")?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(messages)
}

/// Insert a new chat message.
pub async fn insert_message(
    pool: &SqlitePool,
    room_id: Uuid,
    sender_id: Uuid,
    body: &str,
) -> Result<ChatMessage> {
    let row = sqlx::query(
        r#"
        INSERT INTO chat_messages (room_id, sender_id, body)
        VALUES (?, ?, ?)
        RETURNING id, room_id, sender_id, body, created_at
        "#,
    )
    .bind(room_id.to_string())
    .bind(sender_id.to_string())
    .bind(body)
    .fetch_one(pool)
    .await?;

    let message = ChatMessage {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        room_id: Uuid::parse_str(&row.try_get::<String, _>("room_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        sender_id: Uuid::parse_str(&row.try_get::<String, _>("sender_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        body: row.try_get("body")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Inserted chat message: id={}", message.id);
    Ok(message)
}

/// Mark messages as read by a user up to a certain message ID.
pub async fn mark_read(
    pool: &SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
    last_read_message_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO chat_read_receipts (room_id, user_id, last_read_message_id, read_at)
        VALUES (?, ?, ?, datetime('now'))
        ON CONFLICT (room_id, user_id)
        DO UPDATE SET last_read_message_id = ?, read_at = datetime('now')
        "#,
    )
    .bind(room_id.to_string())
    .bind(user_id.to_string())
    .bind(last_read_message_id.to_string())
    .bind(last_read_message_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Marked messages read for user {} in room {}", user_id, room_id);
    Ok(())
}
