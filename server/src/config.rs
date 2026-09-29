use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub otp_issuer: String,
    pub server_host: String,
    pub server_port: u16,
    pub stripe_secret_key: String,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_bundle_id: String,
    pub fcm_api_key: String,
}

impl Config {
    /// Load configuration from environment variables (with optional `.env` file).
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./amulet_ai.db?mode=rwc".into()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-do-not-use-in-production".into()),
            access_token_ttl_minutes: env::var("ACCESS_TOKEN_TTL_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15),
            refresh_token_ttl_days: env::var("REFRESH_TOKEN_TTL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            otp_issuer: env::var("OTP_ISSUER")
                .unwrap_or_else(|_| "Amulet AI".into()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".into()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            apns_key_id: env::var("APNS_KEY_ID").unwrap_or_default(),
            apns_team_id: env::var("APNS_TEAM_ID").unwrap_or_default(),
            apns_bundle_id: env::var("APNS_BUNDLE_ID").unwrap_or_else(|_| "com.amuletai.app".into()),
            fcm_api_key: env::var("FCM_API_KEY").unwrap_or_default(),
        })
    }

    /// Bind address for the server.
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
