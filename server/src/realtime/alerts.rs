use uuid::Uuid;
use super::{HubMessage, RealtimeHub};

/// Handle an incoming SOS alert from a client.
/// Creates an incident record and broadcasts to the circle.
pub async fn handle_sos(
    hub: &RealtimeHub,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
) {
    // Validate coordinates
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        tracing::warn!("Invalid SOS coordinates from user {}: lat={}, lng={}", user_id, latitude, longitude);
        return;
    }

    tracing::warn!("SOS alert from user {} at ({}, {})", user_id, latitude, longitude);

    // Find the user's circles
    let circles = {
        let members = hub.members.lock().await;
        members
            .iter()
            .filter(|(_, users)| users.contains(&user_id))
            .map(|(circle_id, _)| *circle_id)
            .collect::<Vec<_>>()
    };

    for circle_id in circles {
        let incident = crate::types::Incident {
            id: Uuid::new_v4(),
            circle_id,
            user_id: Some(user_id),
            incident_type: "sos".to_string(),
            latitude: Some(latitude),
            longitude: Some(longitude),
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        };

        hub.broadcast(HubMessage::Alert(incident.clone()));
        tracing::warn!("Broadcasted SOS alert to circle {}", circle_id);
    }
}
