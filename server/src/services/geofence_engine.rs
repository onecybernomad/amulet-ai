use geo::{Distance, Haversine, Point};
use crate::types::Place;

/// Result of a geofence check.
#[derive(Debug, Clone)]
pub struct GeofenceEvent {
    pub place_name: String,
    pub entered: bool,
}

/// Check if a user's location triggers any geofence events.
/// Uses the `geo` crate for client-side distance calculation.
/// For production, consider using PostGIS ST_DWithin directly in the database.
pub fn check(lat: f64, lng: f64, places: &[Place]) -> Option<GeofenceEvent> {
    let user_point = Point::new(lng, lat);

    for place in places {
        let place_point = Point::new(place.longitude, place.latitude);
        let distance = Haversine::distance(user_point, place_point);

        if distance <= place.radius_meters {
            tracing::info!(
                "Geofence event: user entered place '{}' (distance: {:.1}m, radius: {:.1}m)",
                place.name,
                distance,
                place.radius_meters
            );
            return Some(GeofenceEvent {
                place_name: place.name.clone(),
                entered: true,
            });
        }
    }

    None
}

/// Check if a user's location triggers any geofence exit events.
pub fn check_exit(lat: f64, lng: f64, places: &[Place], previously_inside: &[String]) -> Vec<GeofenceEvent> {
    let user_point = Point::new(lng, lat);
    let mut events = Vec::new();

    for place_name in previously_inside {
        if let Some(place) = places.iter().find(|p| &p.name == place_name) {
            let place_point = Point::new(place.longitude, place.latitude);
            let distance = Haversine::distance(user_point, place_point);

            if distance > place.radius_meters {
                tracing::info!(
                    "Geofence exit event: user left place '{}' (distance: {:.1}m)",
                    place.name,
                    distance
                );
                events.push(GeofenceEvent {
                    place_name: place.name.clone(),
                    entered: false,
                });
            }
        }
    }

    events
}
