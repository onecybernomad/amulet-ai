-- Amulet AI — Initial PostgreSQL + PostGIS Migration

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    phone TEXT,
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    password_hash TEXT NOT NULL,
    circle_id UUID,
    subscription_tier TEXT DEFAULT 'free',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Circles
CREATE TABLE circles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    invite_code TEXT UNIQUE NOT NULL,
    max_members INT DEFAULT 4,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Circle members
CREATE TABLE circle_members (
    circle_id UUID REFERENCES circles(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role TEXT DEFAULT 'member',
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (circle_id, user_id)
);

-- Places (geofences)
CREATE TABLE places (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    circle_id UUID REFERENCES circles(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    address TEXT,
    geo_point GEOGRAPHY(POINT, 4326) NOT NULL,
    radius_meters INT DEFAULT 150,
    alert_on_arrival BOOLEAN DEFAULT TRUE,
    alert_on_departure BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Location pings (partitioned by time)
CREATE TABLE location_pings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    geo_point GEOGRAPHY(POINT, 4326) NOT NULL,
    accuracy_m FLOAT,
    speed_kmh FLOAT,
    heading FLOAT,
    battery INT,
    ts TIMESTAMPTZ DEFAULT NOW(),
    source TEXT DEFAULT 'gps'
);

CREATE INDEX idx_location_pings_user_ts ON location_pings(user_id, ts DESC);
CREATE INDEX idx_location_pings_geo ON location_pings USING GIST(geo_point);

-- Geofence events
CREATE TABLE geofence_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    place_id UUID REFERENCES places(id) ON DELETE CASCADE,
    event_type TEXT CHECK (event_type IN ('arrival', 'departure')),
    entered_at TIMESTAMPTZ,
    exited_at TIMESTAMPTZ,
    triggered_alert BOOLEAN DEFAULT FALSE
);

-- Incidents (crash / fall / SOS)
CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    incident_type TEXT CHECK (incident_type IN ('crash', 'fall', 'sos')),
    severity TEXT DEFAULT 'high',
    geo_point GEOGRAPHY(POINT, 4326),
    ts TIMESTAMPTZ DEFAULT NOW(),
    sensor_snapshot JSONB,
    status TEXT DEFAULT 'detected'
        CHECK (status IN ('detected', 'acknowledged', 'resolved', 'dispatched')),
    response_log JSONB DEFAULT '[]'
);

CREATE INDEX idx_incidents_user_ts ON incidents(user_id, ts DESC);

-- Medications
CREATE TABLE medications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    dosage TEXT,
    frequency TEXT,
    schedule JSONB,
    start_date DATE,
    end_date DATE,
    reminders_enabled BOOLEAN DEFAULT TRUE,
    adherence_log JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Chat rooms
CREATE TABLE chat_rooms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    circle_id UUID REFERENCES circles(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Chat messages
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    room_id UUID REFERENCES chat_rooms(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    media_urls TEXT[],
    ts TIMESTAMPTZ DEFAULT NOW(),
    encrypted BOOLEAN DEFAULT TRUE,
    read_by UUID[] DEFAULT '{}'
);

CREATE INDEX idx_chat_messages_room_ts ON chat_messages(room_id, ts DESC);

-- Driving sessions
CREATE TABLE driving_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    distance_km FLOAT,
    max_speed FLOAT,
    avg_speed FLOAT,
    events JSONB DEFAULT '[]',
    score INT,
    report_url TEXT
);

-- Subscriptions
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    stripe_customer_id TEXT,
    tier TEXT DEFAULT 'free',
    status TEXT DEFAULT 'active',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tile trackers
CREATE TABLE tile_trackers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tile_device_id TEXT NOT NULL,
    name TEXT,
    last_seen TIMESTAMPTZ,
    geo_point GEOGRAPHY(POINT, 4326),
    battery_level INT,
    status TEXT DEFAULT 'active'
);

-- Sessions (for auth)
CREATE TABLE sessions (
    token TEXT PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
