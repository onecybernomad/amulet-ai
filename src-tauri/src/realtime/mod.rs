pub mod handlers;
pub mod ws_client;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global realtime connection state.
#[derive(Default)]
pub struct RealtimeState {
    pub connected: Arc<AtomicBool>,
}
