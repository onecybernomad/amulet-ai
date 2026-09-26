use super::classifier::SensorWindow;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// States for the fall detection state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallState {
    Idle,
    FallCandidate,
    FallDetected,
    Escalated,
}

/// States for the crash detection state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashState {
    Idle,
    CrashCandidate,
    CrashDetected,
}

/// Fall detection state machine.
///
/// Transitions:
///   Idle → FallCandidate (when free-fall detected)
///   FallCandidate → FallDetected (when impact follows within 500ms)
///   FallCandidate → Idel (timeout without impact)
///   FallDetected → Escalated (after 30s without user response)
pub struct FallDetector {
    state: FallState,
    candidate_since: Option<Instant>,
    detected_since: Option<Instant>,
    window: SensorWindow,
}

impl FallDetector {
    pub fn new() -> Self {
        Self {
            state: FallState::Idle,
            candidate_since: None,
            detected_since: None,
            window: SensorWindow::new(100),
        }
    }

    /// Process a new sensor window and update the state machine.
    pub fn process(&mut self, window: &SensorWindow) -> FallState {
        match self.state {
            FallState::Idle => {
                if window.detect_fall() {
                    self.state = FallState::FallCandidate;
                    self.candidate_since = Some(Instant::now());
                    info!("Fall candidate detected");
                }
            }
            FallState::FallCandidate => {
                if let Some(since) = self.candidate_since {
                    if since.elapsed() > Duration::from_millis(500) {
                        self.state = FallState::Idle;
                        self.candidate_since = None;
                    } else if window.detect_fall() {
                        self.state = FallState::FallDetected;
                        self.detected_since = Some(Instant::now());
                        warn!("Fall detected!");
                    }
                }
            }
            FallState::FallDetected => {
                if let Some(since) = self.detected_since {
                    if since.elapsed() > Duration::from_secs(30) {
                        self.state = FallState::Escalated;
                        warn!("Fall escalated — no user response for 30s");
                    }
                }
            }
            FallState::Escalated => {
                // Remains escalated until manually reset
            }
        }

        // Merge window data
        for i in 0..window.len() {
            // In production: feed individual readings into self.window
        }

        self.state
    }

    /// Reset the state machine to idle (e.g., after user acknowledges).
    pub fn reset(&mut self) {
        self.state = FallState::Idle;
        self.candidate_since = None;
        self.detected_since = None;
        self.window.clear();
    }

    pub fn state(&self) -> FallState {
        self.state
    }
}

impl Default for FallDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Crash detection state machine.
///
/// Transitions:
///   Idle → CrashCandidate (when high jerk detected)
///   CrashCandidate → CrashDetected (when angular velocity confirms within 200ms)
///   CrashCandidate → Idel (timeout without confirmation)
pub struct CrashDetector {
    state: CrashState,
    candidate_since: Option<Instant>,
    window: SensorWindow,
}

impl CrashDetector {
    pub fn new() -> Self {
        Self {
            state: CrashState::Idle,
            candidate_since: None,
            window: SensorWindow::new(50),
        }
    }

    /// Process a new sensor window and update the state machine.
    pub fn process(&mut self, window: &SensorWindow) -> CrashState {
        match self.state {
            CrashState::Idle => {
                if window.detect_crash() {
                    self.state = CrashState::CrashCandidate;
                    self.candidate_since = Some(Instant::now());
                    info!("Crash candidate detected");
                }
            }
            CrashState::CrashCandidate => {
                if let Some(since) = self.candidate_since {
                    if since.elapsed() > Duration::from_millis(200) {
                        self.state = CrashState::Idle;
                        self.candidate_since = None;
                    } else if window.detect_crash() {
                        self.state = CrashState::CrashDetected;
                        warn!("Crash detected!");
                    }
                }
            }
            CrashState::CrashDetected => {
                // Remains detected until manually reset
            }
        }

        self.state
    }

    /// Reset the state machine to idle.
    pub fn reset(&mut self) {
        self.state = CrashState::Idle;
        self.candidate_since = None;
        self.window.clear();
    }

    pub fn state(&self) -> CrashState {
        self.state
    }
}

impl Default for CrashDetector {
    fn default() -> Self {
        Self::new()
    }
}
