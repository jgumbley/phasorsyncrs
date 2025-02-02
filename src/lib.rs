pub mod cli;
pub mod midi;
pub mod scheduler;
pub mod transport;
pub mod ui;

use midi::{DefaultMidiEngine, MidiEngine};
use scheduler::ThreadScheduler;
use std::sync::{Arc, Mutex};
use transport::Transport;

// Re-export Args for convenience
pub use cli::Args;
pub use scheduler::Scheduler;

pub type SharedState = Arc<Mutex<Transport>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(Transport::new()))
}

pub fn create_scheduler() -> ThreadScheduler {
    ThreadScheduler::new()
}

pub fn handle_device_list() -> Vec<String> {
    match DefaultMidiEngine::new(None) {
        Ok(engine) => engine.list_devices(),
        Err(_) => vec!["No MIDI devices found".to_string()],
    }
}
