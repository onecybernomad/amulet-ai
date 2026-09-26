use crate::db::local_cache::LocalCache;
use crate::models::Incident;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SosAlert {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub lat: f64,
    pub lng: f64,
    pub silent: bool,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SosState {
    pub active_sos: Mutex<Option<SosAlert>>,
}

impl Default for SosState {
    fn default() -> Self {
        Self {
            active_sos: Mutex::new(None),
        }
    }
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Trigger an SOS alert.
#[tauri::command]
#[instrument(skip(app))]
pub async fn trigger_sos(
    app: AppHandle,
    state: State<'_, SosState>,
    cache: State<'_, LocalCache>,
    silent: bool,
) -> Result<SosAlert, String> {
    info!(silent, "SOS triggered");

    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let location = cache
        .get_latest_location(&user.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No location data available".to_string())?;

    let alert = SosAlert {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
        user_name: user.name.clone(),
        lat: location.lat,
        lng: location.lng,
        silent,
        status: "active".into(),
        created_at: chrono::Utc::now(),
    };

    // Store in state
    {
        let mut guard = state.active_sos.lock().map_err(|e| e.to_string())?;
        *guard = Some(alert.clone());
    }

    // Persist to local cache
    let incident = Incident {
        id: alert.id.clone(),
        user_id: user.id.clone(),
        incident_type: "sos".into(),
        lat: alert.lat,
        lng: alert.lng,
        severity: "critical".into(),
        description: if silent {
            Some("Silent SOS triggered".to_string())
        } else {
            Some("SOS triggered".to_string())
        },
        status: "active".into(),
        created_at: alert.created_at,
        resolved_at: None,
    };
    cache.insert_incident(&incident).map_err(|e| e.to_string())?;

    // Emit event to frontend
    if let Err(e) = app.emit("sos_alert", &alert) {
        error!(error = %e, "Failed to emit sos_alert event");
    }

    if !silent {
        // In production: trigger push notifications to circle members
        warn!("SOS ALERT: {} at ({}, {})", user.name, alert.lat, alert.lng);
    }

    Ok(alert)
}

/// Cancel an active SOS alert.
#[tauri::command]
#[instrument(skip(app))]
pub async fn cancel_sos(
    app: AppHandle,
    state: State<'_, SosState>,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!("Cancelling SOS");

    let mut guard = state.active_sos.lock().map_err(|e| e.to_string())?;
    if let Some(ref alert) = *guard {
        cache.update_incident_status(&alert.id, "cancelled").map_err(|e| e.to_string())?;
        if let Err(e) = app.emit("sos_cancelled", &alert.id) {
            error!(error = %e, "Failed to emit sos_cancelled event");
        }
        debug!(sos_id = %alert.id, "SOS cancelled");
    }
    *guard = None;
    Ok(())
}

/// Get the currently active SOS alert, if any.
#[tauri::command]
pub async fn get_active_sos(
    state: State<'_, SosState>,
) -> Result<Option<SosAlert>, String> {
    let guard = state.active_sos.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}
