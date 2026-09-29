use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Stricter rate limiter for auth routes.
/// Uses a per-endpoint sliding window with configurable limits.
#[derive(Clone)]
pub struct AuthRateLimiter {
    buckets: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
    max_requests: u64,
    window: Duration,
}

impl AuthRateLimiter {
    pub fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Check if a request is allowed under the rate limit.
    /// Returns true if allowed, false if rate limited.
    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(key.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }

    /// Get the number of remaining requests for a key.
    pub async fn remaining(&self, key: &str) -> u64 {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(key.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > self.window {
            self.max_requests
        } else {
            self.max_requests.saturating_sub(entry.0)
        }
    }

    /// Get the seconds until the window resets for a key.
    pub async fn reset_after(&self, key: &str) -> u64 {
        let buckets = self.buckets.lock().await;
        if let Some((_, start)) = buckets.get(key) {
            let elapsed = Instant::now().duration_since(*start);
            self.window.saturating_sub(elapsed).as_secs().max(1)
        } else {
            1
        }
    }
}

/// Extract client IP from headers (x-forwarded-for or x-real-ip).
fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Rate limiting middleware for auth routes.
/// Applies stricter limits than the global rate limiter.
pub async fn auth_rate_limit_middleware(
    State(limiter): State<AuthRateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = client_ip(req.headers());
    let path = req.uri().path().to_string();

    // Use IP + path as the key for per-endpoint rate limiting
    let key = format!("{}:{}", ip, path);

    if limiter.check(&key).await {
        let remaining = limiter.remaining(&key).await;
        let reset_after = limiter.reset_after(&key).await;

        let mut response = next.run(req).await;
        response.headers_mut().insert(
            "x-ratelimit-remaining",
            remaining.to_string().parse().unwrap(),
        );
        response.headers_mut().insert(
            "x-ratelimit-reset",
            reset_after.to_string().parse().unwrap(),
        );
        Ok(response)
    } else {
        let reset_after = limiter.reset_after(&key).await;
        let mut response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from("Too many requests"))
            .unwrap();
        response.headers_mut().insert(
            "retry-after",
            reset_after.to_string().parse().unwrap(),
        );
        Ok(response)
    }
}
