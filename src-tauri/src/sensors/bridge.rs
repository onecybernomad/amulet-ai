use serde::{Deserialize, Serialize};
use tracing::debug;

/// Raw accelerometer reading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AccelerometerData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp: i64, // Unix millis
}

/// Raw gyroscope reading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GyroscopeData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp: i64,
}

/// Raw magnetometer reading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MagnetometerData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp: i64,
}

/// Bridge trait for accessing device sensors.
///
/// On desktop this is a no-op stub; on mobile, the Tauri command
/// layer feeds data into the classifier.
pub trait SensorBridge {
    /// Start listening to sensor data.
    fn start(&mut self) -> Result<(), String>;

    /// Stop listening to sensor data.
    fn stop(&mut self);

    /// Returns true if the sensor bridge is currently running.
    fn is_running(&self) -> bool;
}

/// Default stub implementation for platforms without native sensors.
pub struct StubSensorBridge {
    running: bool,
}

impl StubSensorBridge {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for StubSensorBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorBridge for StubSensorBridge {
    fn start(&mut self) -> Result<(), String> {
        debug!("Stub sensor bridge started (no-op on desktop)");
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) {
        debug!("Stub sensor bridge stopped");
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}
