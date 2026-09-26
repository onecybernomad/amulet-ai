use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::DrivingSession;

/// Create a new driving session.
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<DrivingSession> {
    let session = sqlx::query_as::<_, DrivingSession>(
        r#"
        INSERT INTO driving_sessions (user_id)
        VALUES ($1)
        RETURNING id, user_id, started_at, ended_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created driving session: id={}", session.id);
    Ok(session)
}

/// Get driving session reports for a user.
pub async fn get_reports(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(Uuid, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>> {
    let reports = sqlx::query_as::<_, (Uuid, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT id, started_at, ended_at
        FROM driving_sessions
        WHERE user_id = $1
        ORDER BY started_at DESC
        LIMIT 100
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(reports)
}

/// Insert a driving event (e.g., hard braking, acceleration).
pub async fn insert_event(
    pool: &PgPool,
    session_id: Uuid,
    event_type: &str,
    latitude: f64,
    longitude: f64,
    severity: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO driving_events (session_id, event_type, latitude, longitude, severity, recorded_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
    )
    .bind(session_id)
    .bind(event_type)
    .bind(latitude)
    .bind(longitude)
    .bind(severity)
    .execute(pool)
    .await?;

    tracing::info!("Inserted driving event for session {}: type={}", session_id, event_type);
    Ok(())
}

/// Update a driving session (e.g., set ended_at).
pub async fn update_session(
    pool: &PgPool,
    session_id: Uuid,
    ended_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<DrivingSession> {
    let session = sqlx::query_as::<_, DrivingSession>(
        r#"
        UPDATE driving_sessions
        SET ended_at = $2
        WHERE id = $1
        RETURNING id, user_id, started_at, ended_at
        "#,
    )
    .bind(session_id)
    .bind(ended_at)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated driving session: id={}", session_id);
    Ok(session)
}
