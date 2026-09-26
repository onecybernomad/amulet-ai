use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::Subscription;

/// Create a new subscription record.
pub async fn create_subscription(
    pool: &PgPool,
    user_id: Uuid,
    stripe_customer_id: &str,
    stripe_subscription_id: Option<&str>,
    tier: &str,
) -> Result<Subscription> {
    let subscription = sqlx::query_as::<_, Subscription>(
        r#"
        INSERT INTO subscriptions (user_id, stripe_customer_id, stripe_subscription_id, tier, status)
        VALUES ($1, $2, $3, $4, 'active')
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(user_id)
    .bind(stripe_customer_id)
    .bind(stripe_subscription_id)
    .bind(tier)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created subscription: id={}, tier={}", subscription.id, subscription.tier);
    Ok(subscription)
}

/// Get a user's subscription.
pub async fn get_subscription(pool: &PgPool, user_id: Uuid) -> Result<Option<Subscription>> {
    let subscription = sqlx::query_as::<_, Subscription>(
        r#"
        SELECT id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        FROM subscriptions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(subscription)
}

/// Update the subscription tier.
pub async fn update_tier(
    pool: &PgPool,
    subscription_id: Uuid,
    tier: &str,
) -> Result<Subscription> {
    let subscription = sqlx::query_as::<_, Subscription>(
        r#"
        UPDATE subscriptions
        SET tier = $2
        WHERE id = $1
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(subscription_id)
    .bind(tier)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated subscription {} tier to {}", subscription_id, tier);
    Ok(subscription)
}

/// Cancel a subscription.
pub async fn cancel_subscription(pool: &PgPool, subscription_id: Uuid) -> Result<Subscription> {
    let subscription = sqlx::query_as::<_, Subscription>(
        r#"
        UPDATE subscriptions
        SET status = 'cancelled'
        WHERE id = $1
        RETURNING id, user_id, stripe_customer_id, stripe_subscription_id, tier, status, created_at
        "#,
    )
    .bind(subscription_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("Cancelled subscription: id={}", subscription_id);
    Ok(subscription)
}
