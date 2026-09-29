use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::DrivingSession;

/// Create a new driving session.
pub async fn create_session(pool: &SqlitePool, user_id: Uuid) -> Result<DrivingSession> {
    let row = sqlx::query(
        r#"
        INSERT INTO driving_sessions (user_id)
        VALUES (?)
        RETURNING id, user_id, started_at, ended_at
        "#,
    )
    .bind(user_id.to_string())
    .fetch_one(pool)
    .await?;

    let session = DrivingSession {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        started_at: crate::db::parse_datetime(&row.try_get::<String, _>("started_at")?)?,
        ended_at: row.try_get::<String, _>("ended_at").ok().and_then(|s| crate::db::parse_datetime(&s).ok()),
        distance_km: None,
        max_speed: None,
        avg_speed: None,
        harsh_braking_count: None,
        rapid_acceleration_count: None,
        phone_usage_count: None,
    };

    tracing::info!("Created driving session: id={}", session.id);
    Ok(session)
}

/// Get driving session reports for a user.
pub async fn get_reports(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<Vec<(Uuid, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>> {
    let rows = sqlx::query(
        r#"
        SELECT id, started_at, ended_at
        FROM driving_sessions
        WHERE user_id = ?
        ORDER BY started_at DESC
        LIMIT 100
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut reports = Vec::new();
    for row in rows {
        let id: Uuid = Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?;
        let started_at: chrono::DateTime<chrono::Utc> = crate::db::parse_datetime(&row.try_get::<String, _>("started_at")?)?;
        let ended_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get::<String, _>("ended_at").ok().and_then(|s| crate::db::parse_datetime(&s).ok());
        reports.push((id, started_at, ended_at));
    }

    Ok(reports)
}

/// Insert a driving event (e.g., hard braking, acceleration).
pub async fn insert_event(
    pool: &SqlitePool,
    session_id: Uuid,
    event_type: &str,
    latitude: f64,
    longitude: f64,
    severity: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO driving_events (session_id, event_type, latitude, longitude, severity, recorded_at)
        VALUES (?, ?, ?, ?, ?, datetime('now'))
        "#,
    )
    .bind(session_id.to_string())
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
    pool: &SqlitePool,
    session_id: Uuid,
    ended_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<DrivingSession> {
    let row = sqlx::query(
        r#"
        UPDATE driving_sessions
        SET ended_at = ?
        WHERE id = ?
        RETURNING id, user_id, started_at, ended_at
        "#,
    )
    .bind(ended_at.map(|dt| dt.to_rfc3339()))
    .bind(session_id.to_string())
    .fetch_one(pool)
    .await?;

    let session = DrivingSession {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        started_at: crate::db::parse_datetime(&row.try_get::<String, _>("started_at")?)?,
        ended_at: row.try_get::<String, _>("ended_at").ok().and_then(|s| crate::db::parse_datetime(&s).ok()),
        distance_km: None,
        max_speed: None,
        avg_speed: None,
        harsh_braking_count: None,
        rapid_acceleration_count: None,
        phone_usage_count: None,
    };

    tracing::info!("Updated driving session: id={}", session_id);
    Ok(session)
}

/// Get a driving session with all its events.
pub async fn get_session_with_events(
    pool: &SqlitePool,
    session_id: Uuid,
) -> Result<DrivingSessionDetail> {
    let session_row = sqlx::query(
        r#"
        SELECT id, user_id, started_at, ended_at, distance_km, max_speed, avg_speed,
               harsh_braking_count, rapid_acceleration_count, phone_usage_count
        FROM driving_sessions
        WHERE id = ?
        "#,
    )
    .bind(session_id.to_string())
    .fetch_one(pool)
    .await?;

    let session = DrivingSession {
        id: Uuid::parse_str(&session_row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&session_row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        started_at: crate::db::parse_datetime(&session_row.try_get::<String, _>("started_at")?)?,
        ended_at: session_row.try_get::<String, _>("ended_at").ok().and_then(|s| crate::db::parse_datetime(&s).ok()),
        distance_km: session_row.try_get("distance_km").ok(),
        max_speed: session_row.try_get("max_speed").ok(),
        avg_speed: session_row.try_get("avg_speed").ok(),
        harsh_braking_count: session_row.try_get("harsh_braking_count").ok(),
        rapid_acceleration_count: session_row.try_get("rapid_acceleration_count").ok(),
        phone_usage_count: session_row.try_get("phone_usage_count").ok(),
    };

    let event_rows = sqlx::query(
        r#"
        SELECT event_type, latitude, longitude, severity, recorded_at
        FROM driving_events
        WHERE session_id = ?
        ORDER BY recorded_at ASC
        "#,
    )
    .bind(session_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in event_rows {
        events.push(DrivingEvent {
            event_type: row.try_get("event_type")?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            severity: row.try_get("severity")?,
            recorded_at: crate::db::parse_datetime(&row.try_get::<String, _>("recorded_at")?)?,
        });
    }

    // Calculate safety score
    let safety_score = calculate_safety_score(&session, &events);

    Ok(DrivingSessionDetail {
        session,
        events,
        safety_score,
    })
}

/// Get driving sessions for a user within a time range.
pub async fn get_sessions_in_range(
    pool: &SqlitePool,
    user_id: Uuid,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<DrivingSession>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, started_at, ended_at
        FROM driving_sessions
        WHERE user_id = ? AND started_at >= ? AND started_at <= ?
        ORDER BY started_at DESC
        LIMIT 100
        "#,
    )
    .bind(user_id.to_string())
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(DrivingSession {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            started_at: crate::db::parse_datetime(&row.try_get::<String, _>("started_at")?)?,
            ended_at: row.try_get::<String, _>("ended_at").ok().and_then(|s| crate::db::parse_datetime(&s).ok()),
            distance_km: None,
            max_speed: None,
            avg_speed: None,
            harsh_braking_count: None,
            rapid_acceleration_count: None,
            phone_usage_count: None,
        });
    }

    Ok(sessions)
}

/// Get driving statistics for a user.
pub async fn get_driving_stats(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<DrivingStats> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total_sessions,
            COALESCE(SUM(distance_km), 0) as total_distance,
            COALESCE(MAX(max_speed), 0) as max_speed,
            COALESCE(AVG(avg_speed), 0) as avg_speed,
            COALESCE(SUM(harsh_braking_count), 0) as total_harsh_braking,
            COALESCE(SUM(rapid_acceleration_count), 0) as total_rapid_accel,
            COALESCE(SUM(phone_usage_count), 0) as total_phone_use
        FROM driving_sessions
        WHERE user_id = ?
        "#,
    )
    .bind(user_id.to_string())
    .fetch_one(pool)
    .await?;

    Ok(DrivingStats {
        total_sessions: row.try_get("total_sessions").unwrap_or(0),
        total_distance: row.try_get("total_distance").unwrap_or(0.0),
        max_speed: row.try_get("max_speed").unwrap_or(0.0),
        avg_speed: row.try_get("avg_speed").unwrap_or(0.0),
        total_harsh_braking: row.try_get("total_harsh_braking").unwrap_or(0),
        total_rapid_accel: row.try_get("total_rapid_accel").unwrap_or(0),
        total_phone_use: row.try_get("total_phone_use").unwrap_or(0),
    })
}

/// Calculate safety score for a driving session.
fn calculate_safety_score(session: &DrivingSession, events: &[DrivingEvent]) -> i64 {
    let mut score = 100i64;

    // Deduct for events
    for event in events {
        match event.event_type.as_str() {
            "hard_braking" => score -= 10,
            "rapid_acceleration" => score -= 5,
            "phone_use" => score -= 15,
            "speeding" => score -= 8,
            _ => {}
        }
    }

    // Deduct for session-level counts
    score -= session.harsh_braking_count.unwrap_or(0) * 5;
    score -= session.rapid_acceleration_count.unwrap_or(0) * 3;
    score -= session.phone_usage_count.unwrap_or(0) * 10;

    score.max(0)
}

// ── Row Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct DrivingSessionDetail {
    pub session: DrivingSession,
    pub events: Vec<DrivingEvent>,
    pub safety_score: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DrivingEvent {
    pub event_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub severity: f64,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DrivingStats {
    pub total_sessions: i64,
    pub total_distance: f64,
    pub max_speed: f64,
    pub avg_speed: f64,
    pub total_harsh_braking: i64,
    pub total_rapid_accel: i64,
    pub total_phone_use: i64,
}
