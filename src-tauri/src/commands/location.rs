use crate::db::local_cache::LocalCache;
use crate::models::LocationPing;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberLocation {
    pub member_id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocationTrackingState {
    pub active: Arc<AtomicBool>,
}

impl Default for LocationTrackingState {
    fn default() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Start continuous location tracking.
#[tauri::command]
#[instrument(skip(app))]
pub async fn start_location_tracking(
    app: AppHandle,
    state: State<'_, LocationTrackingState>,
    cache: State<'_, LocalCache>,
    mode: Option<String>,
) -> Result<(), String> {
    if state.active.swap(true, Ordering::SeqCst) {
        return Err("Location tracking is already active".into());
    }

    let tracking_mode = mode.unwrap_or_else(|| "balanced".into());
    info!(mode = %tracking_mode, "Starting location tracking");
    let active = state.active.clone();
    let app_handle = app.clone();
    let cache = cache.inner().clone();

    let interval_secs = match tracking_mode.as_str() {
        "active" => 5,
        "passive" => 30,
        _ => 10, // balanced
    };

    tokio::spawn(async move {
        while active.load(Ordering::SeqCst) {
            // In production, use tauri-plugin-geolocation to get real position
            // For scaffolding, emit a simulated ping
            let ping = LocationPing {
                id: Uuid::new_v4().to_string(),
                user_id: "current_user".into(),
                lat: 40.7128,
                lng: -74.0060,
                accuracy: 10.0,
                speed: None,
                heading: None,
                timestamp: chrono::Utc::now(),
            };

            if let Err(e) = cache.insert_location_ping(&ping) {
                error!(error = %e, "Failed to cache location ping");
            }

            if let Err(e) = app_handle.emit("location_update", &ping) {
                error!(error = %e, "Failed to emit location_update event");
            }

            debug!(lat = %ping.lat, lng = %ping.lng, "Location ping emitted");
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
        }
        debug!("Location tracking loop terminated");
    });

    Ok(())
}

/// Stop continuous location tracking.
#[tauri::command]
pub async fn stop_location_tracking(
    state: State<'_, LocationTrackingState>,
) -> Result<(), String> {
    if !state.active.swap(false, Ordering::SeqCst) {
        return Err("Location tracking is not active".into());
    }
    info!("Location tracking stopped");
    Ok(())
}

/// Get the current device location.
#[tauri::command]
pub async fn get_current_location(
    cache: State<'_, LocalCache>,
) -> Result<LocationPing, String> {
    info!("Getting current location");
    cache
        .get_latest_location("current_user")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No location data available".into())
}

/// Get locations of all circle members.
#[tauri::command]
pub async fn get_member_locations(
    cache: State<'_, LocalCache>,
) -> Result<Vec<MemberLocation>, String> {
    info!("Getting member locations");
    let pings = cache.get_all_latest_locations().map_err(|e| e.to_string())?;

    let members: Vec<MemberLocation> = pings
        .into_iter()
        .map(|p| MemberLocation {
            member_id: p.user_id.clone(),
            name: p.user_id, // In production, look up display name
            lat: p.lat,
            lng: p.lng,
            accuracy: p.accuracy,
            speed: p.speed,
            heading: p.heading,
            timestamp: p.timestamp,
        })
        .collect();

    debug!(count = members.len(), "Retrieved member locations");
    Ok(members)
}
