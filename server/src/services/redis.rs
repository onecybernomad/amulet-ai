use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

/// Shared Redis connection manager for token blacklist and session management.
#[derive(Clone)]
pub struct RedisClient {
    conn: MultiplexedConnection,
}

impl RedisClient {
    /// Create a new Redis client from a connection URL.
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    /// Add a token to the blacklist with a TTL (in seconds).
    pub async fn blacklist_token(&self, token: &str, ttl_secs: u64) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let key = format!("blacklist:{}", token);
        conn.set_ex::<_, _, ()>(&key, "1", ttl_secs).await?;
        Ok(())
    }

    /// Check if a token is blacklisted.
    pub async fn is_token_blacklisted(&self, token: &str) -> anyhow::Result<bool> {
        let mut conn = self.conn.clone();
        let key = format!("blacklist:{}", token);
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }

    /// Store a refresh token family for rotation tracking.
    pub async fn store_token_family(
        &self,
        family_id: &str,
        user_id: &str,
        token_hash: &str,
        ttl_secs: u64,
    ) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let key = format!("family:{}", family_id);
        let value = format!("{}:{}", user_id, token_hash);
        conn.set_ex::<_, _, ()>(&key, value, ttl_secs).await?;
        Ok(())
    }

    /// Get the token family info if it exists.
    pub async fn get_token_family(&self, family_id: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.conn.clone();
        let key = format!("family:{}", family_id);
        let value: Option<String> = conn.get(&key).await?;
        Ok(value)
    }

    /// Remove a token family (e.g., after logout or reuse detection).
    pub async fn remove_token_family(&self, family_id: &str) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let key = format!("family:{}", family_id);
        conn.del::<_, ()>(&key).await?;
        Ok(())
    }

    /// Store the current refresh token for a user (for reuse detection).
    pub async fn store_refresh_token(
        &self,
        user_id: &str,
        token_hash: &str,
        ttl_secs: u64,
    ) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let key = format!("refresh:{}", user_id);
        conn.set_ex::<_, _, ()>(&key, token_hash, ttl_secs).await?;
        Ok(())
    }

    /// Get the stored refresh token hash for a user.
    pub async fn get_refresh_token(&self, user_id: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.conn.clone();
        let key = format!("refresh:{}", user_id);
        let value: Option<String> = conn.get(&key).await?;
        Ok(value)
    }

    /// Remove the stored refresh token for a user.
    pub async fn remove_refresh_token(&self, user_id: &str) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let key = format!("refresh:{}", user_id);
        conn.del::<_, ()>(&key).await?;
        Ok(())
    }
}
