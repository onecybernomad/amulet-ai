use chrono::Utc;
use uuid::Uuid;
use super::{HubMessage, RealtimeHub};

/// Handle an incoming location_ping message from a client.
/// Validates the coordinates, stores in DB, checks geofences, and broadcasts to circle members.
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

    // Persist location ping and check geofences for each circle
    for circle_id in &circles {
        // Persist to database
        if let Err(e) = crate::db::locations::insert_location_ping(
            &hub.db_pool, user_id, *circle_id, latitude, longitude,
        ).await {
            tracing::error!("Failed to persist location ping: {}", e);
        }

        // Check geofence events
        check_geofences_for_user(hub, user_id, *circle_id, latitude, longitude).await;
    }

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

/// Check if a user's location triggers any geofence events.
async fn check_geofences_for_user(
    hub: &RealtimeHub,
    user_id: Uuid,
    circle_id: Uuid,
    latitude: f64,
    longitude: f64,
) {
    // Get all places for this circle
    let places = match crate::db::geofence::get_places(&hub.db_pool, circle_id).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to get places for geofence check: {}", e);
            return;
        }
    };

    if places.is_empty() {
        return;
    }

    // Get user's current geofence state (which places they're inside)
    let user_state = match crate::db::geofence_events::get_user_geofence_state(&hub.db_pool, user_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get user geofence state: {}", e);
            return;
        }
    };

    let currently_inside: std::collections::HashSet<String> = user_state
        .iter()
        .map(|s| s.place_id.clone())
        .collect();

    let mut new_inside: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut events_to_broadcast: Vec<HubMessage> = Vec::new();

    for place in &places {
        let distance = haversine_distance(latitude, longitude, place.latitude, place.longitude);
        let is_inside = distance <= place.radius_meters;
        let was_inside = currently_inside.contains(&place.id.to_string());

        if is_inside {
            new_inside.insert(place.id.to_string());
        }

        // Detect enter event
        if is_inside && !was_inside {
            tracing::info!(
                "Geofence ENTER: user={} place={} (distance: {:.1}m, radius: {:.1}m)",
                user_id, place.name, distance, place.radius_meters
            );

            // Record event
            let _ = crate::db::geofence_events::record_geofence_event(
                &hub.db_pool,
                place.id,
                &place.name,
                circle_id,
                user_id,
                "enter",
                latitude,
                longitude,
            ).await;

            // Update state
            let _ = crate::db::geofence_events::mark_user_entered(
                &hub.db_pool,
                user_id,
                place.id,
                &place.name,
                circle_id,
            ).await;

            // Broadcast event
            events_to_broadcast.push(HubMessage::GeofenceEvent {
                place_name: place.name.clone(),
                user_id,
                circle_id,
                entered: true,
                latitude,
                longitude,
            });
        }

        // Detect exit event
        if !is_inside && was_inside {
            tracing::info!(
                "Geofence EXIT: user={} place={} (distance: {:.1}m)",
                user_id, place.name, distance
            );

            // Record event
            let _ = crate::db::geofence_events::record_geofence_event(
                &hub.db_pool,
                place.id,
                &place.name,
                circle_id,
                user_id,
                "exit",
                latitude,
                longitude,
            ).await;

            // Update state
            let _ = crate::db::geofence_events::mark_user_exited(
                &hub.db_pool,
                user_id,
                place.id,
            ).await;

            // Broadcast event
            events_to_broadcast.push(HubMessage::GeofenceEvent {
                place_name: place.name.clone(),
                user_id,
                circle_id,
                entered: false,
                latitude,
                longitude,
            });
        }
    }

    // Broadcast all geofence events
    for msg in events_to_broadcast {
        hub.broadcast(msg);
    }
}

/// Calculate Haversine distance between two coordinates in meters.
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS: f64 = 6_371_000.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}
