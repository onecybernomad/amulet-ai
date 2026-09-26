use serde::{Deserialize, Serialize};

// ── Auth ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

// ── User ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ── Circle ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub member_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Place / Geofence ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub radius: f64,
    pub place_type: String,
    pub notify_on_enter: bool,
    pub notify_on_exit: bool,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Location ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationPing {
    pub id: String,
    pub user_id: String,
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── Incident ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub user_id: String,
    pub incident_type: String,
    pub lat: f64,
    pub lng: f64,
    pub severity: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ── Medication ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Vec<String>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseLog {
    pub id: String,
    pub medication_id: String,
    pub user_id: String,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── Chat ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub name: String,
    pub circle_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub sender_id: String,
    pub body: String,
    pub media_urls: Option<Vec<String>>,
    pub read_by: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Driving ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingSession {
    pub id: String,
    pub user_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub distance_km: Option<f64>,
    pub max_speed: Option<f64>,
    pub avg_speed: Option<f64>,
    pub harsh_braking_count: Option<i32>,
    pub rapid_acceleration_count: Option<i32>,
    pub phone_usage_count: Option<i32>,
}

// ── Subscription ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub status: String,
    pub current_period_start: chrono::DateTime<chrono::Utc>,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Tile ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileTracker {
    pub id: String,
    pub user_id: String,
    pub tile_id: String,
    pub name: String,
    pub device_type: Option<String>,
    pub battery_level: Option<i32>,
    pub last_location: Option<LocationPing>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub ring_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
