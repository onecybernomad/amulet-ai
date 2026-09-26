use serde::{Deserialize, Serialize};

/// Raw sensor data from a device.
#[derive(Debug, Clone, Deserialize)]
pub struct SensorData {
    pub accelerometer: Option<(f64, f64, f64)>,
    pub gyroscope: Option<(f64, f64, f64)>,
    pub speed: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Classified incident types from sensor data.
#[derive(Debug, Clone, Serialize)]
pub enum IncidentType {
    HardBraking,
    RapidAcceleration,
    SharpTurn,
    Collision,
    Rollover,
}

/// Process incoming sensor data and classify any incidents.
pub fn process_sensor_data(data: &SensorData) -> Option<IncidentType> {
    tracing::debug!("Processing sensor data at {}", data.timestamp);

    // Check accelerometer for collision/rollover
    if let Some((x, y, z)) = data.accelerometer {
        let magnitude = (x * x + y * y + z * z).sqrt();

        // Collision detection: very high acceleration magnitude
        if magnitude > 50.0 {
            tracing::warn!("Collision detected! Magnitude: {:.2}", magnitude);
            return Some(IncidentType::Collision);
        }

        // Rollover detection: sustained high lateral acceleration
        if x.abs() > 30.0 && y.abs() > 20.0 {
            tracing::warn!("Rollover detected! X: {:.2}, Y: {:.2}", x, y);
            return Some(IncidentType::Rollover);
        }

        // Hard braking: strong negative Z acceleration
        if z < -15.0 {
            tracing::info!("Hard braking detected: Z={:.2}", z);
            return Some(IncidentType::HardBraking);
        }

        // Rapid acceleration: strong positive Z acceleration
        if z > 15.0 {
            tracing::info!("Rapid acceleration detected: Z={:.2}", z);
            return Some(IncidentType::RapidAcceleration);
        }
    }

    // Check gyroscope for sharp turns
    if let Some((x, y, z)) = data.gyroscope {
        let rotation_magnitude = (x * x + y * y + z * z).sqrt();
        if rotation_magnitude > 25.0 {
            tracing::info!("Sharp turn detected: magnitude={:.2}", rotation_magnitude);
            return Some(IncidentType::SharpTurn);
        }
    }

    None
}

/// Batch process multiple sensor readings.
pub fn process_batch(readings: &[SensorData]) -> Vec<(chrono::DateTime<chrono::Utc>, IncidentType)> {
    let mut incidents = Vec::new();

    for reading in readings {
        if let Some(incident_type) = process_sensor_data(reading) {
            incidents.push((reading.timestamp, incident_type));
        }
    }

    incidents
}
