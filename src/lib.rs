use std::sync::{Arc, Mutex};

pub mod cli;
pub mod logging;
pub mod midi;
pub mod scheduler;
pub mod state;
pub mod transport;
pub mod ui;
use midi::DefaultMidiEngine;
use scheduler::ThreadScheduler;
use state::TransportState;

// Re-export Args for convenience
pub use cli::Args;
pub use scheduler::Scheduler;
pub use transport::run_timing_simulation;

pub type SharedState = Arc<Mutex<TransportState>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(TransportState::new()))
}

pub fn create_scheduler() -> ThreadScheduler {
    ThreadScheduler::new()
}

pub fn handle_device_list() -> Vec<String> {
    DefaultMidiEngine::list_devices()
}
