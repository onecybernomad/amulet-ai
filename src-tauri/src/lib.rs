mod commands;
mod db;
mod models;
mod push;
mod realtime;
mod sensors;

use tauri::Manager;
use tauri_plugin_store::StoreBuilder;
use tracing::{debug, info};

/// Run the Amulet AI Tauri application.
pub fn run() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "amulet_ai_lib=info,tauri=warn".into()),
        )
        .init();

    info!("Starting Amulet AI v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        // ── Plugins ──────────────────────────────────────────────────────
        .plugin(tauri_plugin_geolocation::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        // ── State ────────────────────────────────────────────────────────
        .manage(db::LocalCache::new().expect("Failed to initialize local cache"))
        .manage(sensors::SensorState::default())
        // ── Setup ───────────────────────────────────────────────────────
        .setup(|app| {
            let handle = app.handle().clone();

            // Persist session / settings in a Tauri store
            let store_path = handle
                .path()
                .resolve("store.json", tauri::path::BaseDirectory::AppData)
                .map_err(|e| tauri::Error::Io(std::io::Error::other(e)))?;
            let _store = StoreBuilder::new(&handle, store_path).build();

            // Check geolocation permissions (desktop fallback)
            #[cfg(desktop)]
            {
                use tauri_plugin_geolocation::GeolocationExt;
                let _ = app.geolocation().check_permissions()?;
            }

            debug!("Tauri application initialized");
            Ok(())
        })
        // ── Commands ─────────────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::login,
            commands::auth::register,
            commands::auth::verify_otp,
            commands::auth::logout,
            // Location
            commands::location::start_location_tracking,
            commands::location::stop_location_tracking,
            commands::location::get_current_location,
            commands::location::get_member_locations,
            // SOS
            commands::sos::trigger_sos,
            commands::sos::cancel_sos,
            commands::sos::get_active_sos,
            // Medications
            commands::medications::get_medications,
            commands::medications::add_medication,
            commands::medications::update_medication,
            commands::medications::delete_medication,
            commands::medications::log_dose,
            // Chat
            commands::chat::get_rooms,
            commands::chat::get_messages,
            commands::chat::send_message,
            // Tile
            commands::tile::link_tile,
            commands::tile::unlink_tile,
            commands::tile::get_tiles,
            commands::tile::ring_tile,
            commands::tile::locate_tile,
            // Subscriptions
            commands::subscriptions::get_plans,
            commands::subscriptions::subscribe,
            commands::subscriptions::cancel_subscription,
            commands::subscriptions::get_current_subscription,
            // Geofence
            commands::geofence::check_geofence,
            commands::geofence::get_places,
            commands::geofence::add_place,
            commands::geofence::update_place,
            commands::geofence::delete_place,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Amulet AI");
}
