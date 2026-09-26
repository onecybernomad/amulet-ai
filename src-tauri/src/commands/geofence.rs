use crate::db::local_cache::LocalCache;
use crate::models::Place;
use geo::{point, GeodesicDistance};
use serde::Deserialize;
use tauri::State;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddPlaceRequest {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub radius: f64,
    pub place_type: String, // "home" | "work" | "school" | "custom"
    pub notify_on_enter: bool,
    pub notify_on_exit: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaceRequest {
    pub id: String,
    pub name: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius: Option<f64>,
    pub place_type: Option<String>,
    pub notify_on_enter: Option<bool>,
    pub notify_on_exit: Option<bool>,
    pub active: Option<bool>,
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Check if a coordinate is inside any geofence.
#[tauri::command]
pub async fn check_geofence(
    lat: f64,
    lng: f64,
    cache: State<'_, LocalCache>,
) -> Result<Vec<Place>, String> {
    info!(lat, lng, "Checking geofence");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let places = cache.get_active_places(&user.id).map_err(|e| e.to_string())?;
    let point_of_interest = point!(x: lng, y: lat);

    let mut inside: Vec<Place> = Vec::new();
    for place in places {
        let place_point = point!(x: place.lng, y: place.lat);
        let distance = place_point.geodesic_distance(&point_of_interest);
        if distance <= place.radius {
            debug!(place_name = %place.name, distance_m = %distance, "Inside geofence");
            inside.push(place);
        }
    }

    Ok(inside)
}

/// Get all places for the current user.
#[tauri::command]
pub async fn get_places(
    cache: State<'_, LocalCache>,
) -> Result<Vec<Place>, String> {
    info!("Getting places");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;
    let places = cache.get_places(&user.id).map_err(|e| e.to_string())?;
    debug!(count = places.len(), "Retrieved places");
    Ok(places)
}

/// Add a new geofence place.
#[tauri::command]
pub async fn add_place(
    req: AddPlaceRequest,
    cache: State<'_, LocalCache>,
) -> Result<Place, String> {
    info!(name = %req.name, lat = %req.lat, lng = %req.lng, "Adding place");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let place = Place {
        id: Uuid::new_v4().to_string(),
        user_id: user.id,
        name: req.name,
        lat: req.lat,
        lng: req.lng,
        radius: req.radius,
        place_type: req.place_type,
        notify_on_enter: req.notify_on_enter,
        notify_on_exit: req.notify_on_exit,
        active: true,
        created_at: chrono::Utc::now(),
    };

    cache.insert_place(&place).map_err(|e| e.to_string())?;
    Ok(place)
}

/// Update an existing place.
#[tauri::command]
pub async fn update_place(
    req: UpdatePlaceRequest,
    cache: State<'_, LocalCache>,
) -> Result<Place, String> {
    info!(place_id = %req.id, "Updating place");
    let mut place = cache
        .get_place(&req.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Place not found".to_string())?;

    if let Some(name) = req.name {
        place.name = name;
    }
    if let Some(lat) = req.lat {
        place.lat = lat;
    }
    if let Some(lng) = req.lng {
        place.lng = lng;
    }
    if let Some(radius) = req.radius {
        place.radius = radius;
    }
    if let Some(place_type) = req.place_type {
        place.place_type = place_type;
    }
    if let Some(notify_on_enter) = req.notify_on_enter {
        place.notify_on_enter = notify_on_enter;
    }
    if let Some(notify_on_exit) = req.notify_on_exit {
        place.notify_on_exit = notify_on_exit;
    }
    if let Some(active) = req.active {
        place.active = active;
    }

    cache.update_place(&place).map_err(|e| e.to_string())?;
    Ok(place)
}

/// Delete a place.
#[tauri::command]
pub async fn delete_place(
    place_id: String,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!(place_id = %place_id, "Deleting place");
    cache.delete_place(&place_id).map_err(|e| e.to_string())?;
    Ok(())
}
