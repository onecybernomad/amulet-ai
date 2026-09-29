use uuid::Uuid;
use super::{HubMessage, RealtimeHub};

/// Handle an incoming fall detection event from a client.
/// Creates an incident record and broadcasts to the circle.
pub async fn handle_fall(
    hub: &RealtimeHub,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
) {
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        tracing::warn!("Invalid fall coordinates from user {}: lat={}, lng={}", user_id, latitude, longitude);
        return;
    }

    tracing::warn!("Fall detected for user {} at ({}, {})", user_id, latitude, longitude);

    let circles = {
        let members = hub.members.lock().await;
        members
            .iter()
            .filter(|(_, users)| users.contains(&user_id))
            .map(|(circle_id, _)| *circle_id)
            .collect::<Vec<_>>()
    };

    for circle_id in circles {
        let incident_id = Uuid::new_v4();

        if let Err(e) = crate::db::incidents::create_incident(
            &hub.db_pool,
            circle_id,
            Some(user_id),
            "fall",
            Some(latitude),
            Some(longitude),
        ).await {
            tracing::error!("Failed to persist fall incident: {}", e);
        }

        let incident = crate::types::Incident {
            id: incident_id,
            circle_id,
            user_id: Some(user_id),
            incident_type: "fall".to_string(),
            latitude: Some(latitude),
            longitude: Some(longitude),
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        };

        hub.broadcast(HubMessage::Alert(incident.clone()));
        tracing::warn!("Broadcasted fall alert to circle {}", circle_id);
    }
}

/// Handle an incoming crash detection event from a client.
/// Creates an incident record and broadcasts to the circle.
pub async fn handle_crash(
    hub: &RealtimeHub,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
    severity: &str,
) {
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        tracing::warn!("Invalid crash coordinates from user {}: lat={}, lng={}", user_id, latitude, longitude);
        return;
    }

    tracing::warn!("Crash detected for user {} at ({}, {}) — severity: {}", user_id, latitude, longitude, severity);

    let circles = {
        let members = hub.members.lock().await;
        members
            .iter()
            .filter(|(_, users)| users.contains(&user_id))
            .map(|(circle_id, _)| *circle_id)
            .collect::<Vec<_>>()
    };

    for circle_id in circles {
        let incident_id = Uuid::new_v4();

        if let Err(e) = crate::db::incidents::create_incident(
            &hub.db_pool,
            circle_id,
            Some(user_id),
            "crash",
            Some(latitude),
            Some(longitude),
        ).await {
            tracing::error!("Failed to persist crash incident: {}", e);
        }

        let incident = crate::types::Incident {
            id: incident_id,
            circle_id,
            user_id: Some(user_id),
            incident_type: "crash".to_string(),
            latitude: Some(latitude),
            longitude: Some(longitude),
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        };

        hub.broadcast(HubMessage::Alert(incident.clone()));
        tracing::warn!("Broadcasted crash alert to circle {}", circle_id);
    }
}

/// Handle an incoming SOS alert from a client.
/// Creates an incident record in the DB and broadcasts to the circle.
pub async fn handle_sos(
    hub: &RealtimeHub,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
    silent: bool,
) {
    // Validate coordinates
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        tracing::warn!("Invalid SOS coordinates from user {}: lat={}, lng={}", user_id, latitude, longitude);
        return;
    }

    tracing::warn!("SOS alert from user {} at ({}, {}) [silent={}]", user_id, latitude, longitude, silent);

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
        let incident_id = Uuid::new_v4();

        // Persist to database
        if let Err(e) = crate::db::incidents::create_incident(
            &hub.db_pool,
            circle_id,
            Some(user_id),
            "sos",
            Some(latitude),
            Some(longitude),
        ).await {
            tracing::error!("Failed to persist SOS incident: {}", e);
        }

        let incident = crate::types::Incident {
            id: incident_id,
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
