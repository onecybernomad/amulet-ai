mod config;
mod db;
mod middleware;
mod realtime;
mod services;
mod types;

use std::sync::Arc;
use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;
use sqlx::Row;

use crate::config::Config;
use crate::db::Database;
use crate::middleware::auth::{auth_middleware, AuthState, AuthUser, Claims};
use crate::middleware::auth_rate_limit::{auth_rate_limit_middleware, AuthRateLimiter};
use crate::middleware::rate_limit::{rate_limit_middleware, RateLimiter};
use crate::realtime::{handle_connection, RealtimeHub};
use crate::types::*;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub hub: RealtimeHub,
    pub config: Config,
    pub redis: Option<crate::services::redis::RedisClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Starting Amulet AI server...");

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    let db = match Database::connect(&config.database_url).await {
        Ok(db) => {
            tracing::info!("Database connected: {}", config.database_url);
            db
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    let hub = RealtimeHub::new(db.pool.clone());

    // Initialize Redis (optional - app works without it)
    let redis = match crate::services::redis::RedisClient::new(&config.redis_url).await {
        Ok(r) => {
            tracing::info!("Redis connected: {}", config.redis_url);
            Some(r)
        }
        Err(e) => {
            tracing::warn!("Redis not available: {:?}. Token blacklist disabled.", e);
            None
        }
    };

    let app_state = Arc::new(AppState {
        db,
        hub,
        config: config.clone(),
        redis: redis.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let rate_limiter = RateLimiter::new(100, std::time::Duration::from_secs(60));
    let auth_rate_limiter = AuthRateLimiter::new(5, std::time::Duration::from_secs(60));

    let auth_state = Arc::new(AuthState {
        jwt_secret: config.jwt_secret.clone(),
        redis: redis.clone(),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/otp/send", post(send_otp))
        .route("/api/auth/otp/verify", post(verify_otp))
        .route("/api/auth/password/reset", post(request_password_reset))
        .route("/api/auth/password/reset/confirm", post(confirm_password_reset))
        .route("/api/auth/email/verify", post(request_email_verification))
        .route("/api/auth/email/verify/confirm", post(confirm_email_verification))
        .route("/api/users/me", get(get_current_user))
        .route("/api/users/me", put(update_current_user))
        .route("/api/circles", post(create_circle))
        .route("/api/circles", get(list_user_circles))
        .route("/api/circles/:id", get(get_circle))
        .route("/api/circles/:id/join", post(join_circle))
        .route("/api/circles/:id/leave", post(leave_circle))
        .route("/api/circles/:id/members", get(get_circle_members))
        .route("/api/circles/:id/invite-code", post(regenerate_invite_code))
        .route("/api/circles/:id/locations", get(get_member_locations))
        .route("/api/circles/:id/locations/history/:user_id", get(get_location_history))
        .route("/api/circles/:id/places", post(create_place))
        .route("/api/circles/:id/places", get(get_places))
        .route("/api/places/:id", put(update_place))
        .route("/api/places/:id", delete(delete_place))
        .route("/api/circles/:id/incidents", post(create_incident))
        .route("/api/circles/:id/incidents", get(get_incidents))
        .route("/api/incidents/:id/status", put(update_incident_status))
        .route("/api/incidents/:id/respond", post(log_incident_response))
        .route("/api/medications", post(create_medication))
        .route("/api/medications", get(get_medications))
        .route("/api/medications/:id", put(update_medication))
        .route("/api/medications/:id", delete(delete_medication))
        .route("/api/medications/:id/adherence", post(log_adherence))
        .route("/api/medications/adherence", get(get_adherence))
        .route("/api/circles/:id/rooms", post(create_room))
        .route("/api/circles/:id/rooms", get(get_rooms))
        .route("/api/rooms/:id/messages", get(get_messages))
        .route("/api/rooms/:id/messages", post(send_message))
        .route("/api/rooms/:id/read", post(mark_read))
        .route("/api/driving/sessions", post(create_driving_session))
        .route("/api/driving/sessions/:id", put(update_driving_session))
        .route("/api/driving/sessions/:id/events", post(insert_driving_event))
        .route("/api/driving/reports", get(get_driving_reports))
        .route("/api/subscriptions", post(create_subscription))
        .route("/api/subscriptions", get(get_subscription))
        .route("/api/subscriptions/:id/tier", put(update_subscription_tier))
        .route("/api/subscriptions/:id/cancel", post(cancel_subscription))
        .route("/api/tiles", post(link_tile))
        .route("/api/tiles/:id/ring", post(ring_tile))
        .route("/api/tiles/:id/locate", get(locate_tile))
        .route("/api/circles/:id/geofence-events", get(get_circle_geofence_events))
        .route("/api/incidents/:id/acknowledge", post(acknowledge_incident))
        .route("/api/incidents/:id/responses", get(get_incident_responses))
        .route("/api/sensors/readings", post(submit_sensor_reading))
        .route("/api/sensors/status", get(get_sensor_status))
        .route("/api/medications/:id/adherence/stats", get(get_medication_adherence_stats))
        .route("/api/medications/:id/adherence/streak", get(get_medication_adherence_streak))
        .route("/api/medications/upcoming", get(get_upcoming_medication_schedules))
        .route("/api/driving/sessions/:id/detail", get(get_driving_session_detail))
        .route("/api/driving/stats", get(get_driving_stats))
        .route("/api/dispatch/incident/:id", post(trigger_emergency_dispatch))
        .route("/api/dispatch/:id/status", get(get_dispatch_status))
        .route("/api/dispatch/:id/cancel", post(cancel_dispatch))
        .route("/api/tiles/:id/community-finds", get(get_tile_community_finds))
        .route("/api/tiles/community-find", post(submit_community_find))
        .route("/api/assistance/roadside", post(request_roadside_assistance))
        .route("/api/assistance/medical", post(request_medical_advice))
        .route("/api/assistance/:id/status", get(get_assistance_status))
        .route("/api/assistance/:id/cancel", post(cancel_assistance))
        .route("/api/assistance/providers", get(get_nearby_providers))
        .route("/ws", get(ws_handler))
        .layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            auth_rate_limiter.clone(),
            auth_rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.bind_address()).await?;
    tracing::info!("Server listening on {}", config.bind_address());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

// ─── Auth Handler Functions ──────────────────────────────────────────────────

async fn health_check() -> impl IntoResponse {
    json_response(StatusCode::OK, ApiResponse::success("healthy"))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let email = req["email"].as_str().unwrap_or("").trim().to_lowercase();
    let password = req["password"].as_str().unwrap_or("");
    let display_name = req["display_name"].as_str().unwrap_or("").trim();

    if email.is_empty() || password.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Email and password are required"));
    }
    if !email.contains('@') {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid email address"));
    }
    if password.len() < 8 {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Password must be at least 8 characters"));
    }

    match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(_)) => {
            return json_response(StatusCode::CONFLICT, ApiResponse::<()>::error("User with this email already exists"));
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("Database error during registration: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Registration failed"));
        }
    }

    let password_hash = match hash_password(password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Password hashing failed: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Registration failed"));
        }
    };

    match db::users::create_user(&state.db.pool, &email, &password_hash, display_name).await {
        Ok(user) => {
            let family_id = Uuid::new_v4().to_string();
            let access_token = generate_access_token(&user.id.to_string(), &state.config.jwt_secret, state.config.access_token_ttl_minutes, &family_id);
            let refresh_token = generate_refresh_token(&user.id.to_string(), &state.config.jwt_secret, state.config.refresh_token_ttl_days, &family_id);
            let user_profile = UserProfile::from(user);
            json_response(StatusCode::OK, ApiResponse::success(AuthResponse {
                token: access_token,
                refresh_token,
                user: user_profile,
            }))
        }
        Err(e) => {
            tracing::error!("Registration failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error(format!("Registration failed: {}", e)))
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::debug!("Login request body: {:?}", req);
    let email = req["email"].as_str().unwrap_or("").trim().to_lowercase();
    let password = req["password"].as_str().unwrap_or("");
    tracing::debug!("Extracted email: '{}', password_len: {}", email, password.len());

    if email.is_empty() || password.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Email and password are required"));
    }

    let user = match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let _ = verify_password(password, "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG");
            return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid credentials"));
        }
        Err(e) => {
            tracing::error!("Login failed: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Login failed"));
        }
    };

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        let now = chrono::Utc::now();
        if now < locked_until {
            let remaining = (locked_until - now).num_minutes();
            tracing::warn!("Login attempt on locked account: {} (locked for {} more minutes)", email, remaining);
            return json_response(
                StatusCode::FORBIDDEN,
                ApiResponse::<()>::error(format!("Account is locked. Try again in {} minutes.", remaining)),
            );
        }
    }

    match verify_password(password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!("Failed login attempt for email: {} (attempt {})", email, user.failed_login_attempts + 1);
            
            // Increment failed login attempts
            let attempts = match db::users::increment_failed_login(&state.db.pool, user.id).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("Failed to increment login attempts: {:?}", e);
                    return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Login failed"));
                }
            };

            // Lock account after 5 failed attempts
            if attempts >= 5 {
                let lock_duration = chrono::Duration::minutes(15);
                let locked_until = chrono::Utc::now() + lock_duration;
                if let Err(e) = db::users::lock_user(&state.db.pool, user.id, locked_until).await {
                    tracing::error!("Failed to lock user account: {:?}", e);
                }
                tracing::warn!("Account locked for user {} after {} failed attempts", email, attempts);
                return json_response(
                    StatusCode::FORBIDDEN,
                    ApiResponse::<()>::error("Account locked due to too many failed attempts. Try again in 15 minutes."),
                );
            }

            return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid credentials"));
        }
        Err(e) => {
            tracing::error!("Password verification failed: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Login failed"));
        }
    }

    // Reset failed login attempts on successful login
    if let Err(e) = db::users::reset_failed_logins(&state.db.pool, user.id).await {
        tracing::error!("Failed to reset login attempts: {:?}", e);
    }

    let family_id = Uuid::new_v4().to_string();
    let access_token = generate_access_token(&user.id.to_string(), &state.config.jwt_secret, state.config.access_token_ttl_minutes, &family_id);
    let refresh_token = generate_refresh_token(&user.id.to_string(), &state.config.jwt_secret, state.config.refresh_token_ttl_days, &family_id);
    let user_profile = UserProfile::from(user);

    json_response(StatusCode::OK, ApiResponse::success(AuthResponse {
        token: access_token,
        refresh_token,
        user: user_profile,
    }))
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let refresh_token = req["refresh_token"].as_str().unwrap_or("");
    if refresh_token.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Refresh token is required"));
    }

    let token_data = match decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(_) => return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid refresh token")),
    };

    if token_data.claims.token_type != "refresh" {
        return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid token type"));
    }

    let user_id = match Uuid::parse_str(&token_data.claims.sub) {
        Ok(id) => id,
        Err(_) => return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid token")),
    };

    let family_id = &token_data.claims.family_id;

    // Token rotation: if Redis is available, check for token reuse
    if let Some(ref redis) = state.redis {
        // Check if this refresh token has already been used (reuse detection)
        let token_hash = format!("{:x}", md5::compute(refresh_token));
        match redis.get_refresh_token(&user_id.to_string()).await {
            Ok(Some(stored_hash)) => {
                if stored_hash != token_hash {
                    // Token reuse detected! Invalidate the entire family
                    tracing::warn!("Token reuse detected for user {}. Invalidating family {}.", user_id, family_id);
                    let _ = redis.remove_token_family(family_id).await;
                    let _ = redis.remove_refresh_token(&user_id.to_string()).await;
                    return json_response(
                        StatusCode::UNAUTHORIZED,
                        ApiResponse::<()>::error("Token reuse detected. Please login again."),
                    );
                }
            }
            Ok(None) => {
                // No stored token - this might be an old token or a replay
                tracing::warn!("No stored refresh token found for user {} during rotation", user_id);
            }
            Err(e) => {
                tracing::error!("Redis error during token rotation: {:?}", e);
            }
        }
    }

    match db::users::get_user_by_id(&state.db.pool, user_id).await {
        Ok(Some(user)) => {
            // Generate new token pair with the same family_id
            let new_access = generate_access_token(&user.id.to_string(), &state.config.jwt_secret, state.config.access_token_ttl_minutes, family_id);
            let new_refresh = generate_refresh_token(&user.id.to_string(), &state.config.jwt_secret, state.config.refresh_token_ttl_days, family_id);

            // Store the new refresh token hash for reuse detection
            if let Some(ref redis) = state.redis {
                let new_token_hash = format!("{:x}", md5::compute(&new_refresh));
                let ttl_secs = (state.config.refresh_token_ttl_days * 24 * 60 * 60) as u64;
                if let Err(e) = redis.store_refresh_token(&user_id.to_string(), &new_token_hash, ttl_secs).await {
                    tracing::error!("Failed to store refresh token hash: {:?}", e);
                }
            }

            json_response(StatusCode::OK, ApiResponse::success(RefreshTokenResponse {
                token: new_access,
                refresh_token: new_refresh,
            }))
        }
        Ok(None) => json_response(StatusCode::NOT_FOUND, ApiResponse::<()>::error("User not found")),
        Err(e) => {
            tracing::error!("Token refresh failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Token refresh failed"))
        }
    }
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    // If Redis is available, blacklist the provided tokens
    if let Some(ref redis) = state.redis {
        // Blacklist access token if provided
        if let Some(token) = req["access_token"].as_str() {
            if !token.is_empty() {
                // Access tokens have 15 min TTL, so blacklist for that long
                let ttl = (state.config.access_token_ttl_minutes * 60) as u64;
                if let Err(e) = redis.blacklist_token(token, ttl).await {
                    tracing::error!("Failed to blacklist access token: {:?}", e);
                }
            }
        }

        // Blacklist refresh token if provided
        if let Some(token) = req["refresh_token"].as_str() {
            if !token.is_empty() {
                let ttl = (state.config.refresh_token_ttl_days * 24 * 60 * 60) as u64;
                if let Err(e) = redis.blacklist_token(token, ttl).await {
                    tracing::error!("Failed to blacklist refresh token: {:?}", e);
                }
            }
        }

        // Remove refresh token from rotation tracking
        if let Some(user_id) = req["user_id"].as_str() {
            let _ = redis.remove_refresh_token(user_id).await;
        }
    }

    json_response(StatusCode::OK, ApiResponse::success("Logged out"))
}

