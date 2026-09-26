pub mod bridge;
pub mod classifier;
pub mod detector;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global sensor state.
#[derive(Default)]
pub struct SensorState {
    pub running: Arc<AtomicBool>,
}
