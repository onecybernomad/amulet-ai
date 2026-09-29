use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;

/// Record a response to an incident (acknowledge, respond, resolve).
pub async fn record_incident_response(
    pool: &SqlitePool,
    incident_id: Uuid,
    user_id: Uuid,
    action: &str,
    note: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO incident_responses (id, incident_id, user_id, action, note)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(incident_id.to_string())
    .bind(user_id.to_string())
    .bind(action)
    .bind(note)
    .execute(pool)
    .await?;

    tracing::info!(
        "Recorded incident response: incident={} user={} action={}",
        incident_id,
        user_id,
        action
    );
    Ok(())
}

/// Get all responses for an incident.
pub async fn get_incident_responses(
    pool: &SqlitePool,
    incident_id: Uuid,
) -> Result<Vec<IncidentResponseRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, incident_id, user_id, action, note, created_at
        FROM incident_responses
        WHERE incident_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(incident_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut responses = Vec::new();
    for row in rows {
        responses.push(IncidentResponseRow {
            id: row.try_get("id")?,
            incident_id: row.try_get("incident_id")?,
            user_id: row.try_get("user_id")?,
            action: row.try_get("action")?,
            note: row.try_get("note")?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(responses)
}

/// Update incident status.
pub async fn update_incident_status(
    pool: &SqlitePool,
    incident_id: Uuid,
    status: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE incidents SET status = ? WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(incident_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Updated incident {} status to {}", incident_id, status);
    Ok(())
}

// ── Row Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct IncidentResponseRow {
    pub id: String,
    pub incident_id: String,
    pub user_id: String,
    pub action: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}
