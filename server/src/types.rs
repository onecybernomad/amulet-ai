use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Entity Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_tier: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub failed_login_attempts: i64,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// User profile returned by the API (no password_hash).
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_tier: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            phone: user.phone,
            avatar_url: user.avatar_url,
            subscription_tier: user.subscription_tier,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub dosage: String,
    pub schedule: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub distance_km: Option<f64>,
    pub max_speed: Option<f64>,
    pub avg_speed: Option<f64>,
    pub harsh_braking_count: Option<i64>,
    pub rapid_acceleration_count: Option<i64>,
    pub phone_usage_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: Option<String>,
    pub tier: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerifyRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerifyConfirmRequest {
    pub token: String,
}

// ─── Response Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofenceEvent {
    pub id: Uuid,
    pub place_id: Uuid,
    pub place_name: String,
    pub circle_id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SosTriggerRequest {
    pub circle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub silent: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SosAcknowledgeRequest {
    pub incident_id: Uuid,
    pub action: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,
    pub gyro_x: f64,
    pub gyro_y: f64,
    pub gyro_z: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub fall_detected: bool,
    pub fall_state: String,
    pub crash_detected: bool,
    pub crash_state: String,
    pub crash_severity: Option<String>,
    pub is_false_positive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
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
