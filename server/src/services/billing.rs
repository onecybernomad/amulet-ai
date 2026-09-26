use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Stripe checkout session response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSessionResponse {
    pub session_id: String,
    pub checkout_url: String,
}

/// Create a Stripe checkout session for a subscription.
pub async fn create_checkout_session(
    stripe_secret_key: &str,
    customer_email: &str,
    price_id: &str,
    success_url: &str,
    cancel_url: &str,
) -> Result<CheckoutSessionResponse> {
    tracing::info!("Creating checkout session for customer: {}", customer_email);

    let client = reqwest::Client::new();

    let params = [
        ("customer_email", customer_email),
        ("line_items[0][price]", price_id),
        ("line_items[0][quantity]", "1"),
        ("mode", "subscription"),
        ("success_url", success_url),
        ("cancel_url", cancel_url),
    ];

    let response = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(stripe_secret_key)
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        tracing::error!("Stripe API error: {} - {}", status, body);
        anyhow::bail!("Stripe API error: {}", status);
    }

    let session: serde_json::Value = response.json().await?;
    let session_id = session["id"].as_str().unwrap_or("").to_string();
    let checkout_url = session["url"].as_str().unwrap_or("").to_string();

    tracing::info!("Created checkout session: {}", session_id);

    Ok(CheckoutSessionResponse {
        session_id,
        checkout_url,
    })
}

/// Cancel a Stripe subscription.
pub async fn cancel_subscription(
    stripe_secret_key: &str,
    subscription_id: &str,
) -> Result<()> {
    tracing::info!("Cancelling subscription: {}", subscription_id);

    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "https://api.stripe.com/v1/subscriptions/{}",
            subscription_id
        ))
        .bearer_auth(stripe_secret_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        tracing::error!("Stripe API error: {} - {}", status, body);
        anyhow::bail!("Stripe API error: {}", status);
    }

    tracing::info!("Cancelled subscription: {}", subscription_id);
    Ok(())
}

/// Get a Stripe subscription's details.
pub async fn get_subscription(
    stripe_secret_key: &str,
    subscription_id: &str,
) -> Result<serde_json::Value> {
    tracing::info!("Fetching subscription: {}", subscription_id);

    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "https://api.stripe.com/v1/subscriptions/{}",
            subscription_id
        ))
        .bearer_auth(stripe_secret_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        tracing::error!("Stripe API error: {} - {}", status, body);
        anyhow::bail!("Stripe API error: {}", status);
    }

    let subscription: serde_json::Value = response.json().await?;
    Ok(subscription)
}
