use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;

/// Record a geofence enter/exit event.
pub async fn record_geofence_event(
    pool: &SqlitePool,
    place_id: Uuid,
    place_name: &str,
    circle_id: Uuid,
    user_id: Uuid,
    event_type: &str,
    latitude: f64,
    longitude: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO geofence_events (id, place_id, place_name, circle_id, user_id, event_type, latitude, longitude)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(place_id.to_string())
    .bind(place_name)
    .bind(circle_id.to_string())
    .bind(user_id.to_string())
    .bind(event_type)
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await?;

    tracing::info!(
        "Recorded geofence {} event: user={} place={} ({})",
        event_type,
        user_id,
        place_name,
        circle_id
    );
    Ok(())
}

/// Get recent geofence events for a circle.
pub async fn get_circle_geofence_events(
    pool: &SqlitePool,
    circle_id: Uuid,
    limit: i64,
) -> Result<Vec<GeofenceEventRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, place_id, place_name, circle_id, user_id, event_type, latitude, longitude, notified, created_at
        FROM geofence_events
        WHERE circle_id = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(circle_id.to_string())
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        events.push(GeofenceEventRow {
            id: row.try_get("id")?,
            place_id: row.try_get("place_id")?,
            place_name: row.try_get("place_name")?,
            circle_id: row.try_get("circle_id")?,
            user_id: row.try_get("user_id")?,
            event_type: row.try_get("event_type")?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            notified: row.try_get::<i64, _>("notified")? != 0,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(events)
}

/// Get the list of places a user is currently inside (for exit detection).
pub async fn get_user_geofence_state(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<Vec<UserGeofenceState>> {
    let rows = sqlx::query(
        r#"
        SELECT user_id, place_id, place_name, circle_id, entered_at
        FROM user_geofence_state
        WHERE user_id = ?
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut states = Vec::new();
    for row in rows {
        states.push(UserGeofenceState {
            user_id: row.try_get("user_id")?,
            place_id: row.try_get("place_id")?,
            place_name: row.try_get("place_name")?,
            circle_id: row.try_get("circle_id")?,
            entered_at: crate::db::parse_datetime(&row.try_get::<String, _>("entered_at")?)?,
        });
    }

    Ok(states)
}

/// Update the geofence state when a user enters a place.
pub async fn mark_user_entered(
    pool: &SqlitePool,
    user_id: Uuid,
    place_id: Uuid,
    place_name: &str,
    circle_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO user_geofence_state (user_id, place_id, place_name, circle_id, entered_at)
        VALUES (?, ?, ?, ?, datetime('now'))
        "#,
    )
    .bind(user_id.to_string())
    .bind(place_id.to_string())
    .bind(place_name)
    .bind(circle_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Update the geofence state when a user exits a place.
pub async fn mark_user_exited(
    pool: &SqlitePool,
    user_id: Uuid,
    place_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM user_geofence_state
        WHERE user_id = ? AND place_id = ?
        "#,
    )
    .bind(user_id.to_string())
    .bind(place_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Mark geofence events as notified.
pub async fn mark_events_notified(pool: &SqlitePool, event_ids: &[String]) -> Result<()> {
    if event_ids.is_empty() {
        return Ok(());
    }
    let placeholders = event_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "UPDATE geofence_events SET notified = 1 WHERE id IN ({})",
        placeholders
    );
    let mut q = sqlx::query(&query);
    for id in event_ids {
        q = q.bind(id);
    }
    q.execute(pool).await?;
    Ok(())
}

// ── Row Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct GeofenceEventRow {
    pub id: String,
    pub place_id: String,
    pub place_name: String,
    pub circle_id: String,
    pub user_id: String,
    pub event_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserGeofenceState {
    pub user_id: String,
    pub place_id: String,
    pub place_name: String,
    pub circle_id: String,
    pub entered_at: DateTime<Utc>,
}
