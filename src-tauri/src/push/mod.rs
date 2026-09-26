use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tracing::{debug, info, instrument};

/// Request notification permission from the OS.
#[instrument(skip(app))]
pub async fn request_permission(app: &tauri::AppHandle) -> Result<(), String> {
    info!("Requesting notification permission");
    app.notification()
        .request_permission()
        .map(|granted| {
            debug!("Notification permission response: {:?}", granted);
        })
        .map_err(|e| e.to_string())
}

/// Check if notification permission is granted.
#[instrument(skip(app))]
pub fn check_permission(app: &tauri::AppHandle) -> Result<bool, String> {
    let granted = app.notification().permission_state()
        .map(|s| matches!(s, tauri_plugin_notification::PermissionState::Granted))
        .unwrap_or(false);
    debug!(granted, "Notification permission status");
    Ok(granted)
}

/// Send a local notification.
#[instrument(skip(app))]
pub async fn send_notification(
    app: &tauri::AppHandle,
    title: &str,
    body: &str,
) -> Result<(), String> {
    info!(title = %title, "Sending notification");
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

/// Send an SOS alert notification.
#[instrument(skip(app))]
pub async fn send_sos_notification(
    app: &tauri::AppHandle,
    user_name: &str,
    lat: f64,
    lng: f64,
) -> Result<(), String> {
    let title = "SOS ALERT";
    let body = format!("{} triggered an SOS at ({:.4}, {:.4})", user_name, lat, lng);
    send_notification(app, &title, &body).await
}

/// Send a geofence alert notification.
#[instrument(skip(app))]
pub async fn send_geofence_notification(
    app: &tauri::AppHandle,
    member_name: &str,
    place_name: &str,
    event: &str,
) -> Result<(), String> {
    let title = "Geofence Alert";
    let body = format!("{} {} {}", member_name, event, place_name);
    send_notification(app, &title, &body).await
}

/// Send a medication reminder notification.
#[instrument(skip(app))]
pub async fn send_medication_reminder(
    app: &tauri::AppHandle,
    med_name: &str,
    dosage: &str,
) -> Result<(), String> {
    let title = "Medication Reminder";
    let body = format!("Time to take {} — {}", med_name, dosage);
    send_notification(app, &title, &body).await
}
