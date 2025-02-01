use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
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
    /// Optional name to greet
    #[arg(short, long)]
    pub name: Option<String>,

    /// List available MIDI devices
    #[arg(long)]
    pub device_list: bool,
}

/// Get the user's name either from command line args or through interactive prompt
pub fn get_user_name(args_name: Option<String>) -> String {
    match args_name {
        Some(name) => name,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("What's your name?")
            .interact_text()
            .unwrap(),
    }
}

/// Handle listing of MIDI devices
pub fn handle_device_list() -> Vec<String> {
    midi::list_devices()
}
