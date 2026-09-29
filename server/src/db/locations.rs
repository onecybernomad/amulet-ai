use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::MemberLocationResponse;

/// Insert a new location ping for a user in a circle.
pub async fn insert_location_ping(
    pool: &SqlitePool,
    user_id: Uuid,
    circle_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO location_pings (user_id, circle_id, latitude, longitude, recorded_at)
        VALUES (?, ?, ?, ?, datetime('now'))
        "#,
    )
    .bind(user_id.to_string())
    .bind(circle_id.to_string())
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await?;

    tracing::debug!("Inserted location ping: user={} circle={} lat={} lng={}", user_id, circle_id, latitude, longitude);
    Ok(())
}

/// Get the most recent location for each member of a circle.
/// Uses a subquery with MAX(recorded_at) to simulate PostgreSQL's DISTINCT ON.
pub async fn get_member_locations(
    pool: &SqlitePool,
    circle_id: Uuid,
) -> Result<Vec<MemberLocationResponse>> {
    let rows = sqlx::query(
        r#"
        SELECT lp.user_id, lp.latitude, lp.longitude, lp.recorded_at as timestamp
        FROM location_pings lp
        INNER JOIN (
            SELECT user_id, MAX(recorded_at) AS max_recorded_at
            FROM location_pings
            WHERE circle_id = ?
            GROUP BY user_id
        ) latest ON lp.user_id = latest.user_id AND lp.recorded_at = latest.max_recorded_at
        WHERE lp.circle_id = ?
        "#,
    )
    .bind(circle_id.to_string())
    .bind(circle_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut locations = Vec::new();
    for row in rows {
        locations.push(MemberLocationResponse {
            user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            timestamp: crate::db::parse_datetime(&row.try_get::<String, _>("timestamp")?)?,
        });
    }

    Ok(locations)
}

/// Get location history for a specific user within a circle.
pub async fn get_location_history(
    pool: &SqlitePool,
    circle_id: Uuid,
    user_id: Uuid,
    since: DateTime<Utc>,
) -> Result<Vec<MemberLocationResponse>> {
    let rows = sqlx::query(
        r#"
        SELECT user_id, latitude, longitude, recorded_at as timestamp
        FROM location_pings
        WHERE circle_id = ? AND user_id = ? AND recorded_at > ?
        ORDER BY recorded_at DESC
        LIMIT 500
        "#,
    )
    .bind(circle_id.to_string())
    .bind(user_id.to_string())
    .bind(since.to_rfc3339())
    .fetch_all(pool)
    .await?;

    let mut locations = Vec::new();
    for row in rows {
        locations.push(MemberLocationResponse {
            user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            timestamp: crate::db::parse_datetime(&row.try_get::<String, _>("timestamp")?)?,
        });
    }

    Ok(locations)
}

/// Delete location pings older than the retention period.
/// Uses SQLite datetime modifier for date arithmetic instead of PostgreSQL's ::interval.
pub async fn delete_old_locations(pool: &SqlitePool, retention_days: i32) -> Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM location_pings
        WHERE recorded_at < datetime('now', ? || ' days')
        "#,
    )
    .bind(retention_days.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Deleted {} old location pings (retention: {} days)", result.rows_affected(), retention_days);
    Ok(result.rows_affected())
}
