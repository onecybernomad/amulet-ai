use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::Medication;

/// Create a new medication record.
pub async fn create_medication(
    pool: &SqlitePool,
    user_id: Uuid,
    name: &str,
    dosage: &str,
    schedule: &str,
) -> Result<Medication> {
    let row = sqlx::query(
        r#"
        INSERT INTO medications (user_id, name, dosage, schedule)
        VALUES (?, ?, ?, ?)
        RETURNING id, user_id, name, dosage, schedule, created_at
        "#,
    )
    .bind(user_id.to_string())
    .bind(name)
    .bind(dosage)
    .bind(schedule)
    .fetch_one(pool)
    .await?;

    let medication = Medication {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        name: row.try_get("name")?,
        dosage: row.try_get("dosage")?,
        schedule: row.try_get("schedule")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Created medication: id={}, name={}", medication.id, medication.name);
    Ok(medication)
}

/// Get all medications for a user.
pub async fn get_medications(pool: &SqlitePool, user_id: Uuid) -> Result<Vec<Medication>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, name, dosage, schedule, created_at
        FROM medications
        WHERE user_id = ?
        ORDER BY name
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut medications = Vec::new();
    for row in rows {
        medications.push(Medication {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            name: row.try_get("name")?,
            dosage: row.try_get("dosage")?,
            schedule: row.try_get("schedule")?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(medications)
}

/// Update a medication.
pub async fn update_medication(
    pool: &SqlitePool,
    medication_id: Uuid,
    name: &str,
    dosage: &str,
    schedule: &str,
) -> Result<Medication> {
    let row = sqlx::query(
        r#"
        UPDATE medications
        SET name = ?, dosage = ?, schedule = ?
        WHERE id = ?
        RETURNING id, user_id, name, dosage, schedule, created_at
        "#,
    )
    .bind(name)
    .bind(dosage)
    .bind(schedule)
    .bind(medication_id.to_string())
    .fetch_one(pool)
    .await?;

    let medication = Medication {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        name: row.try_get("name")?,
        dosage: row.try_get("dosage")?,
        schedule: row.try_get("schedule")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Updated medication: id={}", medication_id);
    Ok(medication)
}

/// Delete a medication.
pub async fn delete_medication(pool: &SqlitePool, medication_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM medications
        WHERE id = ?
        "#,
    )
    .bind(medication_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Deleted medication: id={}", medication_id);
    Ok(())
}

/// Log medication adherence (taken/missed).
pub async fn log_adherence(
    pool: &SqlitePool,
    medication_id: Uuid,
    user_id: Uuid,
    taken: bool,
    notes: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO medication_adherence (medication_id, user_id, taken, notes, logged_at)
        VALUES (?, ?, ?, ?, datetime('now'))
        "#,
    )
    .bind(medication_id.to_string())
    .bind(user_id.to_string())
    .bind(taken)
    .bind(notes)
    .execute(pool)
    .await?;

    tracing::info!("Logged adherence for medication {}: taken={}", medication_id, taken);
    Ok(())
}

/// Get adherence records for a user's medications.
pub async fn get_adherence(
    pool: &SqlitePool,
    user_id: Uuid,
    medication_id: Option<Uuid>,
) -> Result<Vec<(Uuid, Uuid, bool, Option<String>, chrono::DateTime<chrono::Utc>)>> {
    let rows = sqlx::query(
        r#"
        SELECT medication_id, user_id, taken, notes, logged_at
        FROM medication_adherence
        WHERE user_id = ?
        AND (? IS NULL OR medication_id = ?)
        ORDER BY logged_at DESC
        LIMIT 200
        "#,
    )
    .bind(user_id.to_string())
    .bind(medication_id.map(|v| v.to_string()))
    .bind(medication_id.map(|v| v.to_string()))
    .fetch_all(pool)
    .await?;

    let mut records = Vec::new();
    for row in rows {
        let med_id: Uuid = Uuid::parse_str(&row.try_get::<String, _>("medication_id")?).map_err(|e| anyhow::anyhow!("{}", e))?;
        let uid: Uuid = Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?;
        let taken: bool = row.try_get("taken")?;
        let notes: Option<String> = row.try_get("notes").ok();
        let logged_at: chrono::DateTime<chrono::Utc> = crate::db::parse_datetime(&row.try_get::<String, _>("logged_at")?)?;
        records.push((med_id, uid, taken, notes, logged_at));
    }

    Ok(records)
}

/// Get adherence statistics for a medication.
pub async fn get_adherence_stats(
    pool: &SqlitePool,
    medication_id: Uuid,
) -> Result<AdherenceStats> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN taken = 1 THEN 1 ELSE 0 END) as taken_count,
            SUM(CASE WHEN taken = 0 THEN 1 ELSE 0 END) as missed_count
        FROM medication_adherence
        WHERE medication_id = ?
        "#,
    )
    .bind(medication_id.to_string())
    .fetch_one(pool)
    .await?;

    let total: i64 = row.try_get("total").unwrap_or(0);
    let taken_count: i64 = row.try_get("taken_count").unwrap_or(0);
    let missed_count: i64 = row.try_get("missed_count").unwrap_or(0);
    let rate = if total > 0 {
        (taken_count as f64 / total as f64 * 100.0) as i64
    } else {
        0
    };

    Ok(AdherenceStats {
        total,
        taken: taken_count,
        missed: missed_count,
        rate,
    })
}

