pub mod cli;
pub mod midi;
pub mod transport;
pub mod ui;

use midi::{DefaultMidiEngine, MidiEngine};
use std::sync::{Arc, Mutex};
use transport::Transport;

// Re-export Args for convenience
pub use cli::Args;

pub type SharedState = Arc<Mutex<Transport>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(Transport::new()))
}

pub fn handle_device_list() -> Vec<String> {
    match DefaultMidiEngine::new(None) {
        Ok(engine) => engine.list_devices(),
        Err(_) => vec!["No MIDI devices found".to_string()],
    }
}
