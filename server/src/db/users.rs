use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::User;

/// Insert a new user row and return the created entity.
pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str, display_name: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, display_name, created_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created user: id={}, email={}", user.id, user.email);
    Ok(user)
}

/// Fetch a user by their email address.
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, display_name, created_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Fetch a user by their UUID.
pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, display_name, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Update a user's display_name (or other mutable fields).
pub async fn update_user(pool: &PgPool, id: Uuid, display_name: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET display_name = $2
        WHERE id = $1
        RETURNING id, email, display_name, created_at
        "#,
    )
    .bind(id)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated user: id={}", id);
    Ok(user)
}
