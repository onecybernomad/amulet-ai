-- Amulet AI — SQLite Migration (for development)
-- For PostgreSQL + PostGIS, use 001_initial.sql instead

-- Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    phone TEXT,
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    password_hash TEXT NOT NULL,
    circle_id TEXT,
    subscription_tier TEXT DEFAULT 'free',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Circles
CREATE TABLE IF NOT EXISTS circles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    invite_code TEXT UNIQUE NOT NULL,
    max_members INTEGER DEFAULT 4,
    created_by TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Circle members
CREATE TABLE IF NOT EXISTS circle_members (
    circle_id TEXT,
    user_id TEXT,
    role TEXT DEFAULT 'member',
    joined_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY (circle_id, user_id)
);

-- Places (geofences) — lat/lng as REAL for SQLite (no PostGIS)
CREATE TABLE IF NOT EXISTS places (
    id TEXT PRIMARY KEY,
    circle_id TEXT,
    name TEXT NOT NULL,
    address TEXT,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    radius_meters INTEGER DEFAULT 150,
    alert_on_arrival INTEGER DEFAULT 1,
    alert_on_departure INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Location pings
CREATE TABLE IF NOT EXISTS location_pings (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    device_id TEXT NOT NULL,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    accuracy_m REAL,
    speed_kmh REAL,
    heading REAL,
    battery INTEGER,
    ts TEXT DEFAULT (datetime('now')),
    source TEXT DEFAULT 'gps'
);

CREATE INDEX IF NOT EXISTS idx_location_pings_user_ts ON location_pings(user_id, ts DESC);

-- Geofence events
CREATE TABLE IF NOT EXISTS geofence_events (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    place_id TEXT,
    event_type TEXT CHECK (event_type IN ('arrival', 'departure')),
    entered_at TEXT,
    exited_at TEXT,
    triggered_alert INTEGER DEFAULT 0
);

-- Incidents (crash / fall / SOS)
CREATE TABLE IF NOT EXISTS incidents (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    incident_type TEXT CHECK (incident_type IN ('crash', 'fall', 'sos')),
    severity TEXT DEFAULT 'high',
    lat REAL,
    lng REAL,
    ts TEXT DEFAULT (datetime('now')),
    sensor_snapshot TEXT,
    status TEXT DEFAULT 'detected'
        CHECK (status IN ('detected', 'acknowledged', 'resolved', 'dispatched')),
    response_log TEXT DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_incidents_user_ts ON incidents(user_id, ts DESC);

-- Medications
CREATE TABLE IF NOT EXISTS medications (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    name TEXT NOT NULL,
    dosage TEXT,
    frequency TEXT,
    schedule TEXT,
    start_date TEXT,
    end_date TEXT,
    reminders_enabled INTEGER DEFAULT 1,
    adherence_log TEXT DEFAULT '[]',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Chat rooms
CREATE TABLE IF NOT EXISTS chat_rooms (
    id TEXT PRIMARY KEY,
    circle_id TEXT,
    name TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Chat messages
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    room_id TEXT,
    sender_id TEXT,
    body TEXT NOT NULL,
    media_urls TEXT,
    ts TEXT DEFAULT (datetime('now')),
    encrypted INTEGER DEFAULT 1,
    read_by TEXT DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_room_ts ON chat_messages(room_id, ts DESC);

-- Driving sessions
CREATE TABLE IF NOT EXISTS driving_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    start_time TEXT,
    end_time TEXT,
    distance_km REAL,
    max_speed REAL,
    avg_speed REAL,
    events TEXT DEFAULT '[]',
    score INTEGER,
    report_url TEXT
);

-- Subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    stripe_customer_id TEXT,
    tier TEXT DEFAULT 'free',
    status TEXT DEFAULT 'active',
    current_period_start TEXT,
    current_period_end TEXT,
    cancel_at_period_end INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Tile trackers
CREATE TABLE IF NOT EXISTS tile_trackers (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    tile_device_id TEXT NOT NULL,
    name TEXT,
    last_seen TEXT,
    lat REAL,
    lng REAL,
    battery_level INTEGER,
    status TEXT DEFAULT 'active'
);

-- Sessions (for auth)
CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    user_id TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);
