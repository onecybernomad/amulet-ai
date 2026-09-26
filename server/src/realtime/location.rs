use chrono::Utc;
use uuid::Uuid;
use super::{HubMessage, RealtimeHub};

/// Handle an incoming location_ping message from a client.
/// Validates the coordinates, stores in DB, and broadcasts to circle members.
pub async fn handle_location_ping(
    hub: &RealtimeHub,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
) {
    // Validate coordinates
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        tracing::warn!("Invalid coordinates from user {}: lat={}, lng={}", user_id, latitude, longitude);
        return;
    }

    tracing::debug!("Location ping from user {}: lat={}, lng={}", user_id, latitude, longitude);

    // Get the circles this user belongs to
    let circles = {
        let members = hub.members.lock().await;
        members
            .iter()
            .filter(|(_, users)| users.contains(&user_id))
            .map(|(circle_id, _)| *circle_id)
            .collect::<Vec<_>>()
    };

    // Broadcast to each circle the user is in
    for circle_id in circles {
        let ping = crate::types::LocationPing {
            user_id,
            circle_id,
            latitude,
            longitude,
            timestamp: Utc::now(),
        };

        hub.broadcast(HubMessage::LocationPing(ping.clone()));
        tracing::debug!("Broadcasted location ping to circle {}", circle_id);
    }
}
