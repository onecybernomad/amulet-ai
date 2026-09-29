use anyhow::Result;
use sqlx::SqlitePool;
use sqlx::Row;
use uuid::Uuid;
use crate::types::Place;

/// Create a new geofenced place.
pub async fn create_place(
    pool: &SqlitePool,
    circle_id: Uuid,
    name: &str,
    latitude: f64,
    longitude: f64,
    radius_meters: f64,
) -> Result<Place> {
    let row = sqlx::query(
        r#"
        INSERT INTO places (circle_id, name, latitude, longitude, radius_meters)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, circle_id, name, latitude, longitude, radius_meters, created_at
        "#,
    )
    .bind(circle_id.to_string())
    .bind(name)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_meters)
    .fetch_one(pool)
    .await?;

    let place = Place {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        name: row.try_get("name")?,
        latitude: row.try_get("latitude")?,
        longitude: row.try_get("longitude")?,
        radius_meters: row.try_get("radius_meters")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Created place: id={}, name={}", place.id, place.name);
    Ok(place)
}

/// Get all places for a circle.
pub async fn get_places(pool: &SqlitePool, circle_id: Uuid) -> Result<Vec<Place>> {
    let rows = sqlx::query(
        r#"
        SELECT id, circle_id, name, latitude, longitude, radius_meters, created_at
        FROM places
        WHERE circle_id = ?
        ORDER BY name
        "#,
    )
    .bind(circle_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut places = Vec::new();
    for row in rows {
        places.push(Place {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
            name: row.try_get("name")?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            radius_meters: row.try_get("radius_meters")?,
            created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
        });
    }

    Ok(places)
}

/// Update a place's properties.
pub async fn update_place(
    pool: &SqlitePool,
    place_id: Uuid,
    name: &str,
    latitude: f64,
    longitude: f64,
    radius_meters: f64,
) -> Result<Place> {
    let row = sqlx::query(
        r#"
        UPDATE places
        SET name = ?, latitude = ?, longitude = ?, radius_meters = ?
        WHERE id = ?
        RETURNING id, circle_id, name, latitude, longitude, radius_meters, created_at
        "#,
    )
    .bind(name)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_meters)
    .bind(place_id.to_string())
    .fetch_one(pool)
    .await?;

    let place = Place {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        circle_id: Uuid::parse_str(&row.try_get::<String, _>("circle_id")?).map_err(|e| anyhow::anyhow!("{}", e))?,
        name: row.try_get("name")?,
        latitude: row.try_get("latitude")?,
        longitude: row.try_get("longitude")?,
        radius_meters: row.try_get("radius_meters")?,
        created_at: crate::db::parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    };

    tracing::info!("Updated place: id={}", place_id);
    Ok(place)
}

/// Delete a place.
pub async fn delete_place(pool: &SqlitePool, place_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM places
        WHERE id = ?
        "#,
    )
    .bind(place_id.to_string())
    .execute(pool)
    .await?;

    tracing::info!("Deleted place: id={}", place_id);
    Ok(())
}

/// Check geofence events for a user's location against their circle's places.
/// Uses the Haversine formula to calculate great-circle distance.
pub async fn check_geofence_events(
    pool: &SqlitePool,
    circle_id: Uuid,
    user_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<(String, bool)>> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.name,
            p.latitude AS place_lat,
            p.longitude AS place_lng,
            p.radius_meters,
            (6371000.0 * acos(
                cos(radians(?)) * cos(radians(p.latitude)) *
                cos(radians(p.longitude) - radians(?)) +
                sin(radians(?)) * sin(radians(p.latitude))
            )) AS distance
        FROM places p
        WHERE p.circle_id = ?
        "#,
    )
    .bind(latitude)
    .bind(longitude)
    .bind(latitude)
    .bind(circle_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let name: String = row.try_get("name")?;
        let distance: f64 = row.try_get("distance")?;
        let radius_meters: f64 = row.try_get("radius_meters")?;
        let entered = distance <= radius_meters;
        events.push((name, entered));
    }

    tracing::debug!("Checked geofence events for user={} at ({}, {})", user_id, latitude, longitude);
    Ok(events)
}
