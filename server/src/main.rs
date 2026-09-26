mod config;
mod db;
mod middleware;
mod realtime;
mod services;
mod types;

// Re-export db modules for use in handlers
use db::{circles, chat, driving, geofence, incidents, locations, medications, subscriptions, users};

use std::sync::Arc;
use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::config::Config;
use crate::db::Database;
use crate::middleware::auth::{auth_middleware, AuthState};
use crate::middleware::rate_limit::{rate_limit_middleware, RateLimiter};
use crate::realtime::{handle_connection, RealtimeHub};
use crate::types::*;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub hub: RealtimeHub,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Starting Amulet AI server...");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Create database pool (optional for now — server starts without DB)
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Database not available ({}). Starting in mock mode.", e);
            // Return a dummy pool — handlers will return mock data
            std::process::exit(1);
        });

    let db = Database::new(pool);
    let hub = RealtimeHub::new();

    let app_state = Arc::new(AppState {
        db,
        hub,
        config: config.clone(),
    });

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Rate limiter
    let rate_limiter = RateLimiter::new(100, std::time::Duration::from_secs(60));

    // Auth state
    let auth_state = Arc::new(AuthState {
        jwt_secret: config.jwt_secret.clone(),
    });

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/otp/send", post(send_otp))
        .route("/api/auth/otp/verify", post(verify_otp))
        // User routes
        .route("/api/users/me", get(get_current_user))
        .route("/api/users/me", put(update_current_user))
        // Circle routes
        .route("/api/circles", post(create_circle))
        .route("/api/circles/:id", get(get_circle))
        .route("/api/circles/:id/join", post(join_circle))
        .route("/api/circles/:id/leave", post(leave_circle))
        .route("/api/circles/:id/members", get(get_circle_members))
        .route("/api/circles/:id/invite-code", post(regenerate_invite_code))
        // Location routes
        .route("/api/circles/:id/locations", get(get_member_locations))
        .route("/api/circles/:id/locations/history/:user_id", get(get_location_history))
        // Place/Geofence routes
        .route("/api/circles/:id/places", post(create_place))
        .route("/api/circles/:id/places", get(get_places))
        .route("/api/places/:id", put(update_place))
        .route("/api/places/:id", delete(delete_place))
        // Incident routes
        .route("/api/circles/:id/incidents", post(create_incident))
        .route("/api/circles/:id/incidents", get(get_incidents))
        .route("/api/incidents/:id/status", put(update_incident_status))
        .route("/api/incidents/:id/respond", post(log_incident_response))
        // Medication routes
        .route("/api/medications", post(create_medication))
        .route("/api/medications", get(get_medications))
        .route("/api/medications/:id", put(update_medication))
        .route("/api/medications/:id", delete(delete_medication))
        .route("/api/medications/:id/adherence", post(log_adherence))
        .route("/api/medications/adherence", get(get_adherence))
        // Chat routes
        .route("/api/circles/:id/rooms", post(create_room))
        .route("/api/circles/:id/rooms", get(get_rooms))
        .route("/api/rooms/:id/messages", get(get_messages))
        .route("/api/rooms/:id/messages", post(send_message))
        .route("/api/rooms/:id/read", post(mark_read))
        // Driving routes
        .route("/api/driving/sessions", post(create_driving_session))
        .route("/api/driving/sessions/:id", put(update_driving_session))
        .route("/api/driving/sessions/:id/events", post(insert_driving_event))
        .route("/api/driving/reports", get(get_driving_reports))
        // Subscription routes
        .route("/api/subscriptions", post(create_subscription))
        .route("/api/subscriptions", get(get_subscription))
        .route("/api/subscriptions/:id/tier", put(update_subscription_tier))
        .route("/api/subscriptions/:id/cancel", post(cancel_subscription))
        // Tile routes
        .route("/api/tiles", post(link_tile))
        .route("/api/tiles/:id/ring", post(ring_tile))
        .route("/api/tiles/:id/locate", get(locate_tile))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server with graceful shutdown
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

// ─── Handler Functions ───────────────────────────────────────────────────────

