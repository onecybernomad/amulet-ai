use chrono::TimeZone;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub mod circles;
pub mod chat;
pub mod driving;
pub mod geofence;
pub mod geofence_events;
pub mod incident_responses;
pub mod incidents;
pub mod locations;
pub mod medications;
pub mod subscriptions;
pub mod users;

/// Database wrapper holding the connection pool.
/// Uses SQLite for local development.
#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new database pool from a connection string.
    /// Automatically creates tables if they don't exist.
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Create tables if they don't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                phone TEXT,
                display_name TEXT NOT NULL,
                avatar_url TEXT,
                password_hash TEXT NOT NULL,
                subscription_tier TEXT DEFAULT 'free',
                failed_login_attempts INTEGER DEFAULT 0,
                locked_until TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Add columns if they don't exist (for existing databases)
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN locked_until TEXT").execute(&pool).await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS circles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                invite_code TEXT UNIQUE NOT NULL,
                owner_id TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS circle_members (
                circle_id TEXT,
                user_id TEXT,
                role TEXT DEFAULT 'member',
                joined_at TEXT DEFAULT (datetime('now')),
                PRIMARY KEY (circle_id, user_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS places (
                id TEXT PRIMARY KEY,
                circle_id TEXT,
                name TEXT NOT NULL,
                lat REAL NOT NULL,
                lng REAL NOT NULL,
                radius_meters INTEGER DEFAULT 150,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS incidents (
                id TEXT PRIMARY KEY,
                circle_id TEXT,
                user_id TEXT,
                incident_type TEXT,
                lat REAL,
                lng REAL,
                status TEXT DEFAULT 'detected',
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS medications (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                name TEXT NOT NULL,
                dosage TEXT,
                schedule TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dose_logs (
                id TEXT PRIMARY KEY,
                medication_id TEXT,
                user_id TEXT,
                taken INTEGER DEFAULT 1,
                notes TEXT,
                timestamp TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_rooms (
                id TEXT PRIMARY KEY,
                circle_id TEXT,
                name TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                room_id TEXT,
                sender_id TEXT,
                body TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS driving_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                started_at TEXT,
                ended_at TEXT,
                distance_km REAL,
                max_speed REAL,
                avg_speed REAL,
                harsh_braking_count INTEGER DEFAULT 0,
                rapid_acceleration_count INTEGER DEFAULT 0,
                phone_usage_count INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS subscriptions (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                stripe_customer_id TEXT,
                stripe_subscription_id TEXT,
                tier TEXT DEFAULT 'free',
                status TEXT DEFAULT 'active',
                current_period_start TEXT,
                current_period_end TEXT,
                cancel_at_period_end INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tile_trackers (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                tile_id TEXT NOT NULL,
                name TEXT,
                device_type TEXT,
                battery_level INTEGER,
                last_lat REAL,
                last_lng REAL,
                last_seen TEXT,
                ring_status TEXT DEFAULT 'ok',
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS location_pings (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                circle_id TEXT,
                lat REAL NOT NULL,
                lng REAL NOT NULL,
                accuracy REAL,
                speed REAL,
                heading REAL,
                timestamp TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS password_resets (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                used INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_verifications (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                email TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                verified INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS geofence_events (
                id TEXT PRIMARY KEY,
                place_id TEXT NOT NULL,
                place_name TEXT NOT NULL,
                circle_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                event_type TEXT NOT NULL CHECK(event_type IN ('enter', 'exit')),
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                notified INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS incident_responses (
                id TEXT PRIMARY KEY,
                incident_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                note TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Track which places each user is currently inside (for exit detection)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_geofence_state (
                user_id TEXT NOT NULL,
                place_id TEXT NOT NULL,
                place_name TEXT NOT NULL,
                circle_id TEXT NOT NULL,
                entered_at TEXT DEFAULT (datetime('now')),
                PRIMARY KEY (user_id, place_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self::new(pool))
    }
}

/// Parse a datetime string from SQLite, trying multiple formats.
/// SQLite's `datetime('now')` returns "YYYY-MM-DD HH:MM:SS" format,
/// but some values may be stored as RFC3339.
pub fn parse_datetime(s: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    // Try RFC3339 first
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }
    // Try SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(chrono::Utc.from_utc_datetime(&dt));
    }
    // Try with fractional seconds
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
        return Ok(chrono::Utc.from_utc_datetime(&dt));
    }
    anyhow::bail!("Cannot parse datetime: {}", s)
}
