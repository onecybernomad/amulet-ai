//! SQL migrations for the local SQLite database.

pub const MIGRATIONS: &[&str] = &[
    // ── Users ──────────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        phone TEXT,
        avatar_url TEXT,
        password_hash TEXT NOT NULL,
        totp_secret TEXT,
        totp_enabled INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
    "#,
    // ── Sessions ───────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS sessions (
        token TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        expires_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    "#,
    // ── Medications ────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS medications (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        name TEXT NOT NULL,
        dosage TEXT NOT NULL,
        frequency TEXT NOT NULL,
        time_of_day TEXT NOT NULL,
        notes TEXT,
        color TEXT,
        icon TEXT,
        active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_medications_user ON medications(user_id);
    "#,
    // ── Dose Logs ──────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS dose_logs (
        id TEXT PRIMARY KEY,
        medication_id TEXT NOT NULL,
        user_id TEXT NOT NULL,
        status TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (medication_id) REFERENCES medications(id) ON DELETE CASCADE,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_dose_logs_med ON dose_logs(medication_id);
    "#,
    // ── Chat Rooms ─────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS chat_rooms (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        circle_id TEXT,
        created_at TEXT NOT NULL
    );
    "#,
    // ── Chat Messages ──────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS chat_messages (
        id TEXT PRIMARY KEY,
        room_id TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        body TEXT NOT NULL,
        media_urls TEXT,
        read_by TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY (room_id) REFERENCES chat_rooms(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_chat_messages_room ON chat_messages(room_id);
    "#,
    // ── Location Pings (cached) ────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS cached_locations (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        lat REAL NOT NULL,
        lng REAL NOT NULL,
        accuracy REAL NOT NULL,
        speed REAL,
        heading REAL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_cached_locations_user ON cached_locations(user_id);
    CREATE INDEX IF NOT EXISTS idx_cached_locations_timestamp ON cached_locations(timestamp);
    "#,
    // ── Incidents ──────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS incidents (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        incident_type TEXT NOT NULL,
        lat REAL NOT NULL,
        lng REAL NOT NULL,
        severity TEXT NOT NULL,
        description TEXT,
        status TEXT NOT NULL DEFAULT 'active',
        created_at TEXT NOT NULL,
        resolved_at TEXT,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_incidents_user ON incidents(user_id);
    CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents(status);
    "#,
    // ── Places (Geofences) ─────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS places (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        name TEXT NOT NULL,
        lat REAL NOT NULL,
        lng REAL NOT NULL,
        radius REAL NOT NULL,
        place_type TEXT NOT NULL,
        notify_on_enter INTEGER NOT NULL DEFAULT 1,
        notify_on_exit INTEGER NOT NULL DEFAULT 1,
        active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_places_user ON places(user_id);
    "#,
    // ── Tile Trackers ──────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS tile_trackers (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        tile_id TEXT NOT NULL,
        name TEXT NOT NULL,
        device_type TEXT,
        battery_level INTEGER,
        last_location TEXT,
        last_seen TEXT,
        ring_status TEXT NOT NULL DEFAULT 'idle',
        created_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_tile_trackers_user ON tile_trackers(user_id);
    "#,
    // ── Subscriptions ──────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS subscriptions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        plan_id TEXT NOT NULL,
        stripe_subscription_id TEXT,
        stripe_customer_id TEXT,
        status TEXT NOT NULL DEFAULT 'active',
        current_period_start TEXT NOT NULL,
        current_period_end TEXT NOT NULL,
        cancel_at_period_end INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON subscriptions(user_id);
    "#,
    // ── Settings ───────────────────────────────────────────────────────
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    "#,
];