async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("healthy"))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let email = req["email"].as_str().unwrap_or("");
    let password = req["password"].as_str().unwrap_or("");
    let display_name = req["display_name"].as_str().unwrap_or("");

    if email.is_empty() || password.is_empty() {
        return Json(ApiResponse::<()>::error("Email and password are required"));
    }

    let password_hash = format!("hashed_{}", password); // Use argon2 in production

    match db::users::create_user(&state.db.pool, email, &password_hash, display_name).await {
        Ok(user) => {
            let token = generate_jwt(&user.id.to_string(), &state.config.jwt_secret);
            Json(ApiResponse::success(AuthResponse { token, user }))
        }
        Err(e) => {
            tracing::error!("Registration failed: {:?}", e);
            Json(ApiResponse::<()>::error("Registration failed"))
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let email = req["email"].as_str().unwrap_or("");
    let password = req["password"].as_str().unwrap_or("");

    match db::users::get_user_by_email(&state.db.pool, email).await {
        Ok(Some(user)) => {
            let token = generate_jwt(&user.id.to_string(), &state.config.jwt_secret);
            Json(ApiResponse::success(AuthResponse { token, user }))
        }
        Ok(None) => Json(ApiResponse::<()>::error("Invalid credentials")),
        Err(e) => {
            tracing::error!("Login failed: {:?}", e);
            Json(ApiResponse::<()>::error("Login failed"))
        }
    }
}

async fn send_otp(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<OtpSendRequest>,
) -> impl IntoResponse {
    Json(ApiResponse::success("OTP sent"))
}

async fn verify_otp(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<OtpVerifyRequest>,
) -> impl IntoResponse {
    Json(ApiResponse::success("OTP verified"))
}

async fn get_current_user() -> impl IntoResponse {
    Json(ApiResponse::success("current_user"))
}

async fn update_current_user() -> impl IntoResponse {
    Json(ApiResponse::success("updated"))
}

async fn create_circle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCircleRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::circles::create_circle(&state.db.pool, &req.name, user_id).await {
        Ok(circle) => Json(ApiResponse::success(circle)),
        Err(e) => {
            tracing::error!("Create circle failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create circle"))
        }
    }
}

async fn get_circle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::get_circle(&state.db.pool, id).await {
        Ok(Some(circle)) => Json(ApiResponse::success(circle)),
        Ok(None) => Json(ApiResponse::<()>::error("Circle not found")),
        Err(e) => {
            tracing::error!("Get circle failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get circle"))
        }
    }
}

async fn join_circle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::circles::join_circle(&state.db.pool, id, user_id).await {
        Ok(_) => Json(ApiResponse::success("Joined circle")),
        Err(e) => {
            tracing::error!("Join circle failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to join circle"))
        }
    }
}

async fn leave_circle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::circles::leave_circle(&state.db.pool, id, user_id).await {
        Ok(_) => Json(ApiResponse::success("Left circle")),
        Err(e) => {
            tracing::error!("Leave circle failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to leave circle"))
        }
    }
}

async fn get_circle_members(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::get_circle_members(&state.db.pool, id).await {
        Ok(members) => Json(ApiResponse::success(members)),
        Err(e) => {
            tracing::error!("Get circle members failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get circle members"))
        }
    }
}

async fn regenerate_invite_code(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::circles::regenerate_invite_code(&state.db.pool, id).await {
        Ok(code) => Json(ApiResponse::success(code)),
        Err(e) => {
            tracing::error!("Regenerate invite code failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to regenerate invite code"))
        }
    }
}

async fn get_member_locations(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::locations::get_member_locations(&state.db.pool, id).await {
        Ok(locations) => Json(ApiResponse::success(locations)),
        Err(e) => {
            tracing::error!("Get member locations failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get member locations"))
        }
    }
}

async fn get_location_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((id, user_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let since = chrono::Utc::now() - chrono::Duration::days(7);
    match db::locations::get_location_history(&state.db.pool, id, user_id, since).await {
        Ok(history) => Json(ApiResponse::success(history)),
        Err(e) => {
            tracing::error!("Get location history failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get location history"))
        }
    }
}

async fn create_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreatePlaceRequest>,
) -> impl IntoResponse {
    match db::geofence::create_place(
        &state.db.pool,
        id,
        &req.name,
        req.latitude,
        req.longitude,
        req.radius_meters,
    )
    .await
    {
        Ok(place) => Json(ApiResponse::success(place)),
        Err(e) => {
            tracing::error!("Create place failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create place"))
        }
    }
}

