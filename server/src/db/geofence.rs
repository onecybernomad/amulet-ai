use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::Place;

/// Create a new geofenced place.
pub async fn create_place(
    pool: &PgPool,
    circle_id: Uuid,
    name: &str,
    latitude: f64,
    longitude: f64,
    radius_meters: f64,
) -> Result<Place> {
    let place = sqlx::query_as::<_, Place>(
        r#"
        INSERT INTO places (circle_id, name, latitude, longitude, radius_meters)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, circle_id, name, latitude, longitude, radius_meters, created_at
        "#,
    )
    .bind(circle_id)
    .bind(name)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_meters)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created place: id={}, name={}", place.id, place.name);
    Ok(place)
}

/// Get all places for a circle.
pub async fn get_places(pool: &PgPool, circle_id: Uuid) -> Result<Vec<Place>> {
    let places = sqlx::query_as::<_, Place>(
        r#"
        SELECT id, circle_id, name, latitude, longitude, radius_meters, created_at
        FROM places
        WHERE circle_id = $1
        ORDER BY name
        "#,
    )
    .bind(circle_id)
    .fetch_all(pool)
    .await?;

    Ok(places)
}

/// Update a place's properties.
pub async fn update_place(
    pool: &PgPool,
    place_id: Uuid,
    name: &str,
    latitude: f64,
    longitude: f64,
    radius_meters: f64,
) -> Result<Place> {
    let place = sqlx::query_as::<_, Place>(
        r#"
        UPDATE places
        SET name = $2, latitude = $3, longitude = $4, radius_meters = $5
        WHERE id = $1
        RETURNING id, circle_id, name, latitude, longitude, radius_meters, created_at
        "#,
    )
    .bind(place_id)
    .bind(name)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_meters)
    .fetch_one(pool)
    .await?;

    tracing::info!("Updated place: id={}", place_id);
    Ok(place)
}

/// Delete a place.
pub async fn delete_place(pool: &PgPool, place_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM places
        WHERE id = $1
        "#,
    )
    .bind(place_id)
    .execute(pool)
    .await?;

    tracing::info!("Deleted place: id={}", place_id);
    Ok(())
}

/// Check geofence events for a user's location against their circle's places.
/// Uses PostGIS ST_DWithin for spatial queries.
pub async fn check_geofence_events(
    pool: &PgPool,
    circle_id: Uuid,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<(String, bool)>> {
    let events = sqlx::query_as::<_, (String, bool)>(
        r#"
        SELECT
            p.name,
            ST_DWithin(
                ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography,
                ST_SetSRID(ST_MakePoint(p.longitude, p.latitude), 4326)::geography,
                p.radius_meters
            ) as entered
        FROM places p
        WHERE p.circle_id = $1
        "#,
    )
    .bind(circle_id)
    .bind(latitude)
    .bind(longitude)
    .fetch_all(pool)
    .await?;

    tracing::debug!("Checked geofence events for user={} at ({}, {})", user_id, latitude, longitude);
    Ok(events)
}
