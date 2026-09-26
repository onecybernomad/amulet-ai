use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::MemberLocationResponse;

/// Insert a new location ping for a user in a circle.
pub async fn insert_location_ping(
    pool: &PgPool,
    user_id: Uuid,
    circle_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO location_pings (user_id, circle_id, latitude, longitude, recorded_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(user_id)
    .bind(circle_id)
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await?;

    tracing::debug!("Inserted location ping: user={} circle={} lat={} lng={}", user_id, circle_id, latitude, longitude);
    Ok(())
}

/// Get the most recent location for each member of a circle.
pub async fn get_member_locations(
    pool: &PgPool,
    circle_id: Uuid,
) -> Result<Vec<MemberLocationResponse>> {
    let rows = sqlx::query_as::<_, MemberLocationResponse>(
        r#"
        SELECT DISTINCT ON (lp.user_id)
            lp.user_id,
            lp.latitude,
            lp.longitude,
            lp.recorded_at as timestamp
        FROM location_pings lp
        WHERE lp.circle_id = $1
        ORDER BY lp.user_id, lp.recorded_at DESC
        "#,
    )
    .bind(circle_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get location history for a specific user within a circle.
pub async fn get_location_history(
    pool: &PgPool,
    circle_id: Uuid,
    user_id: Uuid,
    since: DateTime<Utc>,
) -> Result<Vec<MemberLocationResponse>> {
    let rows = sqlx::query_as::<_, MemberLocationResponse>(
        r#"
        SELECT user_id, latitude, longitude, recorded_at as timestamp
        FROM location_pings
        WHERE circle_id = $1 AND user_id = $2 AND recorded_at > $3
        ORDER BY recorded_at DESC
        LIMIT 500
        "#,
    )
    .bind(circle_id)
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Delete location pings older than the retention period.
pub async fn delete_old_locations(pool: &PgPool, retention_days: i32) -> Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM location_pings
        WHERE recorded_at < NOW() - ($1 || ' days')::interval
        "#,
    )
    .bind(retention_days)
    .execute(pool)
    .await?;

    tracing::info!("Deleted {} old location pings (retention: {} days)", result.rows_affected(), retention_days);
    Ok(result.rows_affected())
}