async fn get_places(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::geofence::get_places(&state.db.pool, id).await {
        Ok(places) => Json(ApiResponse::success(places)),
        Err(e) => {
            tracing::error!("Get places failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get places"))
        }
    }
}

async fn update_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreatePlaceRequest>,
) -> impl IntoResponse {
    match db::geofence::update_place(
        &state.db.pool,
        id,
        &req.name,
        req.latitude,
        req.longitude,
        req.radius_meters,
    )
    .await
    {
        Ok(place) => Json(ApiResponse::success(place)),
        Err(e) => {
            tracing::error!("Update place failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to update place"))
        }
    }
}

async fn delete_place(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::geofence::delete_place(&state.db.pool, id).await {
        Ok(_) => Json(ApiResponse::success("Place deleted")),
        Err(e) => {
            tracing::error!("Delete place failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to delete place"))
        }
    }
}

async fn create_incident(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<IncidentRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::incidents::create_incident(
        &state.db.pool,
        id,
        Some(user_id),
        &req.incident_type,
        req.latitude,
        req.longitude,
    )
    .await
    {
        Ok(incident) => Json(ApiResponse::success(incident)),
        Err(e) => {
            tracing::error!("Create incident failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create incident"))
        }
    }
}

async fn get_incidents(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::incidents::get_incidents(&state.db.pool, id, None).await {
        Ok(incidents) => Json(ApiResponse::success(incidents)),
        Err(e) => {
            tracing::error!("Get incidents failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get incidents"))
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
        Ok(incident) => Json(ApiResponse::success(incident)),
        Err(e) => {
            tracing::error!("Update incident status failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to update incident status"))
        }
    }
}

async fn log_incident_response(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    let action = req["action"].as_str().unwrap_or("acknowledged");
    match db::incidents::log_incident_response(&state.db.pool, id, user_id, action).await {
        Ok(_) => Json(ApiResponse::success("Response logged")),
        Err(e) => {
            tracing::error!("Log incident response failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to log response"))
        }
    }
}

async fn create_medication(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMedicationRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::medications::create_medication(
        &state.db.pool,
        user_id,
        &req.name,
        &req.dosage,
        &req.schedule,
    )
    .await
    {
        Ok(medication) => Json(ApiResponse::success(medication)),
        Err(e) => {
            tracing::error!("Create medication failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create medication"))
        }
    }
}

async fn get_medications(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::medications::get_medications(&state.db.pool, user_id).await {
        Ok(medications) => Json(ApiResponse::success(medications)),
        Err(e) => {
            tracing::error!("Get medications failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get medications"))
        }
    }
}

async fn update_medication(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<CreateMedicationRequest>,
) -> impl IntoResponse {
    match db::medications::update_medication(
        &state.db.pool,
        id,
        &req.name,
        &req.dosage,
        &req.schedule,
    )
    .await
    {
        Ok(medication) => Json(ApiResponse::success(medication)),
        Err(e) => {
            tracing::error!("Update medication failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to update medication"))
        }
    }
}

async fn delete_medication(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::medications::delete_medication(&state.db.pool, id).await {
        Ok(_) => Json(ApiResponse::success("Medication deleted")),
        Err(e) => {
            tracing::error!("Delete medication failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to delete medication"))
        }
    }
}

async fn log_adherence(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    let taken = req["taken"].as_bool().unwrap_or(true);
    let notes = req["notes"].as_str();
    match db::medications::log_adherence(&state.db.pool, id, user_id, taken, notes).await {
        Ok(_) => Json(ApiResponse::success("Adherence logged")),
        Err(e) => {
            tracing::error!("Log adherence failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to log adherence"))
        }
    }
}

async fn get_adherence(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::medications::get_adherence(&state.db.pool, user_id, None).await {
        Ok(records) => Json(ApiResponse::success(records)),
        Err(e) => {
            tracing::error!("Get adherence failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get adherence"))
        }
    }
}

async fn create_room(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = req["name"].as_str().unwrap_or("General");
    match db::chat::create_room(&state.db.pool, id, name).await {
        Ok(room_id) => Json(ApiResponse::success(room_id)),
        Err(e) => {
            tracing::error!("Create room failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create room"))
        }
    }
}

