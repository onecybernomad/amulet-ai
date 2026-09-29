use axum::{
    extract::{Request, FromRequestParts, State},
    http::{request::Parts, header, StatusCode},
    middleware::Next,
    response::Response,
};
use axum::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::sync::Arc;
use uuid::Uuid;
use crate::services::redis::RedisClient;

/// JWT claims structure.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(default)]
    pub token_type: String, // "access" or "refresh"
    #[serde(default)]
    pub family_id: String, // UUID linking tokens in a rotation chain
}

/// Shared state for the auth middleware.
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub redis: Option<RedisClient>,
}

/// Routes that don't require authentication.
fn is_public_path(path: &str) -> bool {
    path == "/health"
        || path.starts_with("/api/auth/login")
        || path.starts_with("/api/auth/register")
        || path.starts_with("/api/auth/refresh")
        || path.starts_with("/api/auth/logout")
        || path.starts_with("/api/auth/otp/send")
        || path.starts_with("/api/auth/otp/verify")
        || path.starts_with("/api/auth/password/reset")
        || path.starts_with("/api/auth/email/verify")
        || path == "/ws"
}

/// JWT validation middleware.
/// Skips validation for public routes (auth endpoints, health check).
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    tracing::debug!("[AUTH] Incoming request to: {}", path);

    // Skip auth for public routes
    if is_public_path(&path) {
        tracing::debug!("[AUTH] Public route, skipping auth: {}", path);
        return Ok(next.run(req).await);
    }

    // Allow WebSocket upgrade requests through — auth happens on first message
    let is_upgrade = req
        .headers()
        .get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);
    if is_upgrade {
        tracing::debug!("[AUTH] WebSocket upgrade, deferring auth to first message: {}", path);
        return Ok(next.run(req).await);
    }

    let auth_header = match req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        Some(h) => h,
        None => {
            tracing::warn!("[AUTH] Missing Authorization header for: {}", path);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            tracing::warn!("[AUTH] Invalid Authorization header format for: {}", path);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    tracing::debug!("[AUTH] Token length: {}", token.len());

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("[AUTH] Token decode failed for {}: {:?}", path, e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    tracing::debug!("[AUTH] Token decoded successfully, sub: {}", token_data.claims.sub);

    // Ensure this is an access token, not a refresh token
    if token_data.claims.token_type != "access" {
        tracing::warn!("[AUTH] Invalid token type: {}", token_data.claims.token_type);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if token is blacklisted (only if Redis is available)
    if let Some(ref redis) = state.redis {
        match redis.is_token_blacklisted(token).await {
            Ok(true) => {
                tracing::warn!("[AUTH] Token is blacklisted");
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(false) => {}
            Err(e) => {
                tracing::error!("[AUTH] Redis error checking blacklist: {:?}", e);
                // Fail open - don't block requests if Redis is down
            }
        }
    }

    let mut req = req;
    req.extensions_mut().insert(token_data.claims);
    tracing::debug!("[AUTH] Auth successful for: {}", path);
    Ok(next.run(req).await)
}

/// Extractor for the authenticated user's ID from JWT claims.
pub struct AuthUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(Self { user_id })
    }
}
