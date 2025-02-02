pub mod cli;
pub mod midi;
pub mod transport;
pub mod ui;

use midi::{DefaultMidiEngine, MidiEngine};
use std::sync::{Arc, Mutex}; // Import both trait and default implementation

// Re-export Args for convenience
pub use cli::Args;

#[derive(Debug)]
pub struct Transport {
    pub bpm: f32,
    pub tick_count: u32,
    pub beat: u32,
    pub bar: u32,
    pub is_playing: bool,
}

pub type SharedState = Arc<Mutex<Transport>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(Transport {
        bpm: 120.0,
        tick_count: 0,
        beat: 1,
        bar: 1,
        is_playing: false,
    }))
}

pub fn handle_device_list() -> Vec<String> {
    match DefaultMidiEngine::new(None) {
        Ok(engine) => engine.list_devices(),
        Err(_) => vec!["No MIDI devices found".to_string()],
    }
}
