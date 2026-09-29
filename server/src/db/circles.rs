use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::{Circle, User};

/// Create a new circle with an auto-generated invite code.
pub async fn create_circle(pool: &SqlitePool, name: &str, owner_id: Uuid) -> Result<Circle> {
    let invite_code = generate_invite_code();
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO circles (id, name, invite_code, owner_id, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(&invite_code)
    .bind(owner_id.to_string())
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    // Add owner as member
    sqlx::query(
        r#"
        INSERT INTO circle_members (circle_id, user_id, role)
        VALUES (?, ?, 'owner')
        "#,
    )
    .bind(&id)
    .bind(owner_id.to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let circle = Circle {
        id: Uuid::parse_str(&id)?,
        name: name.to_string(),
        invite_code,
        owner_id,
        created_at: crate::db::parse_datetime(&now)?,
    };

    tracing::info!("Created circle: id={}, name={}", circle.id, circle.name);
    Ok(circle)
}

/// Fetch a circle by UUID.
pub async fn get_circle(pool: &SqlitePool, id: Uuid) -> Result<Option<Circle>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, invite_code, owner_id, created_at
        FROM circles
        WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let circle = Circle {
                id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                name: row.try_get("name")?,
                invite_code: row.try_get("invite_code")?,
                owner_id: Uuid::parse_str(&row.try_get::<String, _>("owner_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
            };
            Ok(Some(circle))
        }
        None => Ok(None),
    }
}

/// Look up a circle by its invite code.
pub async fn get_circle_by_invite_code(pool: &SqlitePool, invite_code: &str) -> Result<Option<Circle>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, invite_code, owner_id, created_at
        FROM circles
        WHERE invite_code = ?
        "#,
    )
    .bind(invite_code)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let circle = Circle {
                id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                name: row.try_get("name")?,
                invite_code: row.try_get("invite_code")?,
                owner_id: Uuid::parse_str(&row.try_get::<String, _>("owner_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
            };
            Ok(Some(circle))
        }
        None => Ok(None),
    }
}

/// Get all circles that a user is a member of (including owned circles).
pub async fn get_user_circles(pool: &SqlitePool, user_id: Uuid) -> Result<Vec<Circle>> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.name, c.invite_code, c.owner_id, c.created_at
        FROM circles c
        JOIN circle_members cm ON cm.circle_id = c.id
        WHERE cm.user_id = ?
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut circles = Vec::new();
    for row in rows {
        circles.push(Circle {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            name: row.try_get("name")?,
            invite_code: row.try_get("invite_code")?,
            owner_id: Uuid::parse_str(&row.try_get::<String, _>("owner_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(circles)
}

/// Add a user to a circle via invite code.
pub async fn join_circle(pool: &SqlitePool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO circle_members (circle_id, user_id)
        VALUES (?, ?)
        ON CONFLICT (circle_id, user_id) DO NOTHING
        "#,
    )
    .bind(circle_id.to_string())
    .bind(user_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("User {} joined circle {}", user_id, circle_id);
    Ok(())
}

/// Remove a user from a circle.
pub async fn leave_circle(pool: &SqlitePool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM circle_members
        WHERE circle_id = ? AND user_id = ?
        "#,
    )
    .bind(circle_id.to_string())
    .bind(user_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("User {} left circle {}", user_id, circle_id);
    Ok(())
}

/// Get all members of a circle.
pub async fn get_circle_members(pool: &SqlitePool, circle_id: Uuid) -> Result<Vec<User>> {
    let rows = sqlx::query(
        r#"
        SELECT u.id, u.email, u.display_name, u.created_at
        FROM circle_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.circle_id = ?
        ORDER BY u.display_name
        "#,
    )
    .bind(circle_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut members = Vec::new();
    for row in rows {
        members.push(User {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            phone: None,
            avatar_url: None,
            subscription_tier: String::new(),
            password_hash: String::new(),
            failed_login_attempts: 0,
            locked_until: None,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
            updated_at: None,
        });
    }

    Ok(members)
}

/// Regenerate the invite code for a circle.
pub async fn regenerate_invite_code(pool: &SqlitePool, circle_id: Uuid) -> Result<String> {
    let new_code = generate_invite_code();

    sqlx::query(
        r#"
        UPDATE circles
        SET invite_code = ?
        WHERE id = ?
        "#,
    )
    .bind(&new_code)
    .bind(circle_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Regenerated invite code for circle: id={}", circle_id);
    Ok(new_code)
}

fn generate_invite_code() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
}