async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PasswordResetRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Valid email is required"));
    }

    let user = match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // Don't reveal if email exists
            return json_response(StatusCode::OK, ApiResponse::success("If the email exists, a password reset link has been sent"));
        }
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return json_response(StatusCode::OK, ApiResponse::success("If the email exists, a password reset link has been sent"));
        }
    };

    // Generate reset token
    let token = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();

    // Store token in database
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO password_resets (token, user_id, expires_at)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&token)
    .bind(user.id.to_string())
    .bind(&expires_at)
    .execute(&state.db.pool)
    .await
    {
        tracing::error!("Failed to store password reset token: {:?}", e);
        return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
    }

    // In production, send email with reset link
    tracing::info!("Password reset token for {}: {} (expires: {})", email, token, expires_at);

    json_response(StatusCode::OK, ApiResponse::success("If the email exists, a password reset link has been sent"))
}

async fn confirm_password_reset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PasswordResetConfirmRequest>,
) -> impl IntoResponse {
    let token = req.token.trim();
    let new_password = req.new_password.trim();

    if token.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Reset token is required"));
    }
    if new_password.len() < 8 {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Password must be at least 8 characters"));
    }

    // Look up token
    let row = match sqlx::query(
        r#"
        SELECT user_id, expires_at, used FROM password_resets WHERE token = ?
        "#,
    )
    .bind(token)
    .fetch_optional(&state.db.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
        }
    };

    let (user_id_str, expires_at_str, used): (String, String, i64) = match row {
        Some(r) => (
            r.try_get("user_id").unwrap_or_default(),
            r.try_get("expires_at").unwrap_or_default(),
            r.try_get("used").unwrap_or(1),
        ),
        None => {
            return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid or expired reset token"));
        }
    };

    if used != 0 {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Reset token has already been used"));
    }

    let expires_at = match crate::db::parse_datetime(&expires_at_str) {
        Ok(dt) => dt,
        Err(_) => {
            return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid reset token"));
        }
    };

    if chrono::Utc::now() > expires_at {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Reset token has expired"));
    }

    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => {
            return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid reset token"));
        }
    };

    // Hash new password
    let password_hash = match hash_password(new_password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Password hashing failed: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
        }
    };

    // Update password
    if let Err(e) = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&password_hash)
        .bind(&user_id_str)
        .execute(&state.db.pool)
        .await
    {
        tracing::error!("Failed to update password: {:?}", e);
        return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
    }

    // Mark token as used
    if let Err(e) = sqlx::query("UPDATE password_resets SET used = 1 WHERE token = ?")
        .bind(token)
        .execute(&state.db.pool)
        .await
    {
        tracing::error!("Failed to mark token as used: {:?}", e);
    }

    tracing::info!("Password reset successful for user {}", user_id);
    json_response(StatusCode::OK, ApiResponse::success("Password has been reset successfully"))
}

