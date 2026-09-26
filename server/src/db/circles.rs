use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::{Circle, User};

/// Create a new circle with an auto-generated invite code.
pub async fn create_circle(pool: &PgPool, name: &str, owner_id: Uuid) -> Result<Circle> {
    let invite_code = generate_invite_code();

    let circle = sqlx::query_as::<_, Circle>(
        r#"
        INSERT INTO circles (name, invite_code, owner_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, invite_code, owner_id, created_at
        "#,
    )
    .bind(name)
    .bind(&invite_code)
    .bind(owner_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created circle: id={}, name={}", circle.id, circle.name);
    Ok(circle)
}

/// Fetch a circle by UUID.
pub async fn get_circle(pool: &PgPool, id: Uuid) -> Result<Option<Circle>> {
    let circle = sqlx::query_as::<_, Circle>(
        r#"
        SELECT id, name, invite_code, owner_id, created_at
        FROM circles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(circle)
}

/// Look up a circle by its invite code.
pub async fn get_circle_by_invite_code(pool: &PgPool, invite_code: &str) -> Result<Option<Circle>> {
    let circle = sqlx::query_as::<_, Circle>(
        r#"
        SELECT id, name, invite_code, owner_id, created_at
        FROM circles
        WHERE invite_code = $1
        "#,
    )
    .bind(invite_code)
    .fetch_optional(pool)
    .await?;

    Ok(circle)
}

/// Add a user to a circle via invite code.
pub async fn join_circle(pool: &PgPool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO circle_members (circle_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (circle_id, user_id) DO NOTHING
        "#,
    )
    .bind(circle_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!("User {} joined circle {}", user_id, circle_id);
    Ok(())
}

/// Remove a user from a circle.
pub async fn leave_circle(pool: &PgPool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM circle_members
        WHERE circle_id = $1 AND user_id = $2
        "#,
    )
    .bind(circle_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!("User {} left circle {}", user_id, circle_id);
    Ok(())
}

/// Get all members of a circle.
pub async fn get_circle_members(pool: &PgPool, circle_id: Uuid) -> Result<Vec<User>> {
    let members = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.email, u.display_name, u.created_at
        FROM circle_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.circle_id = $1
        ORDER BY u.display_name
        "#,
    )
    .bind(circle_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}

/// Regenerate the invite code for a circle.
pub async fn regenerate_invite_code(pool: &PgPool, circle_id: Uuid) -> Result<String> {
    let new_code = generate_invite_code();

    sqlx::query(
        r#"
        UPDATE circles
        SET invite_code = $2
        WHERE id = $1
        "#,
    )
    .bind(circle_id)
    .bind(&new_code)
    .execute(pool)
    .await?;

    tracing::info!("Regenerated invite code for circle: id={}", circle_id);
    Ok(new_code)
}

fn generate_invite_code() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
}
