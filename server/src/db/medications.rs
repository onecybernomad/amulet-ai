use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::Medication;

/// Create a new medication record.
pub async fn create_medication(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    dosage: &str,
    schedule: &str,
) -> Result<Medication> {
    let medication = sqlx::query_as::<_, Medication>(
        r#"
        INSERT INTO medications (user_id, name, dosage, schedule)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, name, dosage, schedule, created_at
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(dosage)
    .bind(schedule)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created medication: id={}, name={}", medication.id, medication.name);
    Ok(medication)
}

/// Get all medications for a user.
pub async fn get_medications(pool: &PgPool, user_id: Uuid) -> Result<Vec<Medication>> {
    let medications = sqlx::query_as::<_, Medication>(
        r#"
        SELECT id, user_id, name, dosage, schedule, created_at
        FROM medications
        WHERE user_id = $1
        ORDER BY name
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(medications)
}

/// Update a medication.
pub async fn update_medication(
    pool: &PgPool,
    medication_id: Uuid,
    name: &str,
    dosage: &str,
    schedule: &str,
) -> Result<Medication> {
    let medication = sqlx::query_as::<_, Medication>(
        r#"
        UPDATE medications
        SET name = $2, dosage = $3, schedule = $4
        WHERE id = $1
        RETURNING id, user_id, name, dosage, schedule, created_at
        "#,
    )
    .bind(medication_id)
    .bind(name)
    .bind(dosage)
    .bind(schedule)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated medication: id={}", medication_id);
    Ok(medication)
}

/// Delete a medication.
pub async fn delete_medication(pool: &PgPool, medication_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM medications
        WHERE id = $1
        "#,
    )
    .bind(medication_id)
    .execute(pool)
    .await?;

    tracing::info!("Deleted medication: id={}", medication_id);
    Ok(())
}

/// Log medication adherence (taken/missed).
pub async fn log_adherence(
    pool: &PgPool,
    medication_id: Uuid,
    user_id: Uuid,
    taken: bool,
    notes: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO medication_adherence (medication_id, user_id, taken, notes, logged_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(medication_id)
    .bind(user_id)
    .bind(taken)
    .bind(notes)
    .execute(pool)
    .await?;

    tracing::info!("Logged adherence for medication {}: taken={}", medication_id, taken);
    Ok(())
}

/// Get adherence records for a user's medications.
pub async fn get_adherence(
    pool: &PgPool,
    user_id: Uuid,
    medication_id: Option<Uuid>,
) -> Result<Vec<(Uuid, Uuid, bool, Option<String>, chrono::DateTime<chrono::Utc>)>> {
    let records = sqlx::query_as::<_, (Uuid, Uuid, bool, Option<String>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT medication_id, user_id, taken, notes, logged_at
        FROM medication_adherence
        WHERE user_id = $1
        AND ($2::uuid IS NULL OR medication_id = $2)
        ORDER BY logged_at DESC
        LIMIT 200
        "#,
    )
    .bind(user_id)
    .bind(medication_id)
    .fetch_all(pool)
    .await?;

    Ok(records)
}