async fn request_email_verification(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmailVerifyRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Valid email is required"));
    }

    let user = match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return json_response(StatusCode::OK, ApiResponse::success("If the email exists, a verification link has been sent"));
        }
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return json_response(StatusCode::OK, ApiResponse::success("If the email exists, a verification link has been sent"));
        }
    };

    // Generate verification token
    let token = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();

    // Store token
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO email_verifications (token, user_id, email, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&token)
    .bind(user.id.to_string())
    .bind(&email)
    .bind(&expires_at)
    .execute(&state.db.pool)
    .await
    {
        tracing::error!("Failed to store verification token: {:?}", e);
        return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
    }

    tracing::info!("Email verification token for {}: {} (expires: {})", email, token, expires_at);

    json_response(StatusCode::OK, ApiResponse::success("If the email exists, a verification link has been sent"))
}

async fn confirm_email_verification(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmailVerifyConfirmRequest>,
) -> impl IntoResponse {
    let token = req.token.trim();
    if token.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Verification token is required"));
    }

    // Look up token
    let row = match sqlx::query(
        r#"
        SELECT user_id, email, expires_at, verified FROM email_verifications WHERE token = ?
        "#,
    )
    .bind(token)
    .fetch_optional(&state.db.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to process request"));
        }
    };

    let (user_id_str, email, expires_at_str, verified): (String, String, String, i64) = match row {
        Some(r) => (
            r.try_get("user_id").unwrap_or_default(),
            r.try_get("email").unwrap_or_default(),
            r.try_get("expires_at").unwrap_or_default(),
            r.try_get("verified").unwrap_or(1),
        ),
        None => {
            return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid or expired verification token"));
        }
    };

    if verified != 0 {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Email has already been verified"));
    }

    let expires_at = match crate::db::parse_datetime(&expires_at_str) {
        Ok(dt) => dt,
        Err(_) => {
            return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid verification token"));
        }
    };

    if chrono::Utc::now() > expires_at {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Verification token has expired"));
    }

    // Mark token as verified
    if let Err(e) = sqlx::query("UPDATE email_verifications SET verified = 1 WHERE token = ?")
        .bind(token)
        .execute(&state.db.pool)
        .await
    {
        tracing::error!("Failed to mark token as verified: {:?}", e);
    }

    tracing::info!("Email verification successful for {} ({})", email, user_id_str);
    json_response(StatusCode::OK, ApiResponse::success("Email has been verified successfully"))
}

