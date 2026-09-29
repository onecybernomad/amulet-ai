use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::Subscription;

/// Create a new subscription record.
pub async fn create_subscription(
    pool: &SqlitePool,
    user_id: Uuid,
    stripe_customer_id: &str,
    stripe_subscription_id: Option<&str>,
    tier: &str,
) -> Result<Subscription> {
    let row = sqlx::query(
        r#"
        INSERT INTO subscriptions (user_id, stripe_customer_id, stripe_subscription_id, tier, status)
        VALUES (?, ?, ?, ?, 'active')
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(user_id.to_string())
    .bind(stripe_customer_id)
    .bind(stripe_subscription_id)
    .bind(tier)
    .fetch_one(pool)
    .await?;

    let subscription = Subscription {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        stripe_customer_id: row.try_get("stripe_customer_id")?,
        stripe_subscription_id: row.try_get("stripe_subscription_id").ok(),
        tier: row.try_get("tier")?,
        status: row.try_get("status")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Created subscription: id={}, tier={}", subscription.id, subscription.tier);
    Ok(subscription)
}

/// Get a user's subscription.
pub async fn get_subscription(pool: &SqlitePool, user_id: Uuid) -> Result<Option<Subscription>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        FROM subscriptions
        WHERE user_id = ?
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let subscription = Subscription {
                id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
                stripe_customer_id: row.try_get("stripe_customer_id")?,
                stripe_subscription_id: row.try_get("stripe_subscription_id").ok(),
                tier: row.try_get("tier")?,
                status: row.try_get("status")?,
                created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
            };
            Ok(Some(subscription))
        }
        None => Ok(None),
    }
}

/// Update the subscription tier.
pub async fn update_tier(
    pool: &SqlitePool,
    subscription_id: Uuid,
    tier: &str,
) -> Result<Subscription> {
    let row = sqlx::query(
        r#"
        UPDATE subscriptions
        SET tier = ?
        WHERE id = ?
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(tier)
    .bind(subscription_id.to_string())
    .fetch_one(pool)
    .await?;

    let subscription = Subscription {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        stripe_customer_id: row.try_get("stripe_customer_id")?,
        stripe_subscription_id: row.try_get("stripe_subscription_id").ok(),
        tier: row.try_get("tier")?,
        status: row.try_get("status")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Updated subscription {} tier to {}", subscription_id, tier);
    Ok(subscription)
}

/// Cancel a subscription.
pub async fn cancel_subscription(pool: &SqlitePool, subscription_id: Uuid) -> Result<Subscription> {
    let row = sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = 'cancelled'
        WHERE id = ?
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(subscription_id.to_string())
    .fetch_one(pool)
    .await?;

    let subscription = Subscription {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        user_id: Uuid::parse_str(&row.try_get::<String, _>("user_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        stripe_customer_id: row.try_get("stripe_customer_id")?,
        stripe_subscription_id: row.try_get("stripe_subscription_id").ok(),
        tier: row.try_get("tier")?,
        status: row.try_get("status")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Cancelled subscription: id={}", subscription_id);
    Ok(subscription)
}
