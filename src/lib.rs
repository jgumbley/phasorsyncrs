use clap::Parser;
use std::sync::{Arc, Mutex};

pub mod midi;

#[derive(Debug, Clone)]
pub struct TransportState {
    pub bpm: f64,
    pub tick_count: u64,
    pub beat: u32,
    pub bar: u32,
    pub is_playing: bool,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            tick_count: 0,
            beat: 1,
            bar: 1,
            is_playing: false,
        }
    }
}

pub type SharedState = Arc<Mutex<TransportState>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(TransportState::default()))
}

/// Simple CLI demonstration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// List available MIDI devices
    #[arg(long)]
    pub device_list: bool,
}

/// Handle listing of MIDI devices
pub fn handle_device_list() -> Vec<String> {
    midi::list_devices()
}