async fn send_otp(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OtpSendRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(user)) => {
            let secret = format!("{}:{}", state.config.jwt_secret, user.id);
            let totp = generate_totp(&secret);
            tracing::info!("Generated OTP for user {}: {}", user.id, totp);
            json_response(StatusCode::OK, ApiResponse::success("If the email exists, an OTP has been sent"))
        }
        Ok(None) => json_response(StatusCode::OK, ApiResponse::success("If the email exists, an OTP has been sent")),
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            json_response(StatusCode::OK, ApiResponse::success("If the email exists, an OTP has been sent"))
        }
    }
}

async fn verify_otp(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OtpVerifyRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    let code = req.code.trim();

    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Invalid OTP format"));
    }

    let user = match db::users::get_user_by_email(&state.db.pool, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid credentials")),
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Verification failed"));
        }
    };

    let secret = format!("{}:{}", state.config.jwt_secret, user.id);
    if !verify_totp(&secret, code) {
        return json_response(StatusCode::UNAUTHORIZED, ApiResponse::<()>::error("Invalid or expired OTP"));
    }

    let family_id = Uuid::new_v4().to_string();
    let access_token = generate_access_token(&user.id.to_string(), &state.config.jwt_secret, state.config.access_token_ttl_minutes, &family_id);
    let refresh_token = generate_refresh_token(&user.id.to_string(), &state.config.jwt_secret, state.config.refresh_token_ttl_days, &family_id);
    let user_profile = UserProfile::from(user);

    json_response(StatusCode::OK, ApiResponse::success(AuthResponse {
        token: access_token,
        refresh_token,
        user: user_profile,
    }))
}

async fn get_current_user(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::users::get_user_by_id(&state.db.pool, auth_user.user_id).await {
        Ok(Some(user)) => json_response(StatusCode::OK, ApiResponse::success(UserProfile::from(user))),
        Ok(None) => json_response(StatusCode::NOT_FOUND, ApiResponse::<()>::error("User not found")),
        Err(e) => {
            tracing::error!("Get current user failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get user"))
        }
    }
}

async fn update_current_user(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let display_name = req.display_name.as_deref().unwrap_or("").trim();
    if display_name.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ApiResponse::<()>::error("Display name cannot be empty"));
    }

    match db::users::update_user(
        &state.db.pool,
        auth_user.user_id,
        display_name,
        req.phone.as_deref(),
        req.avatar_url.as_deref(),
    )
    .await
    {
        Ok(user) => json_response(StatusCode::OK, ApiResponse::success(UserProfile::from(user))),
        Err(e) => {
            tracing::error!("Update user failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update user"))
        }
    }
}

// ─── Circle Handler Functions ─────────────────────────────────────────────────

async fn create_circle(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<CreateCircleRequest>,
) -> impl IntoResponse {
    match db::circles::create_circle(&state.db.pool, &req.name, auth_user.user_id).await {
        Ok(circle) => json_response(StatusCode::OK, ApiResponse::success(circle)),
        Err(e) => {
            tracing::error!("Create circle failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create circle"))
        }
    }
}

async fn list_user_circles(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::circles::get_user_circles(&state.db.pool, auth_user.user_id).await {
        Ok(circles) => json_response(StatusCode::OK, ApiResponse::success(circles)),
        Err(e) => {
            tracing::error!("List circles failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to list circles"))
        }
    }
}

