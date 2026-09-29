use crate::db::local_cache::LocalCache;
use crate::sensors::detector::{CrashDetector, FallDetector, CrashState, FallState};
use crate::sensors::classifier::{CrashSeverity, SensorWindow};
use crate::sensors::bridge::{AccelerometerData, GyroscopeData};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

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
    pub id: String,
    pub user_id: String,
    pub status: String, // "pending", "acknowledged", "escalated", "cancelled"
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ── State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SensorDetectionState {
    pub fall_detector: Mutex<FallDetector>,
    pub crash_detector: Mutex<CrashDetector>,
    pub active_fall_alert: Mutex<Option<FallAlert>>,
    pub sensitivity: Mutex<String>, // "low", "medium", "high"
}

impl Default for SensorDetectionState {
    fn default() -> Self {
        Self {
            fall_detector: Mutex::new(FallDetector::new()),
            crash_detector: Mutex::new(CrashDetector::new()),
            active_fall_alert: Mutex::new(None),
            sensitivity: Mutex::new("medium".into()),
        }
    }
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Process a sensor reading for fall/crash detection.
#[tauri::command]
#[instrument(skip(app, reading))]
pub async fn process_sensor_reading(
    app: AppHandle,
    state: State<'_, SensorDetectionState>,
    cache: State<'_, LocalCache>,
    reading: SensorReading,
) -> Result<DetectionResult, String> {
    debug!(accel_x = reading.accel_x, accel_y = reading.accel_y, accel_z = reading.accel_z, "Processing sensor reading");

    let accel = AccelerometerData {
        x: reading.accel_x,
        y: reading.accel_y,
        z: reading.accel_z,
        timestamp: reading.timestamp,
    };
    let gyro = GyroscopeData {
        x: reading.gyro_x,
        y: reading.gyro_y,
        z: reading.gyro_z,
        timestamp: reading.timestamp,
    };

    // Update fall detector
    let fall_state = {
        let mut detector = state.fall_detector.lock().map_err(|e| e.to_string())?;
        // Feed data into the detector's window
        detector.window.push_accel(accel);
        detector.window.push_gyro(gyro);
        let window = detector.window.clone();
        detector.process(&window)
    };

    // Handle fall escalation (outside the mutex guard)
    if fall_state == FallState::Escalated {
        handle_fall_escalation(&app, &state, &cache).await?;
    }

    // Update crash detector
    let (crash_state, crash_severity, is_false_positive) = {
        let mut detector = state.crash_detector.lock().map_err(|e| e.to_string())?;
        detector.window.push_accel(accel);
        detector.window.push_gyro(gyro);
        let window = detector.window.clone();
        let crash_state = detector.process(&window);

        let severity = if crash_state == CrashState::CrashDetected {
            let sev = detector.window.classify_crash_severity();
            let fp = detector.window.is_false_positive();
            if fp {
                None
            } else {
                Some(sev.to_string())
            }
        } else {
            None
        };

        let fp = detector.window.is_false_positive();
        (crash_state, severity, fp)
    };

    // Auto-trigger SOS for severe crashes
    if crash_state == CrashState::CrashDetected && crash_severity.is_some() {
        let sev = crash_severity.as_deref().unwrap_or("moderate");
        if sev == "severe" {
            warn!("Severe crash detected — auto-triggering SOS");
            // In production: trigger SOS via the SOS command
            let _ = app.emit("crash_detected", serde_json::json!({
                "severity": sev,
                "auto_sos": true,
            }));
        }
    }

    Ok(DetectionResult {
        fall_detected: fall_state == FallState::FallDetected || fall_state == FallState::Escalated,
        fall_state: format!("{:?}", fall_state),
        crash_detected: crash_state == CrashState::CrashDetected,
        crash_state: format!("{:?}", crash_state),
        crash_severity,
        is_false_positive,
    })
}

/// Acknowledge a fall alert (user confirms they are OK).
#[tauri::command]
#[instrument(skip(app))]
pub async fn acknowledge_fall_alert(
    app: AppHandle,
    state: State<'_, SensorDetectionState>,
) -> Result<(), String> {
    info!("Fall alert acknowledged by user");

    let mut alert = state.active_fall_alert.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut a) = *alert {
        a.status = "acknowledged".into();
        a.acknowledged_at = Some(chrono::Utc::now());

        // Reset the fall detector
        let mut detector = state.fall_detector.lock().map_err(|e| e.to_string())?;
        detector.reset();

        let _ = app.emit("fall_alert_acknowledged", serde_json::json!({
            "alert_id": a.id,
        }));
    }
    *alert = None;
    Ok(())
}

/// Cancel a fall alert (false alarm).
#[tauri::command]
#[instrument(skip(app))]
pub async fn cancel_fall_alert(
    app: AppHandle,
    state: State<'_, SensorDetectionState>,
) -> Result<(), String> {
    info!("Fall alert cancelled by user");

    let mut alert = state.active_fall_alert.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut a) = *alert {
        a.status = "cancelled".into();

        // Reset the fall detector
        let mut detector = state.fall_detector.lock().map_err(|e| e.to_string())?;
        detector.reset();

        let _ = app.emit("fall_alert_cancelled", serde_json::json!({
            "alert_id": a.id,
        }));
    }
    *alert = None;
    Ok(())
}

/// Get the current fall alert status.
#[tauri::command]
pub async fn get_fall_alert_status(
    state: State<'_, SensorDetectionState>,
) -> Result<Option<FallAlert>, String> {
    let alert = state.active_fall_alert.lock().map_err(|e| e.to_string())?;
    Ok(alert.clone())
}

/// Set detection sensitivity.
#[tauri::command]
pub async fn set_detection_sensitivity(
    state: State<'_, SensorDetectionState>,
    sensitivity: String,
) -> Result<(), String> {
    let mut s = state.sensitivity.lock().map_err(|e| e.to_string())?;
    *s = sensitivity;
    info!(sensitivity = %s, "Detection sensitivity updated");
    Ok(())
}

/// Reset all detection state (after incident resolved).
#[tauri::command]
#[instrument(skip(app))]
pub async fn reset_detection(
    app: AppHandle,
    state: State<'_, SensorDetectionState>,
) -> Result<(), String> {
    info!("Resetting all detection state");

    {
        let mut fall = state.fall_detector.lock().map_err(|e| e.to_string())?;
        fall.reset();
    }
    {
        let mut crash = state.crash_detector.lock().map_err(|e| e.to_string())?;
        crash.reset();
    }
    {
        let mut alert = state.active_fall_alert.lock().map_err(|e| e.to_string())?;
        *alert = None;
    }

    let _ = app.emit("detection_reset", ());
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Handle fall escalation — create alert and notify.
async fn handle_fall_escalation(
    app: &AppHandle,
    state: &SensorDetectionState,
    cache: &LocalCache,
) -> Result<(), String> {
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let alert = FallAlert {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
        status: "pending".into(),
        created_at: chrono::Utc::now(),
        acknowledged_at: None,
    };

    {
        let mut a = state.active_fall_alert.lock().map_err(|e| e.to_string())?;
        *a = Some(alert.clone());
    }

    // Emit event to frontend — triggers audible alert + countdown
    let _ = app.emit("fall_detected", serde_json::json!({
        "alert_id": alert.id,
        "user_id": user.id,
        "user_name": user.name,
        "timestamp": alert.created_at,
    }));

    warn!("Fall detected for user {} — awaiting acknowledgment", user.name);
    Ok(())
}
