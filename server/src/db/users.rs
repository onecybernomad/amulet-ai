use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::User;

/// Insert a new user row and return the created entity.
pub async fn create_user(pool: &SqlitePool, email: &str, password_hash: &str, display_name: &str) -> Result<User> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let user = User {
        id: Uuid::parse_str(&id)?,
        email: email.to_string(),
        display_name: display_name.to_string(),
        phone: None,
        avatar_url: None,
        subscription_tier: "free".to_string(),
        password_hash: password_hash.to_string(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: crate::db::parse_datetime(&now)?,
        updated_at: None,
    };

    tracing::info!("Created user: id={}, email={}", user.id, user.email);
    Ok(user)
}

/// Fetch a user by their email address.
pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, display_name, phone, avatar_url, subscription_tier, password_hash, failed_login_attempts, locked_until, created_at
        FROM users
        WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let password_hash: String = row.try_get("password_hash")?;
            
            if password_hash.is_empty() {
                return Err(anyhow::anyhow!("Empty password hash"));
            }
            
            let id_str: String = row.try_get("id")?;
            let created_at_str: String = row.try_get("created_at")?;
            let created_at = crate::db::parse_datetime(&created_at_str)?;
            let locked_until_str: Option<String> = row.try_get("locked_until").ok();
            let locked_until = locked_until_str.and_then(|s| crate::db::parse_datetime(&s).ok());
            let user = User {
                id: Uuid::parse_str(&id_str)?,
                email: row.try_get("email")?,
                display_name: row.try_get("display_name")?,
                phone: row.try_get("phone").ok(),
                avatar_url: row.try_get("avatar_url").ok(),
                subscription_tier: row.try_get("subscription_tier").unwrap_or_else(|_| "free".to_string()),
                password_hash,
                failed_login_attempts: row.try_get("failed_login_attempts").unwrap_or(0),
                locked_until,
                created_at,
                updated_at: None,
            };
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

/// Fetch a user by their UUID.
pub async fn get_user_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, display_name, phone, avatar_url, subscription_tier, password_hash, failed_login_attempts, locked_until, created_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let password_hash: String = row.try_get("password_hash")?;
            
            if password_hash.is_empty() {
                return Err(anyhow::anyhow!("Empty password hash"));
            }
            
            let id_str: String = row.try_get("id")?;
            let created_at_str: String = row.try_get("created_at")?;
            let created_at = crate::db::parse_datetime(&created_at_str)?;
            let locked_until_str: Option<String> = row.try_get("locked_until").ok();
            let locked_until = locked_until_str.and_then(|s| crate::db::parse_datetime(&s).ok());
            let user = User {
                id: Uuid::parse_str(&id_str)?,
                email: row.try_get("email")?,
                display_name: row.try_get("display_name")?,
                phone: row.try_get("phone").ok(),
                avatar_url: row.try_get("avatar_url").ok(),
                subscription_tier: row.try_get("subscription_tier").unwrap_or_else(|_| "free".to_string()),
                password_hash,
                failed_login_attempts: row.try_get("failed_login_attempts").unwrap_or(0),
                locked_until,
                created_at,
                updated_at: None,
            };
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

/// Update a user's profile fields.
pub async fn update_user(pool: &SqlitePool, id: Uuid, display_name: &str, phone: Option<&str>, avatar_url: Option<&str>) -> Result<User> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE users
        SET display_name = ?, phone = ?, avatar_url = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(display_name)
    .bind(phone)
    .bind(avatar_url)
    .bind(&now)
    .bind(id.to_string())
    .execute(pool)
    .await?;

    get_user_by_id(pool, id).await?.ok_or_else(|| anyhow::anyhow!("User not found after update"))
}

/// Increment failed login attempts for a user.
/// Returns the new attempt count.
pub async fn increment_failed_login(pool: &SqlitePool, id: Uuid) -> Result<i64> {
    sqlx::query(
        r#"
        UPDATE users
        SET failed_login_attempts = failed_login_attempts + 1
        WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .execute(pool)
    .await?;

    let row = sqlx::query(
        r#"
        SELECT failed_login_attempts FROM users WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .fetch_one(pool)
    .await?;

    let attempts: i64 = row.try_get("failed_login_attempts")?;
    Ok(attempts)
}

/// Lock a user account until the specified time.
pub async fn lock_user(pool: &SqlitePool, id: Uuid, until: DateTime<Utc>) -> Result<()> {
    let until_str = until.to_rfc3339();
    sqlx::query(
        r#"
        UPDATE users
        SET locked_until = ?
        WHERE id = ?
        "#,
    )
    .bind(&until_str)
    .bind(id.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

/// Reset failed login attempts and clear lockout (on successful login).
pub async fn reset_failed_logins(pool: &SqlitePool, id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET failed_login_attempts = 0, locked_until = NULL
        WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .execute(pool)
    .await?;
    Ok(())
}
