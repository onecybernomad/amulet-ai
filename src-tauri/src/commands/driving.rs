use crate::db::local_cache::LocalCache;
use crate::models::DrivingSession;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingEvent {
    pub event_type: String, // "hard_braking" | "rapid_acceleration" | "phone_use" | "speeding"
    pub latitude: f64,
    pub longitude: f64,
    pub severity: f64,
    pub speed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingSessionSummary {
    pub id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub distance_km: f64,
    pub max_speed: f64,
    pub avg_speed: f64,
    pub safety_score: i64,
    pub event_count: i64,
}

// ── State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DrivingState {
    pub active: Arc<AtomicBool>,
    pub current_session_id: Arc<std::sync::Mutex<Option<String>>>,
}

impl Default for DrivingState {
    fn default() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            current_session_id: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Start a driving session.
#[tauri::command]
#[instrument(skip(app))]
pub async fn start_driving_session(
    app: AppHandle,
    state: State<'_, DrivingState>,
    cache: State<'_, LocalCache>,
) -> Result<DrivingSession, String> {
    if state.active.swap(true, Ordering::SeqCst) {
        return Err("Driving session already active".into());
    }

    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let session = DrivingSession {
        id: Uuid::new_v4().to_string(),
        user_id: user.id,
        started_at: chrono::Utc::now(),
        ended_at: None,
        distance_km: None,
        max_speed: None,
        avg_speed: None,
        harsh_braking_count: Some(0),
        rapid_acceleration_count: Some(0),
        phone_usage_count: Some(0),
    };

    // Store in local cache
    // In production: insert into driving_sessions table

    {
        let mut session_id = state.current_session_id.lock().map_err(|e| e.to_string())?;
        *session_id = Some(session.id.clone());
    }

    let _ = app.emit("driving_session_started", &session);
    info!(session_id = %session.id, "Driving session started");

    Ok(session)
}

/// Stop the current driving session.
#[tauri::command]
#[instrument(skip(app))]
pub async fn stop_driving_session(
    app: AppHandle,
    state: State<'_, DrivingState>,
    cache: State<'_, LocalCache>,
) -> Result<DrivingSessionSummary, String> {
    if !state.active.swap(false, Ordering::SeqCst) {
        return Err("No active driving session".into());
    }

    let session_id = {
        let mut id = state.current_session_id.lock().map_err(|e| e.to_string())?;
        id.take().ok_or_else(|| "No session ID".to_string())?
    };

    let _user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    // In production: fetch session with events from DB and calculate summary
    let summary = DrivingSessionSummary {
        id: session_id,
        started_at: chrono::Utc::now() - chrono::Duration::minutes(30),
        ended_at: Some(chrono::Utc::now()),
        distance_km: 25.5,
        max_speed: 80.0,
        avg_speed: 55.0,
        safety_score: 85,
        event_count: 3,
    };

    let _ = app.emit("driving_session_ended", &summary);
    info!(session_id = %summary.id, "Driving session ended");

    Ok(summary)
}

/// Report a driving event (hard braking, rapid acceleration, phone use, speeding).
#[tauri::command]
#[instrument(skip(app, event))]
pub async fn report_driving_event(
    app: AppHandle,
    state: State<'_, DrivingState>,
    cache: State<'_, LocalCache>,
    event: DrivingEvent,
) -> Result<(), String> {
    if !state.active.load(Ordering::SeqCst) {
        return Err("No active driving session".into());
    }

    let session_id = {
        let id = state.current_session_id.lock().map_err(|e| e.to_string())?;
        id.clone().ok_or_else(|| "No session ID".to_string())?
    };

    let _user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    // In production: insert into driving_events table
    info!(
        session_id = %session_id,
        event_type = %event.event_type,
        severity = %event.severity,
        "Driving event reported"
    );

    // Emit event for real-time updates
    let _ = app.emit("driving_event", serde_json::json!({
        "session_id": session_id,
        "event_type": event.event_type,
        "latitude": event.latitude,
        "longitude": event.longitude,
        "severity": event.severity,
        "speed": event.speed,
    }));

    Ok(())
}

/// Get the current driving session status.
#[tauri::command]
pub async fn get_driving_status(
    state: State<'_, DrivingState>,
) -> Result<serde_json::Value, String> {
    let active = state.active.load(Ordering::SeqCst);
    let session_id = {
        let id = state.current_session_id.lock().map_err(|e| e.to_string())?;
        id.clone()
    };

    Ok(serde_json::json!({
        "active": active,
        "session_id": session_id,
    }))
}

/// Get driving statistics for the current user.
#[tauri::command]
pub async fn get_driving_stats(
    cache: State<'_, LocalCache>,
) -> Result<serde_json::Value, String> {
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    // In production: fetch from driving_sessions table
    Ok(serde_json::json!({
        "user_id": user.id,
        "total_sessions": 42,
        "total_distance": 1250.5,
        "max_speed": 120.0,
        "avg_speed": 65.0,
        "total_harsh_braking": 8,
        "total_rapid_accel": 12,
        "total_phone_use": 3,
    }))
}
