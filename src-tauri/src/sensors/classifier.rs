use super::bridge::{AccelerometerData, GyroscopeData};
use std::collections::VecDeque;
use tracing::debug;

/// A rolling window of sensor readings for classification.
pub struct SensorWindow {
    accel_buffer: VecDeque<AccelerometerData>,
    gyro_buffer: VecDeque<GyroscopeData>,
    max_size: usize,
}

impl SensorWindow {
    /// Create a new SensorWindow with the given buffer size.
    pub fn new(max_size: usize) -> Self {
        Self {
            accel_buffer: VecDeque::with_capacity(max_size),
            gyro_buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a new accelerometer reading into the window.
    pub fn push_accel(&mut self, data: AccelerometerData) {
        if self.accel_buffer.len() >= self.max_size {
            self.accel_buffer.pop_front();
        }
        self.accel_buffer.push_back(data);
    }

    /// Push a new gyroscope reading into the window.
    pub fn push_gyro(&mut self, data: GyroscopeData) {
        if self.gyro_buffer.len() >= self.max_size {
            self.gyro_buffer.pop_front();
        }
        self.gyro_buffer.push_back(data);
    }

    /// Compute the maximum jerk (derivative of acceleration) in the window.
    ///
    /// Jerk is computed as the magnitude of the difference between consecutive
    /// accelerometer readings divided by the time delta.
    pub fn max_jerk(&self) -> f64 {
        if self.accel_buffer.len() < 2 {
            return 0.0;
        }

        let mut max_jerk = 0.0;
        for i in 1..self.accel_buffer.len() {
            let prev = &self.accel_buffer[i - 1];
            let curr = &self.accel_buffer[i];

            let dt = (curr.timestamp - prev.timestamp) as f64 / 1000.0;
            if dt <= 0.0 {
                continue;
            }

            let dx = curr.x - prev.x;
            let dy = curr.y - prev.y;
            let dz = curr.z - prev.z;

            let jerk = (dx * dx + dy * dy + dz * dz).sqrt() / dt;
            if jerk > max_jerk {
                max_jerk = jerk;
            }
        }

        debug!(max_jerk, "Computed max jerk");
        max_jerk
    }

    /// Detect a fall based on accelerometer data.
    ///
    /// A fall is detected when:
    /// 1. Free-fall (total acceleration < 0.5g) is observed
    /// 2. Followed by a hard impact (total acceleration > 3g)
    pub fn detect_fall(&self) -> bool {
        if self.accel_buffer.len() < 3 {
            return false;
        }

        let g = 9.81;
        let mut free_fall_seen = false;

        for reading in &self.accel_buffer {
            let total_accel =
                (reading.x * reading.x + reading.y * reading.y + reading.z * reading.z).sqrt();

            if total_accel < 0.5 * g {
                free_fall_seen = true;
            } else if free_fall_seen && total_accel > 3.0 * g {
                debug!("Fall detected: free-fall followed by impact");
                return true;
            }
        }

        false
    }

    /// Detect a crash based on accelerometer + gyroscope data.
    ///
    /// A crash is detected when:
    /// 1. High jerk (> 100 m/s³) is observed
    /// 2. Combined with high angular velocity (> 5 rad/s)
    pub fn detect_crash(&self) -> bool {
        if self.accel_buffer.is_empty() || self.gyro_buffer.is_empty() {
            return false;
        }

        let jerk = self.max_jerk();
        let mut max_angular_vel = 0.0;

        for gyro in &self.gyro_buffer {
            let angular_vel =
                (gyro.x * gyro.x + gyro.y * gyro.y + gyro.z * gyro.z).sqrt();
            if angular_vel > max_angular_vel {
                max_angular_vel = angular_vel;
            }
        }

        let crash = jerk > 100.0 && max_angular_vel > 5.0;
        if crash {
            debug!(jerk, max_angular_vel, "Crash detected");
        }
        crash
    }

    /// Clear all buffered data.
    pub fn clear(&mut self) {
        self.accel_buffer.clear();
        self.gyro_buffer.clear();
    }

    /// Returns the current number of buffered accelerometer readings.
    pub fn len(&self) -> usize {
        self.accel_buffer.len()
    }

    /// Returns true if the window is empty.
    pub fn is_empty(&self) -> bool {
        self.accel_buffer.is_empty()
    }
}