async fn get_circle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::get_circle(&state.db.pool, id).await {
        Ok(Some(circle)) => json_response(StatusCode::OK, ApiResponse::success(circle)),
        Ok(None) => json_response(StatusCode::NOT_FOUND, ApiResponse::<()>::error("Circle not found")),
        Err(e) => {
            tracing::error!("Get circle failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get circle"))
        }
    }
}

async fn join_circle(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    match db::circles::join_circle(&state.db.pool, id, auth_user.user_id).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Joined circle")),
        Err(e) => {
            tracing::error!("Join circle failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to join circle"))
        }
    }
}

async fn leave_circle(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::leave_circle(&state.db.pool, id, auth_user.user_id).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Left circle")),
        Err(e) => {
            tracing::error!("Leave circle failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to leave circle"))
        }
    }
}

async fn get_circle_members(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::get_circle_members(&state.db.pool, id).await {
        Ok(members) => json_response(StatusCode::OK, ApiResponse::success(members)),
        Err(e) => {
            tracing::error!("Get circle members failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get circle members"))
        }
    }
}

async fn regenerate_invite_code(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::regenerate_invite_code(&state.db.pool, id).await {
        Ok(code) => json_response(StatusCode::OK, ApiResponse::success(code)),
        Err(e) => {
            tracing::error!("Regenerate invite code failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to regenerate invite code"))
        }
    }
}

// ─── Location Handler Functions ──────────────────────────────────────────────

async fn get_member_locations(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::locations::get_member_locations(&state.db.pool, id).await {
        Ok(locations) => json_response(StatusCode::OK, ApiResponse::success(locations)),
        Err(e) => {
            tracing::error!("Get member locations failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get member locations"))
        }
    }
}

async fn get_location_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((id, user_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let since = chrono::Utc::now() - chrono::Duration::days(7);
    match db::locations::get_location_history(&state.db.pool, id, user_id, since).await {
        Ok(history) => json_response(StatusCode::OK, ApiResponse::success(history)),
        Err(e) => {
            tracing::error!("Get location history failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get location history"))
        }
    }
}

// ─── Place/Geofence Handler Functions ────────────────────────────────────────

async fn create_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreatePlaceRequest>,
) -> impl IntoResponse {
    match db::geofence::create_place(&state.db.pool, id, &req.name, req.latitude, req.longitude, req.radius_meters).await {
        Ok(place) => json_response(StatusCode::OK, ApiResponse::success(place)),
        Err(e) => {
            tracing::error!("Create place failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create place"))
        }
    }
}

async fn get_places(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::geofence::get_places(&state.db.pool, id).await {
        Ok(places) => json_response(StatusCode::OK, ApiResponse::success(places)),
        Err(e) => {
            tracing::error!("Get places failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get places"))
        }
    }
}

async fn update_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreatePlaceRequest>,
) -> impl IntoResponse {
    match db::geofence::update_place(&state.db.pool, id, &req.name, req.latitude, req.longitude, req.radius_meters).await {
        Ok(place) => json_response(StatusCode::OK, ApiResponse::success(place)),
        Err(e) => {
            tracing::error!("Update place failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update place"))
        }
    }
}

async fn delete_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::geofence::delete_place(&state.db.pool, id).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Place deleted")),
        Err(e) => {
            tracing::error!("Delete place failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to delete place"))
        }
    }
}

// ─── Incident Handler Functions ───────────────────────────────────────────────

async fn create_incident(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<IncidentRequest>,
) -> impl IntoResponse {
    match db::incidents::create_incident(&state.db.pool, id, Some(auth_user.user_id), &req.incident_type, req.latitude, req.longitude).await {
        Ok(incident) => json_response(StatusCode::OK, ApiResponse::success(incident)),
        Err(e) => {
            tracing::error!("Create incident failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create incident"))
        }
    }
}

async fn get_incidents(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::incidents::get_incidents(&state.db.pool, id, None).await {
        Ok(incidents) => json_response(StatusCode::OK, ApiResponse::success(incidents)),
        Err(e) => {
            tracing::error!("Get incidents failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get incidents"))
        }
    }
}

async fn update_incident_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let status = req["status"].as_str().unwrap_or("resolved");
    match db::incidents::update_incident_status(&state.db.pool, id, status).await {
        Ok(incident) => json_response(StatusCode::OK, ApiResponse::success(incident)),
        Err(e) => {
            tracing::error!("Update incident status failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update incident status"))
        }
    }
}

async fn log_incident_response(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let action = req["action"].as_str().unwrap_or("acknowledged");
    match db::incidents::log_incident_response(&state.db.pool, id, auth_user.user_id, action).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Response logged")),
        Err(e) => {
            tracing::error!("Log incident response failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to log response"))
        }
    }
}

// ─── Medication Handler Functions ────────────────────────────────────────────

async fn create_medication(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<CreateMedicationRequest>,
) -> impl IntoResponse {
    match db::medications::create_medication(&state.db.pool, auth_user.user_id, &req.name, &req.dosage, &req.schedule).await {
        Ok(medication) => json_response(StatusCode::OK, ApiResponse::success(medication)),
        Err(e) => {
            tracing::error!("Create medication failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create medication"))
        }
    }
}

async fn get_medications(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::medications::get_medications(&state.db.pool, auth_user.user_id).await {
        Ok(medications) => json_response(StatusCode::OK, ApiResponse::success(medications)),
        Err(e) => {
            tracing::error!("Get medications failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get medications"))
        }
    }
}

async fn update_medication(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreateMedicationRequest>,
) -> impl IntoResponse {
    match db::medications::update_medication(&state.db.pool, id, &req.name, &req.dosage, &req.schedule).await {
        Ok(medication) => json_response(StatusCode::OK, ApiResponse::success(medication)),
        Err(e) => {
            tracing::error!("Update medication failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update medication"))
        }
    }
}

