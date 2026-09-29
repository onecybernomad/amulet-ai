use crate::db::migrations::MIGRATIONS;
use crate::models::*;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Thread-safe local SQLite cache using rusqlite.
pub struct LocalCache {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for LocalCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalCache").finish()
    }
}

impl LocalCache {
    /// Create a new LocalCache, opening (or creating) the SQLite database.
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )?;

        let cache = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        cache.run_migrations()?;
        info!(path = %db_path.display(), "Local cache initialized");
        Ok(cache)
    }

    /// Return the default database path inside the app data directory.
    fn db_path() -> Result<PathBuf> {
        let mut path = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        path.push("com.amuletai.app");
        path.push("cache.db");
        Ok(path)
    }

    /// Run all pending migrations.
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(&MIGRATIONS.join("\n"))?;
        debug!("Migrations applied successfully");
        Ok(())
    }

    // ── Users ─────────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub fn insert_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, email, name, phone, avatar_url, password_hash, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                user.id,
                user.email,
                user.name,
                user.phone,
                user.avatar_url,
                user.password_hash,
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339(),
            ],
        )?;
        debug!(user_id = %user.id, "User inserted");
        Ok(())
    }

    pub fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, name, phone, avatar_url, password_hash, totp_secret, totp_enabled, created_at, updated_at
             FROM users WHERE id = ?1",
        )?;
        let user = stmt
            .query_row(params![id], Self::row_to_user)
            .optional()?;
        Ok(user)
    }

    pub fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, name, phone, avatar_url, password_hash, totp_secret, totp_enabled, created_at, updated_at
             FROM users WHERE email = ?1",
        )?;
        let user = stmt
            .query_row(params![email], Self::row_to_user)
            .optional()?;
        Ok(user)
    }

    pub fn get_current_user(&self) -> Result<Option<User>> {
        // In production, look up the user associated with the current session token
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, name, phone, avatar_url, password_hash, totp_secret, totp_enabled, created_at, updated_at
             FROM users ORDER BY created_at LIMIT 1",
        )?;
        let user = stmt
            .query_row([], Self::row_to_user)
            .optional()?;
        Ok(user)
    }

    fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
        Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            phone: row.get(3)?,
            avatar_url: row.get(4)?,
            password_hash: row.get(5)?,
            totp_secret: row.get(6)?,
            totp_enabled: row.get::<_, i64>(7)? != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── TOTP ─────────────────────────────────────────────────────────

    pub fn update_user_totp_secret(&self, user_id: &str, secret: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_secret = ?2, updated_at = ?3 WHERE id = ?1",
            params![user_id, secret, chrono::Utc::now().to_rfc3339()],
        )?;
        debug!(user_id = %user_id, "TOTP secret updated");
        Ok(())
    }

    pub fn enable_user_totp(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_enabled = 1, updated_at = ?2 WHERE id = ?1",
            params![user_id, chrono::Utc::now().to_rfc3339()],
        )?;
        debug!(user_id = %user_id, "TOTP enabled");
        Ok(())
    }

    pub fn disable_user_totp(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_enabled = 0, totp_secret = NULL, updated_at = ?2 WHERE id = ?1",
            params![user_id, chrono::Utc::now().to_rfc3339()],
        )?;
        debug!(user_id = %user_id, "TOTP disabled");
        Ok(())
    }

    // ── Sessions ──────────────────────────────────────────────────────

    pub fn store_session(&self, user_id: &str, token: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (token, user_id, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                token,
                user_id,
                chrono::Utc::now().to_rfc3339(),
                (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
            ],
        )?;
        debug!(user_id = %user_id, "Session stored");
        Ok(())
    }

    pub fn remove_session(&self, token: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
        debug!("Session removed");
        Ok(())
    }

    // ── Medications ───────────────────────────────────────────────────

    pub fn insert_medication(&self, med: &Medication) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO medications (id, user_id, name, dosage, frequency, time_of_day, notes, color, icon, active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                med.id,
                med.user_id,
                med.name,
                med.dosage,
                med.frequency,
                serde_json::to_string(&med.time_of_day)?,
                med.notes,
                med.color,
                med.icon,
                med.active,
                med.created_at.to_rfc3339(),
                med.updated_at.to_rfc3339(),
            ],
        )?;
        debug!(med_id = %med.id, "Medication inserted");
        Ok(())
    }

    pub fn get_medications(&self, user_id: &str) -> Result<Vec<Medication>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, dosage, frequency, time_of_day, notes, color, icon, active, created_at, updated_at
             FROM medications WHERE user_id = ?1 AND active = 1",
        )?;
        let meds = stmt
            .query_map(params![user_id], Self::row_to_medication)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(meds)
    }

    pub fn get_medication(&self, id: &str) -> Result<Option<Medication>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, dosage, frequency, time_of_day, notes, color, icon, active, created_at, updated_at
             FROM medications WHERE id = ?1",
        )?;
        let med = stmt
            .query_row(params![id], Self::row_to_medication)
            .optional()?;
        Ok(med)
    }

    pub fn update_medication(&self, med: &Medication) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE medications SET name = ?2, dosage = ?3, frequency = ?4, time_of_day = ?5,
             notes = ?6, color = ?7, icon = ?8, active = ?9, updated_at = ?10
             WHERE id = ?1",
            params![
                med.id,
                med.name,
                med.dosage,
                med.frequency,
                serde_json::to_string(&med.time_of_day)?,
                med.notes,
                med.color,
                med.icon,
                med.active,
                med.updated_at.to_rfc3339(),
            ],
        )?;
        debug!(med_id = %med.id, "Medication updated");
        Ok(())
    }

    pub fn delete_medication(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM medications WHERE id = ?1", params![id])?;
        debug!(med_id = %id, "Medication deleted");
        Ok(())
    }

    pub fn insert_dose_log(&self, log: &DoseLog) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO dose_logs (id, medication_id, user_id, status, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                log.id,
                log.medication_id,
                log.user_id,
                log.status,
                log.timestamp.to_rfc3339(),
            ],
        )?;
        debug!(log_id = %log.id, "Dose log inserted");
        Ok(())
    }

    fn row_to_medication(row: &rusqlite::Row) -> rusqlite::Result<Medication> {
        Ok(Medication {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            dosage: row.get(3)?,
            frequency: row.get(4)?,
            time_of_day: serde_json::from_str(&row.get::<_, String>(5)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?,
            notes: row.get(6)?,
            color: row.get(7)?,
            icon: row.get(8)?,
            active: row.get(9)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── Chat ──────────────────────────────────────────────────────────

    pub fn insert_chat_message(&self, msg: &ChatMessage) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_messages (id, room_id, sender_id, body, media_urls, read_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                msg.id,
                msg.room_id,
                msg.sender_id,
                msg.body,
                msg.media_urls.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
                serde_json::to_string(&msg.read_by)?,
                msg.created_at.to_rfc3339(),
            ],
        )?;
        debug!(msg_id = %msg.id, "Chat message inserted");
        Ok(())
    }

    pub fn get_chat_messages(
        &self,
        room_id: &str,
        before: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let before_str = before.map(|b| b.to_rfc3339());
        let mut stmt = conn.prepare(
            "SELECT id, room_id, sender_id, body, media_urls, read_by, created_at
             FROM chat_messages
             WHERE room_id = ?1 AND (?2 IS NULL OR created_at < ?2)
             ORDER BY created_at DESC
             LIMIT ?3",
        )?;
        let msgs = stmt
            .query_map(params![room_id, before_str, limit], Self::row_to_chat_message)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(msgs)
    }

    pub fn get_chat_rooms(&self, _user_id: &str) -> Result<Vec<ChatRoom>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, circle_id, created_at FROM chat_rooms",
        )?;
        let rooms = stmt
            .query_map([], Self::row_to_chat_room)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rooms)
    }

    fn row_to_chat_message(row: &rusqlite::Row) -> rusqlite::Result<ChatMessage> {
        let media_urls: Option<String> = row.get(4)?;
        let read_by: String = row.get(5)?;
        Ok(ChatMessage {
            id: row.get(0)?,
            room_id: row.get(1)?,
            sender_id: row.get(2)?,
            body: row.get(3)?,
            media_urls: media_urls
                .map(|s| serde_json::from_str(&s).unwrap_or_default()),
            read_by: serde_json::from_str(&read_by)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    fn row_to_chat_room(row: &rusqlite::Row) -> rusqlite::Result<ChatRoom> {
        Ok(ChatRoom {
            id: row.get(0)?,
            name: row.get(1)?,
            circle_id: row.get(2)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── Location Pings ────────────────────────────────────────────────

    pub fn insert_location_ping(&self, ping: &LocationPing) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cached_locations (id, user_id, lat, lng, accuracy, speed, heading, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                ping.id,
                ping.user_id,
                ping.lat,
                ping.lng,
                ping.accuracy,
                ping.speed,
                ping.heading,
                ping.timestamp.to_rfc3339(),
            ],
        )?;
        debug!(user_id = %ping.user_id, "Location ping cached");
        Ok(())
    }

    pub fn get_latest_location(&self, user_id: &str) -> Result<Option<LocationPing>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, lat, lng, accuracy, speed, heading, timestamp
             FROM cached_locations WHERE user_id = ?1
             ORDER BY timestamp DESC LIMIT 1",
        )?;
        let ping = stmt
            .query_row(params![user_id], Self::row_to_location_ping)
            .optional()?;
        Ok(ping)
    }

    pub fn get_all_latest_locations(&self) -> Result<Vec<LocationPing>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT cl1.id, cl1.user_id, cl1.lat, cl1.lng, cl1.accuracy, cl1.speed, cl1.heading, cl1.timestamp
             FROM cached_locations cl1
             INNER JOIN (
                 SELECT user_id, MAX(timestamp) as max_ts
                 FROM cached_locations
                 GROUP BY user_id
             ) cl2 ON cl1.user_id = cl2.user_id AND cl1.timestamp = cl2.max_ts",
        )?;
        let pings = stmt
            .query_map([], Self::row_to_location_ping)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(pings)
    }

    fn row_to_location_ping(row: &rusqlite::Row) -> rusqlite::Result<LocationPing> {
        Ok(LocationPing {
            id: row.get(0)?,
            user_id: row.get(1)?,
            lat: row.get(2)?,
            lng: row.get(3)?,
            accuracy: row.get(4)?,
            speed: row.get(5)?,
            heading: row.get(6)?,
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── Incidents ─────────────────────────────────────────────────────

    pub fn insert_incident(&self, incident: &Incident) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO incidents (id, user_id, incident_type, lat, lng, severity, description, status, created_at, resolved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                incident.id,
                incident.user_id,
                incident.incident_type,
                incident.lat,
                incident.lng,
                incident.severity,
                incident.description,
                incident.status,
                incident.created_at.to_rfc3339(),
                incident.resolved_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        debug!(incident_id = %incident.id, "Incident inserted");
        Ok(())
    }

    pub fn update_incident_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE incidents SET status = ?2, resolved_at = ?3 WHERE id = ?1",
            params![
                id,
                status,
                if status == "resolved" || status == "cancelled" {
                    Some(chrono::Utc::now().to_rfc3339())
                } else {
                    None
                },
            ],
        )?;
        debug!(incident_id = %id, status = %status, "Incident status updated");
        Ok(())
    }

    // ── Places ────────────────────────────────────────────────────────

    pub fn insert_place(&self, place: &Place) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO places (id, user_id, name, lat, lng, radius, place_type, notify_on_enter, notify_on_exit, active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                place.id,
                place.user_id,
                place.name,
                place.lat,
                place.lng,
                place.radius,
                place.place_type,
                place.notify_on_enter,
                place.notify_on_exit,
                place.active,
                place.created_at.to_rfc3339(),
            ],
        )?;
        debug!(place_id = %place.id, "Place inserted");
        Ok(())
    }

    pub fn get_places(&self, user_id: &str) -> Result<Vec<Place>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, lat, lng, radius, place_type, notify_on_enter, notify_on_exit, active, created_at
             FROM places WHERE user_id = ?1",
        )?;
        let places = stmt
            .query_map(params![user_id], Self::row_to_place)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(places)
    }

    pub fn get_active_places(&self, user_id: &str) -> Result<Vec<Place>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, lat, lng, radius, place_type, notify_on_enter, notify_on_exit, active, created_at
             FROM places WHERE user_id = ?1 AND active = 1",
        )?;
        let places = stmt
            .query_map(params![user_id], Self::row_to_place)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(places)
    }

    pub fn get_place(&self, id: &str) -> Result<Option<Place>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, lat, lng, radius, place_type, notify_on_enter, notify_on_exit, active, created_at
             FROM places WHERE id = ?1",
        )?;
        let place = stmt
            .query_row(params![id], Self::row_to_place)
            .optional()?;
        Ok(place)
    }

    pub fn update_place(&self, place: &Place) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE places SET name = ?2, lat = ?3, lng = ?4, radius = ?5, place_type = ?6,
             notify_on_enter = ?7, notify_on_exit = ?8, active = ?9
             WHERE id = ?1",
            params![
                place.id,
                place.name,
                place.lat,
                place.lng,
                place.radius,
                place.place_type,
                place.notify_on_enter,
                place.notify_on_exit,
                place.active,
            ],
        )?;
        debug!(place_id = %place.id, "Place updated");
        Ok(())
    }

    pub fn delete_place(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM places WHERE id = ?1", params![id])?;
        debug!(place_id = %id, "Place deleted");
        Ok(())
    }

    fn row_to_place(row: &rusqlite::Row) -> rusqlite::Result<Place> {
        Ok(Place {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            lat: row.get(3)?,
            lng: row.get(4)?,
            radius: row.get(5)?,
            place_type: row.get(6)?,
            notify_on_enter: row.get(7)?,
            notify_on_exit: row.get(8)?,
            active: row.get(9)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── Tile Trackers ──────────────────────────────────────────────────

    pub fn insert_tile(&self, tile: &TileTracker) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tile_trackers (id, user_id, tile_id, name, device_type, battery_level, last_location, last_seen, ring_status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                tile.id,
                tile.user_id,
                tile.tile_id,
                tile.name,
                tile.device_type,
                tile.battery_level,
                tile.last_location.as_ref().map(|loc| serde_json::to_string(loc).unwrap_or_default()),
                tile.last_seen.map(|t| t.to_rfc3339()),
                tile.ring_status,
                tile.created_at.to_rfc3339(),
            ],
        )?;
        debug!(tile_id = %tile.id, "Tile tracker inserted");
        Ok(())
    }

    pub fn get_tiles(&self, user_id: &str) -> Result<Vec<TileTracker>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, tile_id, name, device_type, battery_level, last_location, last_seen, ring_status, created_at
             FROM tile_trackers WHERE user_id = ?1",
        )?;
        let tiles = stmt
            .query_map(params![user_id], Self::row_to_tile)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(tiles)
    }

    pub fn get_tile(&self, id: &str) -> Result<Option<TileTracker>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, tile_id, name, device_type, battery_level, last_location, last_seen, ring_status, created_at
             FROM tile_trackers WHERE id = ?1",
        )?;
        let tile = stmt
            .query_row(params![id], Self::row_to_tile)
            .optional()?;
        Ok(tile)
    }

    pub fn update_tile(&self, tile: &TileTracker) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tile_trackers SET name = ?2, device_type = ?3, battery_level = ?4,
             last_location = ?5, last_seen = ?6, ring_status = ?7
             WHERE id = ?1",
            params![
                tile.id,
                tile.name,
                tile.device_type,
                tile.battery_level,
                tile.last_location.as_ref().map(|loc| serde_json::to_string(loc).unwrap_or_default()),
                tile.last_seen.map(|t| t.to_rfc3339()),
                tile.ring_status,
            ],
        )?;
        debug!(tile_db_id = %tile.id, "Tile tracker updated");
        Ok(())
    }

    pub fn delete_tile(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tile_trackers WHERE id = ?1", params![id])?;
        debug!(tile_db_id = %id, "Tile tracker deleted");
        Ok(())
    }

    fn row_to_tile(row: &rusqlite::Row) -> rusqlite::Result<TileTracker> {
        let last_location: Option<String> = row.get(6)?;
        let last_seen: Option<String> = row.get(7)?;
        Ok(TileTracker {
            id: row.get(0)?,
            user_id: row.get(1)?,
            tile_id: row.get(2)?,
            name: row.get(3)?,
            device_type: row.get(4)?,
            battery_level: row.get(5)?,
            last_location: last_location
                .map(|s| serde_json::from_str(&s).unwrap_or_default()),
            last_seen: last_seen
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            ring_status: row.get(8)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }

    // ── Subscriptions ─────────────────────────────────────────────────

    pub fn insert_subscription(&self, sub: &Subscription) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO subscriptions (id, user_id, plan_id, stripe_subscription_id, stripe_customer_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                sub.id,
                sub.user_id,
                sub.plan_id,
                sub.stripe_subscription_id,
                sub.stripe_customer_id,
                sub.status,
                sub.current_period_start.to_rfc3339(),
                sub.current_period_end.to_rfc3339(),
                sub.cancel_at_period_end,
                sub.created_at.to_rfc3339(),
            ],
        )?;
        debug!(sub_id = %sub.id, "Subscription inserted");
        Ok(())
    }

    pub fn get_active_subscription(&self, user_id: &str) -> Result<Option<Subscription>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, plan_id, stripe_subscription_id, stripe_customer_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at
             FROM subscriptions WHERE user_id = ?1 AND status = 'active' LIMIT 1",
        )?;
        let sub = stmt
            .query_row(params![user_id], Self::row_to_subscription)
            .optional()?;
        Ok(sub)
    }

    pub fn cancel_subscription(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE subscriptions SET cancel_at_period_end = 1 WHERE user_id = ?1 AND status = 'active'",
            params![user_id],
        )?;
        debug!(user_id = %user_id, "Subscription cancellation requested");
        Ok(())
    }

    fn row_to_subscription(row: &rusqlite::Row) -> rusqlite::Result<Subscription> {
        Ok(Subscription {
            id: row.get(0)?,
            user_id: row.get(1)?,
            plan_id: row.get(2)?,
            stripe_subscription_id: row.get(3)?,
            stripe_customer_id: row.get(4)?,
            status: row.get(5)?,
            current_period_start: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
            current_period_end: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
            cancel_at_period_end: row.get(8)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&chrono::Utc),
        })
    }
}

// Make LocalCache cloneable for Tauri state management
impl Clone for LocalCache {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