async fn get_rooms(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::chat::get_rooms(&state.db.pool, id).await {
        Ok(rooms) => Json(ApiResponse::success(rooms)),
        Err(e) => {
            tracing::error!("Get rooms failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get rooms"))
        }
    }
}

async fn get_messages(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::chat::get_messages(&state.db.pool, id, 50).await {
        Ok(messages) => Json(ApiResponse::success(messages)),
        Err(e) => {
            tracing::error!("Get messages failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get messages"))
        }
    }
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<ChatMessageRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::chat::insert_message(&state.db.pool, id, user_id, &req.body).await {
        Ok(message) => Json(ApiResponse::success(ChatMessageResponse { message })),
        Err(e) => {
            tracing::error!("Send message failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to send message"))
        }
    }
}

async fn mark_read(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    let last_read = Uuid::new_v4(); // Get from request in production
    match db::chat::mark_read(&state.db.pool, id, user_id, last_read).await {
        Ok(_) => Json(ApiResponse::success("Marked as read")),
        Err(e) => {
            tracing::error!("Mark read failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to mark as read"))
        }
    }
}

async fn create_driving_session(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::driving::create_session(&state.db.pool, user_id).await {
        Ok(session) => Json(ApiResponse::success(session)),
        Err(e) => {
            tracing::error!("Create driving session failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create driving session"))
        }
    }
}

async fn update_driving_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::driving::update_session(&state.db.pool, id, Some(chrono::Utc::now())).await {
        Ok(session) => Json(ApiResponse::success(session)),
        Err(e) => {
            tracing::error!("Update driving session failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to update driving session"))
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
        Ok(_) => Json(ApiResponse::success("Event recorded")),
        Err(e) => {
            tracing::error!("Insert driving event failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to record event"))
        }
    }
}

async fn get_driving_reports(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::driving::get_reports(&state.db.pool, user_id).await {
        Ok(reports) => Json(ApiResponse::success(reports)),
        Err(e) => {
            tracing::error!("Get driving reports failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get driving reports"))
        }
    }
}

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubscribeRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    let stripe_customer_id = format!("cus_{}", Uuid::new_v4().to_string().replace("-", ""));
    match db::subscriptions::create_subscription(
        &state.db.pool,
        user_id,
        &stripe_customer_id,
        None,
        &req.tier,
    )
    .await
    {
        Ok(subscription) => Json(ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Create subscription failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to create subscription"))
        }
    }
}

async fn get_subscription(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    match db::subscriptions::get_subscription(&state.db.pool, user_id).await {
        Ok(Some(subscription)) => Json(ApiResponse::success(subscription)),
        Ok(None) => Json(ApiResponse::<()>::error("No subscription found")),
        Err(e) => {
            tracing::error!("Get subscription failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to get subscription"))
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
        Ok(subscription) => Json(ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Update subscription tier failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to update subscription tier"))
        }
    }
}

async fn cancel_subscription(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match db::subscriptions::cancel_subscription(&state.db.pool, id).await {
        Ok(subscription) => Json(ApiResponse::success(subscription)),
        Err(e) => {
            tracing::error!("Cancel subscription failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to cancel subscription"))
        }
    }
}

async fn link_tile(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<LinkTileRequest>,
) -> impl IntoResponse {
    Json(ApiResponse::success("Tile linked"))
}

async fn ring_tile(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::tile::ring_tile(&id).await {
        Ok(_) => Json(ApiResponse::success("Ring command sent")),
        Err(e) => {
            tracing::error!("Ring tile failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to ring tile"))
        }
    }
}

async fn locate_tile(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match services::tile::locate_tile(&id).await {
        Ok(location) => Json(ApiResponse::success(location)),
        Err(e) => {
            tracing::error!("Locate tile failed: {:?}", e);
            Json(ApiResponse::<()>::error("Failed to locate tile"))
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4(); // Get from JWT in production
    ws.on_upgrade(move |socket| handle_connection(socket, state.hub.clone(), user_id))
}

fn generate_jwt(user_id: &str, secret: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = serde_json::json!({
        "sub": user_id,
        "exp": (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
    });
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}