async fn delete_medication(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::medications::delete_medication(&state.db.pool, id).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Medication deleted")),
        Err(e) => {
            tracing::error!("Delete medication failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to delete medication"))
        }
    }
}

async fn log_adherence(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let taken = req["taken"].as_bool().unwrap_or(true);
    let notes = req["notes"].as_str();
    match db::medications::log_adherence(&state.db.pool, id, auth_user.user_id, taken, notes).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Adherence logged")),
        Err(e) => {
            tracing::error!("Log adherence failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to log adherence"))
        }
    }
}

async fn get_adherence(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::medications::get_adherence(&state.db.pool, auth_user.user_id, None).await {
        Ok(records) => json_response(StatusCode::OK, ApiResponse::success(records)),
        Err(e) => {
            tracing::error!("Get adherence failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get adherence"))
        }
    }
}

// ─── Chat Handler Functions ──────────────────────────────────────────────────

async fn create_room(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = req["name"].as_str().unwrap_or("General");
    match db::chat::create_room(&state.db.pool, id, name).await {
        Ok(room_id) => json_response(StatusCode::OK, ApiResponse::success(room_id)),
        Err(e) => {
            tracing::error!("Create room failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create room"))
        }
    }
}

async fn get_rooms(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::chat::get_rooms(&state.db.pool, id).await {
        Ok(rooms) => json_response(StatusCode::OK, ApiResponse::success(rooms)),
        Err(e) => {
            tracing::error!("Get rooms failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get rooms"))
        }
    }
}

async fn get_messages(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::chat::get_messages(&state.db.pool, id, 50).await {
        Ok(messages) => json_response(StatusCode::OK, ApiResponse::success(messages)),
        Err(e) => {
            tracing::error!("Get messages failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get messages"))
        }
    }
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<ChatMessageRequest>,
) -> impl IntoResponse {
    match db::chat::insert_message(&state.db.pool, id, auth_user.user_id, &req.body).await {
        Ok(message) => json_response(StatusCode::OK, ApiResponse::success(ChatMessageResponse { message })),
        Err(e) => {
            tracing::error!("Send message failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to send message"))
        }
    }
}

async fn mark_read(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let last_read = Uuid::new_v4();
    match db::chat::mark_read(&state.db.pool, id, auth_user.user_id, last_read).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Marked as read")),
        Err(e) => {
            tracing::error!("Mark read failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to mark as read"))
        }
    }
}

// ─── Driving Handler Functions ───────────────────────────────────────────────

async fn create_driving_session(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::driving::create_session(&state.db.pool, auth_user.user_id).await {
        Ok(session) => json_response(StatusCode::OK, ApiResponse::success(session)),
        Err(e) => {
            tracing::error!("Create driving session failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create driving session"))
        }
    }
}

async fn update_driving_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::driving::update_session(&state.db.pool, id, Some(chrono::Utc::now())).await {
        Ok(session) => json_response(StatusCode::OK, ApiResponse::success(session)),
        Err(e) => {
            tracing::error!("Update driving session failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update driving session"))
        }
    }
}

async fn insert_driving_event(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let event_type = req["event_type"].as_str().unwrap_or("unknown");
    let latitude = req["latitude"].as_f64().unwrap_or(0.0);
    let longitude = req["longitude"].as_f64().unwrap_or(0.0);
    let severity = req["severity"].as_f64().unwrap_or(0.0);
    match db::driving::insert_event(&state.db.pool, id, event_type, latitude, longitude, severity).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Event recorded")),
        Err(e) => {
            tracing::error!("Insert driving event failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to record event"))
        }
    }
}

async fn get_driving_reports(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::driving::get_reports(&state.db.pool, auth_user.user_id).await {
        Ok(reports) => json_response(StatusCode::OK, ApiResponse::success(reports)),
        Err(e) => {
            tracing::error!("Get driving reports failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get driving reports"))
        }
    }
}

// ─── Subscription Handler Functions ──────────────────────────────────────────

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<SubscribeRequest>,
) -> impl IntoResponse {
    let stripe_customer_id = format!("cus_{}", Uuid::new_v4().to_string().replace("-", ""));
    match db::subscriptions::create_subscription(&state.db.pool, auth_user.user_id, &stripe_customer_id, None, &req.tier).await {
        Ok(subscription) => json_response(StatusCode::OK, ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Create subscription failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to create subscription"))
        }
    }
}

async fn get_subscription(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::subscriptions::get_subscription(&state.db.pool, auth_user.user_id).await {
        Ok(Some(subscription)) => json_response(StatusCode::OK, ApiResponse::success(subscription)),
        Ok(None) => json_response(StatusCode::NOT_FOUND, ApiResponse::<()>::error("No subscription found")),
        Err(e) => {
            tracing::error!("Get subscription failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get subscription"))
        }
    }
}

async fn update_subscription_tier(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let tier = req["tier"].as_str().unwrap_or("basic");
    match db::subscriptions::update_tier(&state.db.pool, id, tier).await {
        Ok(subscription) => json_response(StatusCode::OK, ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Update subscription tier failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to update subscription tier"))
        }
    }
}

async fn cancel_subscription(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::subscriptions::cancel_subscription(&state.db.pool, id).await {
        Ok(subscription) => json_response(StatusCode::OK, ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Cancel subscription failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to cancel subscription"))
        }
    }
}

// ─── Tile Handler Functions ──────────────────────────────────────────────────

async fn link_tile(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<LinkTileRequest>,
) -> impl IntoResponse {
    json_response(StatusCode::OK, ApiResponse::success("Tile linked"))
}

