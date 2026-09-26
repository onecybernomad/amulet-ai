use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::Incident;

/// Create a new incident record.
pub async fn create_incident(
    pool: &PgPool,
    circle_id: Uuid,
    user_id: Option<Uuid>,
    incident_type: &str,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<Incident> {
    let incident = sqlx::query_as::<_, Incident>(
        r#"
        INSERT INTO incidents (circle_id, user_id, incident_type, latitude, longitude, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        RETURNING id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
        "#,
    )
    .bind(circle_id)
    .bind(user_id)
    .bind(incident_type)
    .bind(latitude)
    .bind(longitude)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created incident: id={}, type={}", incident.id, incident.incident_type);
    Ok(incident)
}

/// Get incidents for a circle, optionally filtered by status.
pub async fn get_incidents(
    pool: &PgPool,
    circle_id: Uuid,
    status: Option<&str>,
) -> Result<Vec<Incident>> {
    let incidents = if let Some(s) = status {
        sqlx::query_as::<_, Incident>(
            r#"
            SELECT id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
            FROM incidents
            WHERE circle_id = $1 AND status = $2
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(circle_id)
        .bind(s)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Incident>(
            r#"
            SELECT id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
            FROM incidents
            WHERE circle_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(circle_id)
        .fetch_all(pool)
        .await?
    };

    Ok(incidents)
}

/// Update the status of an incident.
pub async fn update_incident_status(
    pool: &PgPool,
    incident_id: Uuid,
    status: &str,
) -> Result<Incident> {
    let incident = sqlx::query_as::<_, Incident>(
        r#"
        UPDATE incidents
        SET status = $2
        WHERE id = $1
        RETURNING id, circle_id, user_id, incident_type, latitude, longitude, status, created_at
        "#,
    )
    .bind(incident_id)
    .bind(status)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated incident {} status to {}", incident_id, status);
    Ok(incident)
}

/// Log a response action to an incident.
pub async fn log_incident_response(
    pool: &PgPool,
    incident_id: Uuid,
    responder_id: Uuid,
    action: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO incident_responses (incident_id, responder_id, action, responded_at)
        VALUES ($1, $2, $3, NOW())
        "#,
    )
    .bind(incident_id)
    .bind(responder_id)
    .bind(action)
    .execute(pool)
    .await?;

    tracing::info!("Logged response for incident {}: action={}", incident_id, action);
    Ok(())
}
