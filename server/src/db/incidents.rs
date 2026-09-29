use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::Incident;

/// Create a new incident record.
pub async fn create_incident(
    pool: &SqlitePool,
    circle_id: Uuid,
    user_id: Option<Uuid>,
    incident_type: &str,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<Incident> {
    let row = sqlx::query(
        r#"
        INSERT INTO incidents (circle_id, user_id, incident_type, latitude, longitude, status)
        VALUES (?, ?, ?, ?, ?, 'active')
        RETURNING id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
        "#,
    )
    .bind(circle_id.to_string())
    .bind(user_id.map(|v| v.to_string()))
    .bind(incident_type)
    .bind(latitude)
    .bind(longitude)
    .fetch_one(pool)
    .await?;

    let incident = Incident {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: row.try_get::<String, _>("user_id").ok().and_then(|s| Uuid::parse_str(&s).ok()),
        incident_type: row.try_get("incident_type")?,
        latitude: row.try_get("latitude").ok(),
        longitude: row.try_get("longitude").ok(),
        status: row.try_get("status")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Created incident: id={}, type={}", incident.id, incident.incident_type);
    Ok(incident)
}

/// Get incidents for a circle, optionally filtered by status.
pub async fn get_incidents(
    pool: &SqlitePool,
    circle_id: Uuid,
    status: Option<&str>,
) -> Result<Vec<Incident>> {
    let rows = if let Some(s) = status {
        sqlx::query(
            r#"
            SELECT id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
            FROM incidents
            WHERE circle_id = ? AND status = ?
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(circle_id.to_string())
        .bind(s)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
            FROM incidents
            WHERE circle_id = ?
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(circle_id.to_string())
        .fetch_all(pool)
        .await?
    };

    let mut incidents = Vec::new();
    for row in rows {
        incidents.push(Incident {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            user_id: row.try_get::<String, _>("user_id").ok().and_then(|s| Uuid::parse_str(&s).ok()),
            incident_type: row.try_get("incident_type")?,
            latitude: row.try_get("latitude").ok(),
            longitude: row.try_get("longitude").ok(),
            status: row.try_get("status")?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(incidents)
}

/// Update the status of an incident.
pub async fn update_incident_status(
    pool: &SqlitePool,
    incident_id: Uuid,
    status: &str,
) -> Result<Incident> {
    let row = sqlx::query(
        r#"
        UPDATE incidents
        SET status = ?
        WHERE id = ?
        RETURNING id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
        "#,
    )
    .bind(status)
    .bind(incident_id.to_string())
    .fetch_one(pool)
    .await?;

    let incident = Incident {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: row.try_get::<String, _>("user_id").ok().and_then(|s| Uuid::parse_str(&s).ok()),
        incident_type: row.try_get("incident_type")?,
        latitude: row.try_get("latitude").ok(),
        longitude: row.try_get("longitude").ok(),
        status: row.try_get("status")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Updated incident {} status to {}", incident_id, status);
    Ok(incident)
}

/// Log a response action to an incident.
pub async fn log_incident_response(
    pool: &SqlitePool,
    incident_id: Uuid,
    responder_id: Uuid,
    action: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO incident_responses (incident_id, responder_id, action, responded_at)
        VALUES (?, ?, ?, datetime('now'))
        "#,
    )
    .bind(incident_id.to_string())
    .bind(responder_id.to_string())
    .bind(action)
    .execute(pool)
    .await?;

    tracing::info!("Logged response for incident {}: action={}", incident_id, action);
    Ok(())
}