async fn ring_tile(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::tile::ring_tile(&id).await {
        Ok(_) => json_response(StatusCode::OK, ApiResponse::success("Ring command sent")),
        Err(e) => {
            tracing::error!("Ring tile failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to ring tile"))
        }
    }
}

async fn locate_tile(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::tile::locate_tile(&id).await {
        Ok(location) => json_response(StatusCode::OK, ApiResponse::success(location)),
        Err(e) => {
            tracing::error!("Locate tile failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to locate tile"))
        }
    }
}

// ─── Geofence Event Handler Functions ────────────────────────────────────────

async fn get_circle_geofence_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::geofence_events::get_circle_geofence_events(&state.db.pool, id, 50).await {
        Ok(events) => json_response(StatusCode::OK, ApiResponse::success(events)),
        Err(e) => {
            tracing::error!("Get geofence events failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get geofence events"))
        }
    }
}

// ─── Incident Response Handler Functions ─────────────────────────────────────

async fn acknowledge_incident(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<SosAcknowledgeRequest>,
) -> impl IntoResponse {
    // Record the response
    if let Err(e) = db::incident_responses::record_incident_response(
        &state.db.pool,
        id,
        auth_user.user_id,
        &req.action,
        req.note.as_deref(),
    ).await {
        tracing::error!("Failed to record incident response: {:?}", e);
        return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to record response"));
    }

    // Update incident status
    let new_status = match req.action.as_str() {
        "acknowledged" => "acknowledged",
        "resolved" => "resolved",
        _ => "acknowledged",
    };

    if let Err(e) = db::incident_responses::update_incident_status(&state.db.pool, id, new_status).await {
        tracing::error!("Failed to update incident status: {:?}", e);
    }

    json_response(StatusCode::OK, ApiResponse::success("Incident acknowledged"))
}

async fn get_incident_responses(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::incident_responses::get_incident_responses(&state.db.pool, id).await {
        Ok(responses) => json_response(StatusCode::OK, ApiResponse::success(responses)),
        Err(e) => {
            tracing::error!("Get incident responses failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get incident responses"))
        }
    }
}

// ─── Medication Adherence Handler Functions ─────────────────────────────────

async fn get_medication_adherence_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::medications::get_adherence_stats(&state.db.pool, id).await {
        Ok(stats) => json_response(StatusCode::OK, ApiResponse::success(stats)),
        Err(e) => {
            tracing::error!("Get adherence stats failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get adherence stats"))
        }
    }
}

async fn get_medication_adherence_streak(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::medications::get_adherence_streak(&state.db.pool, id).await {
        Ok(streak) => json_response(StatusCode::OK, ApiResponse::success(serde_json::json!({ "streak": streak }))),
        Err(e) => {
            tracing::error!("Get adherence streak failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get adherence streak"))
        }
    }
}

async fn get_upcoming_medication_schedules(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::medications::get_upcoming_schedules(&state.db.pool, auth_user.user_id, 24).await {
        Ok(schedules) => json_response(StatusCode::OK, ApiResponse::success(schedules)),
        Err(e) => {
            tracing::error!("Get upcoming schedules failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get upcoming schedules"))
        }
    }
}

// ─── Driving Detail Handler Functions ──────────────────────────────────────

async fn get_driving_session_detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::driving::get_session_with_events(&state.db.pool, id).await {
        Ok(detail) => json_response(StatusCode::OK, ApiResponse::success(detail)),
        Err(e) => {
            tracing::error!("Get driving session detail failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get session detail"))
        }
    }
}

async fn get_driving_stats(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match db::driving::get_driving_stats(&state.db.pool, auth_user.user_id).await {
        Ok(stats) => json_response(StatusCode::OK, ApiResponse::success(stats)),
        Err(e) => {
            tracing::error!("Get driving stats failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get driving stats"))
        }
    }
}

// ─── Dispatch Handler Functions ─────────────────────────────────────────────

async fn trigger_emergency_dispatch(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    // Get the incident
    let incident = match db::incidents::get_incidents(&state.db.pool, Uuid::new_v4(), Some("active")).await {
        Ok(incidents) => incidents.into_iter().find(|i| i.id == id),
        Err(e) => {
            tracing::error!("Failed to get incident: {:?}", e);
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get incident"));
        }
    };

    let incident = match incident {
        Some(i) => i,
        None => return json_response(StatusCode::NOT_FOUND, ApiResponse::<()>::error("Incident not found")),
    };

    match services::dispatch::trigger_dispatch(&incident).await {
        Ok(response) => json_response(StatusCode::OK, ApiResponse::success(response)),
        Err(e) => {
            tracing::error!("Dispatch failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Dispatch failed"))
        }
    }
}

async fn get_dispatch_status(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::dispatch::get_dispatch_status(&id).await {
        Ok(status) => json_response(StatusCode::OK, ApiResponse::success(status)),
        Err(e) => {
            tracing::error!("Get dispatch status failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get dispatch status"))
        }
    }
}

async fn cancel_dispatch(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::dispatch::cancel_dispatch(&id).await {
        Ok(status) => json_response(StatusCode::OK, ApiResponse::success(status)),
        Err(e) => {
            tracing::error!("Cancel dispatch failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to cancel dispatch"))
        }
    }
}

// ─── Tile Community Find Handler Functions ─────────────────────────────────

async fn get_tile_community_finds(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::tile::get_community_finds(&id).await {
        Ok(finds) => json_response(StatusCode::OK, ApiResponse::success(finds)),
        Err(e) => {
            tracing::error!("Get community finds failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get community finds"))
        }
    }
}

