use super::ws_client::ServerMessage;
use tracing::{debug, info, instrument};

/// Dispatch an incoming server message to the appropriate handler.
#[instrument(skip(msg))]
pub async fn handle_message(msg: ServerMessage) {
    match msg {
        ServerMessage::MemberLocation {
            member_id,
            name,
            lat,
            lng,
            accuracy,
            timestamp,
        } => {
            info!(member_id = %member_id, "Member location update");
            // In production: emit to the Tauri app handle
            let _ = (name, lat, lng, accuracy, timestamp);
        }

        ServerMessage::GeofenceAlert {
            place_id,
            place_name,
            member_id,
            event,
            timestamp,
        } => {
            info!(
                place_name = %place_name,
                member_id = %member_id,
                event = %event,
                "Geofence alert"
            );
            let _ = (place_id, timestamp);
        }

        ServerMessage::SosAlert {
            alert_id,
            user_id,
            user_name,
            lat,
            lng,
            silent,
            timestamp,
        } => {
            info!(
                user_name = %user_name,
                silent,
                "SOS alert received"
            );
            let _ = (alert_id, user_id, lat, lng, timestamp);
        }

        ServerMessage::ChatMessage {
            room_id,
            sender_id,
            body,
            media_urls,
            timestamp,
        } => {
            info!(room_id = %room_id, sender_id = %sender_id, "Chat message received");
            let _ = (body, media_urls, timestamp);
        }

        ServerMessage::IncidentAlert {
            incident_id,
            user_id,
            incident_type,
            lat,
            lng,
            severity,
            timestamp,
        } => {
            info!(
                incident_type = %incident_type,
                severity = %severity,
                "Incident alert received"
            );
            let _ = (incident_id, user_id, lat, lng, timestamp);
        }

        ServerMessage::Pong => {
            debug!("Pong received");
        }
    }
}
