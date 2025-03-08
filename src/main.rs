use log::{debug, error, info};
use phasorsyncrs::{clock, config, event_loop, external_clock, logging, midi_output, state, tui};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::event_loop::EngineMessage;

fn initialize_clock(config: config::Config, engine_tx: Sender<EngineMessage>) {
    info!("Starting clock thread");

    // Create a new thread for the clock to run independently
    thread::spawn(move || {
        // Create the appropriate clock source based on configuration
        let clock_source: Box<dyn clock::ClockSource> = create_clock_source(&config, engine_tx);

        // Start the clock
        info!("Starting clock");
        clock_source.start();
    });
}

/// Creates the appropriate clock source based on configuration
fn create_clock_source(
    config: &config::Config,
    engine_tx: Sender<EngineMessage>,
) -> Box<dyn clock::ClockSource> {
    match config.clock_source {
        config::ClockSource::Internal => {
            info!("Initializing internal clock");
            Box::new(clock::InternalClock::new(engine_tx))
        }
        config::ClockSource::External => {
            info!("Initializing external clock");
            // Get the device name, panic with helpful message if not provided
            let device_name = config
                .bind_to_device
                .clone()
                .expect("Device binding required for external sync");

            Box::new(external_clock::ExternalClock::new(device_name, engine_tx))
        }
    }
}

fn start_ui(shared_state: Arc<Mutex<state::SharedState>>, engine_tx: Sender<EngineMessage>) {
    info!("Starting TUI");
    if let Err(e) = tui::run_tui_event_loop(shared_state, engine_tx) {
        eprintln!("TUI failed: {}", e);
        std::process::exit(1);
    }
}

fn initialize_logging() {
    // Initialize logging
    if let Err(e) = logging::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
}

// Log configuration details
fn log_config_details(config: &config::Config) {
    debug!(
        "Clock source: {:?}",
        match config.clock_source {
            config::ClockSource::Internal => "Internal",
            config::ClockSource::External => "External",
        }
    );
    if let Some(device) = &config.bind_to_device {
        debug!("Bound to MIDI device: {}", device);
    }
}

// Initialize application components
fn initialize_components(
    config: config::Config,
) -> (Arc<Mutex<state::SharedState>>, Sender<EngineMessage>) {
    // Create shared state
    let shared_state = Arc::new(Mutex::new(state::SharedState::new(config.bpm)));
    info!("Shared state initialized with BPM: {}", config.bpm);

    // Create engine message channel
    let (engine_tx, engine_rx): (Sender<EngineMessage>, Receiver<EngineMessage>) = mpsc::channel();

    // Set up MIDI output - always initialize for musical graph
    info!("Setting up MIDI output for event loop");
    let mut output_manager = midi_output::MidiOutputManager::new();

    let result = if let Some(device) = &config.midi_output_device {
        output_manager.connect_to_device(device)
    } else {
        output_manager.connect_to_first_available()
    };

    let midi_output = if let Err(e) = result {
        error!("Failed to connect MIDI output: {}", e);
        None
    } else {
        info!("MIDI output connected successfully");
        Some(output_manager)
    };

    let midi_output = midi_output;

    // Start the clock thread
    initialize_clock(config, engine_tx.clone());

    // Start the event loop thread with MIDI output
    let event_loop_shared_state = Arc::clone(&shared_state);
    info!("Starting event loop thread");
    thread::spawn(move || {
        let mut event_loop =
            event_loop::EventLoop::new(event_loop_shared_state, engine_rx, midi_output);
        event_loop.run();
    });

    (shared_state, engine_tx)
}

fn main() {
    initialize_logging();
    info!("Starting Phasorsyncrs");

    // Load configuration
    let config = config::Config::new();
    info!("Configuration loaded");

    // Log configuration details

    // Log configuration details
    log_config_details(&config);

    // Setup MIDI output
    info!("MIDI output setup complete");

    // Initialize components
    let (shared_state, engine_tx) = initialize_components(config);

    // Start the UI thread
    start_ui(Arc::clone(&shared_state), engine_tx.clone());

    info!("All threads started, entering main loop");
    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