async fn submit_community_find(
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let tile_id = req["tile_id"].as_str().unwrap_or("");
    let latitude = req["latitude"].as_f64().unwrap_or(0.0);
    let longitude = req["longitude"].as_f64().unwrap_or(0.0);
    let reporter_id = req["reporter_id"].as_str().unwrap_or("");

    match services::tile::submit_community_find(tile_id, latitude, longitude, reporter_id).await {
        Ok(result) => json_response(StatusCode::OK, ApiResponse::success(result)),
        Err(e) => {
            tracing::error!("Submit community find failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to submit community find"))
        }
    }
}

// ─── Assistance Handler Functions ───────────────────────────────────────────

async fn request_roadside_assistance(
    State(_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let assistance_type = match req["assistance_type"].as_str().unwrap_or("towing") {
        "flat_tire" => services::assistance::AssistanceType::FlatTire,
        "jump_start" => services::assistance::AssistanceType::JumpStart,
        "lockout" => services::assistance::AssistanceType::Lockout,
        "fuel_delivery" => services::assistance::AssistanceType::FuelDelivery,
        _ => services::assistance::AssistanceType::Towing,
    };

    let request = services::assistance::AssistanceRequest {
        user_id: auth_user.user_id.to_string(),
        assistance_type,
        latitude: req["latitude"].as_f64().unwrap_or(0.0),
        longitude: req["longitude"].as_f64().unwrap_or(0.0),
        address: req["address"].as_str().map(|s| s.to_string()),
        notes: req["notes"].as_str().map(|s| s.to_string()),
        vehicle_info: req["vehicle_info"].as_str().map(|s| s.to_string()),
    };

    match services::assistance::request_roadside_assistance(&request).await {
        Ok(response) => json_response(StatusCode::OK, ApiResponse::success(response)),
        Err(e) => {
            tracing::error!("Roadside assistance failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to request assistance"))
        }
    }
}

async fn request_medical_advice(
    State(_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let request = services::assistance::AssistanceRequest {
        user_id: auth_user.user_id.to_string(),
        assistance_type: services::assistance::AssistanceType::MedicalAdvice,
        latitude: req["latitude"].as_f64().unwrap_or(0.0),
        longitude: req["longitude"].as_f64().unwrap_or(0.0),
        address: req["address"].as_str().map(|s| s.to_string()),
        notes: req["notes"].as_str().map(|s| s.to_string()),
        vehicle_info: None,
    };

    match services::assistance::request_medical_advice(&request).await {
        Ok(response) => json_response(StatusCode::OK, ApiResponse::success(response)),
        Err(e) => {
            tracing::error!("Medical advice failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to request medical advice"))
        }
    }
}

async fn get_assistance_status(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // In production: query the assistance provider API
    match services::assistance::get_assistance_status(&id).await {
        Ok(status) => json_response(StatusCode::OK, ApiResponse::success(status)),
        Err(e) => {
            tracing::error!("Get assistance status failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get assistance status"))
        }
    }
}

async fn cancel_assistance(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::assistance::cancel_assistance(&id).await {
        Ok(status) => json_response(StatusCode::OK, ApiResponse::success(status)),
        Err(e) => {
            tracing::error!("Cancel assistance failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to cancel assistance"))
        }
    }
}

async fn get_nearby_providers(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let lat = params.get("lat").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let lng = params.get("lng").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let assistance_type = services::assistance::AssistanceType::Towing;

    match services::assistance::get_nearby_providers(lat, lng, &assistance_type).await {
        Ok(providers) => json_response(StatusCode::OK, ApiResponse::success(providers)),
        Err(e) => {
            tracing::error!("Get nearby providers failed: {:?}", e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<()>::error("Failed to get providers"))
        }
    }
}

// ─── Sensor Handler Functions ───────────────────────────────────────────────

async fn submit_sensor_reading(
    State(_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<SensorReading>,
) -> impl IntoResponse {
    // In production: process through sensor pipeline
    // For now, just log and store
    tracing::info!(
        "Sensor reading from user {}: accel=({}, {}, {}) gyro=({}, {}, {})",
        auth_user.user_id,
        req.accel_x, req.accel_y, req.accel_z,
        req.gyro_x, req.gyro_y, req.gyro_z
    );

    json_response(StatusCode::OK, ApiResponse::success("Reading recorded"))
}

async fn get_sensor_status(
    State(_state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    json_response(StatusCode::OK, ApiResponse::success(serde_json::json!({
        "user_id": auth_user.user_id,
        "fall_detection": "active",
        "crash_detection": "active",
        "sensitivity": "medium",
    })))
}

// ─── WebSocket Handler ───────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state.hub.clone(), auth_user.user_id))
}

// ─── Helper Functions ────────────────────────────────────────────────────────

fn json_response<T: serde::Serialize>(status: StatusCode, data: ApiResponse<T>) -> axum::response::Response {
    let body = serde_json::to_string(&data).unwrap_or_default();
    axum::response::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap()
}

fn hash_password(password: &str) -> Result<String, String> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Password hashing failed: {e}"))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed = PasswordHash::new(hash).map_err(|e| format!("Invalid hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map(|_| true)
        .unwrap_or(false))
}

fn generate_access_token(user_id: &str, secret: &str, ttl_minutes: i64, family_id: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(ttl_minutes)).timestamp() as usize,
        token_type: "access".to_string(),
        family_id: family_id.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

fn generate_refresh_token(user_id: &str, secret: &str, ttl_days: i64, family_id: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::days(ttl_days)).timestamp() as usize,
        token_type: "refresh".to_string(),
        family_id: family_id.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

fn generate_totp(secret: &str) -> String {
    use totp_rs::{Algorithm, TOTP};
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.as_bytes().to_vec()).unwrap();
    totp.generate_current().unwrap_or_default()
}

fn verify_totp(secret: &str, code: &str) -> bool {
    use totp_rs::{Algorithm, TOTP};
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.as_bytes().to_vec()).unwrap();
    totp.check_current(code).unwrap_or(false)
}
