use crate::db::local_cache::LocalCache;
use crate::models::TileTracker;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LinkTileRequest {
    pub tile_id: String,
    pub name: String,
    pub device_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TileStatus {
    pub tile_id: String,
    pub connected: bool,
    pub battery_level: Option<i32>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Link a Tile tracker to the current user.
#[tauri::command]
pub async fn link_tile(
    req: LinkTileRequest,
    cache: State<'_, LocalCache>,
) -> Result<TileTracker, String> {
    info!(tile_id = %req.tile_id, name = %req.name, "Linking Tile");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let tile = TileTracker {
        id: Uuid::new_v4().to_string(),
        user_id: user.id,
        tile_id: req.tile_id,
        name: req.name,
        device_type: req.device_type,
        battery_level: None,
        last_location: None,
        last_seen: None,
        ring_status: "idle".into(),
        created_at: chrono::Utc::now(),
    };

    cache.insert_tile(&tile).map_err(|e| e.to_string())?;
    Ok(tile)
}

/// Unlink a Tile tracker.
#[tauri::command]
pub async fn unlink_tile(
    tile_db_id: String,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!(tile_db_id = %tile_db_id, "Unlinking Tile");
    cache.delete_tile(&tile_db_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all Tile trackers for the current user.
#[tauri::command]
pub async fn get_tiles(
    cache: State<'_, LocalCache>,
) -> Result<Vec<TileTracker>, String> {
    info!("Getting Tile trackers");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;
    let tiles = cache.get_tiles(&user.id).map_err(|e| e.to_string())?;
    debug!(count = tiles.len(), "Retrieved Tile trackers");
    Ok(tiles)
}

/// Ring a Tile tracker.
#[tauri::command]
pub async fn ring_tile(
    tile_db_id: String,
    cache: State<'_, LocalCache>,
) -> Result<TileStatus, String> {
    info!(tile_db_id = %tile_db_id, "Ringing Tile");
    let mut tile = cache
        .get_tile(&tile_db_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Tile not found".to_string())?;

    tile.ring_status = "ringing".into();
    tile.last_seen = Some(chrono::Utc::now());
    cache.update_tile(&tile).map_err(|e| e.to_string())?;

    // In production: call Tile API to trigger ring
    Ok(TileStatus {
        tile_id: tile.tile_id,
        connected: true,
        battery_level: tile.battery_level,
        last_seen: tile.last_seen,
    })
}

/// Locate a Tile tracker (get last known location).
#[tauri::command]
pub async fn locate_tile(
    tile_db_id: String,
    cache: State<'_, LocalCache>,
) -> Result<TileStatus, String> {
    info!(tile_db_id = %tile_db_id, "Locating Tile");
    let tile = cache
        .get_tile(&tile_db_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Tile not found".to_string())?;

    // In production: call Tile API for current location
    Ok(TileStatus {
        tile_id: tile.tile_id,
        connected: tile.last_seen.map(|t| {
            chrono::Utc::now().signed_duration_since(t).num_minutes() < 30
        }).unwrap_or(false),
        battery_level: tile.battery_level,
        last_seen: tile.last_seen,
    })
}
