use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Entity Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Circle {
    pub id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Place {
    pub id: Uuid,
    pub circle_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationPing {
    pub user_id: Uuid,
    pub circle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Incident {
    pub id: Uuid,
    pub circle_id: Uuid,
    pub user_id: Option<Uuid>,
    pub incident_type: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Medication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub dosage: String,
    pub schedule: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DrivingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: Option<String>,
    pub tier: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TileTracker {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tile_id: String,
    pub name: String,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

// ─── Request Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCircleRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlaceRequest {
    pub circle_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMedicationRequest {
    pub name: String,
    pub dosage: String,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SosRequest {
    pub circle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentRequest {
    pub circle_id: Uuid,
    pub incident_type: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageRequest {
    pub room_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkTileRequest {
    pub tile_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeRequest {
    pub tier: String,
    pub payment_method_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtpSendRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtpVerifyRequest {
    pub email: String,
    pub code: String,
}

// ─── Response Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MemberLocationResponse {
    pub user_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeofenceAlertResponse {
    pub place_name: String,
    pub entered: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageResponse {
    pub message: ChatMessage,
}

// ─── Generic API Response Wrapper ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}