/// Get current adherence streak for a medication.
pub async fn get_adherence_streak(
    pool: &SqlitePool,
    medication_id: Uuid,
) -> Result<i64> {
    let rows = sqlx::query(
        r#"
        SELECT taken
        FROM medication_adherence
        WHERE medication_id = ?
        ORDER BY logged_at DESC
        "#,
    )
    .bind(medication_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut streak = 0i64;
    for row in rows {
        let taken: bool = row.try_get("taken")?;
        if taken {
            streak += 1;
        } else {
            break;
        }
    }

    Ok(streak)
}

/// Get upcoming medication schedules for a user.
pub async fn get_upcoming_schedules(
    pool: &SqlitePool,
    user_id: Uuid,
    hours_ahead: i32,
) -> Result<Vec<UpcomingSchedule>> {
    let medications = get_medications(pool, user_id).await?;
    let mut schedules = Vec::new();
    let now = chrono::Utc::now();

    for med in medications {
        // Parse schedule string (comma-separated times)
        for time_str in med.schedule.split(',') {
            let time_str = time_str.trim();
            if time_str.is_empty() {
                continue;
            }

            // Parse "HH:MM" format
            let parts: Vec<&str> = time_str.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let hour: u32 = parts[0].parse().unwrap_or(0);
            let minute: u32 = parts[1].parse().unwrap_or(0);

            // Create a DateTime for today at this time
            let scheduled_time = now
                .date_naive()
                .and_hms_opt(hour, minute, 0)
                .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));

            if let Some(scheduled) = scheduled_time {
                let diff = scheduled.signed_duration_since(now);
                if diff.num_seconds() > 0 && diff.num_hours() <= hours_ahead as i64 {
                    schedules.push(UpcomingSchedule {
                        medication_id: med.id,
                        medication_name: med.name.clone(),
                        dosage: med.dosage.clone(),
                        scheduled_time: scheduled,
                        status: "pending".to_string(),
                    });
                }
            }
        }
    }

    // Sort by scheduled time
    schedules.sort_by(|a, b| a.scheduled_time.cmp(&b.scheduled_time));
    Ok(schedules)
}

// ── Row Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct AdherenceStats {
    pub total: i64,
    pub taken: i64,
    pub missed: i64,
    pub rate: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpcomingSchedule {
    pub medication_id: Uuid,
    pub medication_name: String,
    pub dosage: String,
    pub scheduled_time: DateTime<Utc>,
    pub status: String,
}
