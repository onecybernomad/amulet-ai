use crate::db::local_cache::LocalCache;
use crate::models::Subscription;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub features: Vec<String>,
    pub max_members: i32,
    pub max_circles: i32,
}

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub plan_id: String,
    pub payment_method_id: String,
    pub interval: String, // "monthly" | "yearly"
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Get all available subscription plans.
#[tauri::command]
pub async fn get_plans() -> Result<Vec<SubscriptionPlan>, String> {
    info!("Getting subscription plans");
    let plans = vec![
        SubscriptionPlan {
            id: "free".into(),
            name: "Free".into(),
            price_monthly: 0.0,
            price_yearly: 0.0,
            features: vec![
                "1 Circle".into(),
                "Up to 3 Members".into(),
                "Basic Location Tracking".into(),
                "SOS Alerts".into(),
            ],
            max_members: 3,
            max_circles: 1,
        },
        SubscriptionPlan {
            id: "premium".into(),
            name: "Premium".into(),
            price_monthly: 9.99,
            price_yearly: 99.99,
            features: vec![
                "Unlimited Circles".into(),
                "Up to 10 Members".into(),
                "Real-time Location".into(),
                "Geofencing".into(),
                "Medication Tracking".into(),
                "Chat".into(),
                "Tile Integration".into(),
            ],
            max_members: 10,
            max_circles: 5,
        },
        SubscriptionPlan {
            id: "family".into(),
            name: "Family".into(),
            price_monthly: 19.99,
            price_yearly: 199.99,
            features: vec![
                "Unlimited Circles".into(),
                "Unlimited Members".into(),
                "All Premium Features".into(),
                "Priority Support".into(),
                "Incident Detection".into(),
                "Driving Reports".into(),
            ],
            max_members: 999,
            max_circles: 999,
        },
    ];
    Ok(plans)
}

/// Subscribe to a plan.
#[tauri::command]
pub async fn subscribe(
    req: SubscribeRequest,
    cache: State<'_, LocalCache>,
) -> Result<Subscription, String> {
    info!(plan_id = %req.plan_id, interval = %req.interval, "Creating subscription");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let subscription = Subscription {
        id: Uuid::new_v4().to_string(),
        user_id: user.id,
        plan_id: req.plan_id,
        stripe_subscription_id: None,
        stripe_customer_id: None,
        status: "active".into(),
        current_period_start: chrono::Utc::now(),
        current_period_end: chrono::Utc::now()
            + if req.interval == "yearly" {
                chrono::Duration::days(365)
            } else {
                chrono::Duration::days(30)
            },
        cancel_at_period_end: false,
        created_at: chrono::Utc::now(),
    };

    cache.insert_subscription(&subscription).map_err(|e| e.to_string())?;
    debug!(subscription_id = %subscription.id, "Subscription created");
    Ok(subscription)
}

/// Cancel the current subscription.
#[tauri::command]
pub async fn cancel_subscription(
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!("Cancelling subscription");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    cache.cancel_subscription(&user.id).map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the current user's subscription.
#[tauri::command]
pub async fn get_current_subscription(
    cache: State<'_, LocalCache>,
) -> Result<Option<Subscription>, String> {
    info!("Getting current subscription");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;
    let sub = cache.get_active_subscription(&user.id).map_err(|e| e.to_string())?;
    Ok(sub)
}
