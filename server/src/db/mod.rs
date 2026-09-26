use sqlx::PgPool;

pub mod circles;
pub mod chat;
pub mod driving;
pub mod geofence;
pub mod incidents;
pub mod locations;
pub mod medications;
pub mod subscriptions;
pub mod users;

/// Database wrapper holding the connection pool.
#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
