use log::{debug, error, info};
use phasorsyncrs::{clock, config, event_loop, external_clock, logging, midi_output, state, tui};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::event_loop::EngineMessage;
use phasorsyncrs::midi_output::MidiMessage;

fn initialize_clock(
    config: config::Config,
    shared_state: Arc<Mutex<state::SharedState>>,
    tick_tx: Sender<EngineMessage>,
) {
    let clock_shared_state = Arc::clone(&shared_state);
    info!("Starting clock thread");
    let tick_tx_clone = tick_tx.clone();
    thread::spawn(move || {
        let clock: Box<dyn clock::ClockSource> = match config.clock_source {
            config::ClockSource::Internal => {
                info!("Initializing internal clock");
                Box::new(clock::InternalClock::new(tick_tx_clone.clone()))
            }
            config::ClockSource::External => {
                info!("Initializing external clock");
                let device = config
                    .bind_to_device
                    .clone()
                    .expect("Device binding required for external sync");
                let external_clock = external_clock::ExternalClock::new(device, tick_tx_clone);
                Box::new(external_clock)
            }
        };

        info!("Starting clock");
        clock.start(Box::new(move || {
            if let Ok(mut state) = clock_shared_state.lock() {
                state.tick_update();
            }
        }));
    });
}

fn start_ui(shared_state: Arc<Mutex<state::SharedState>>, tick_tx: Sender<EngineMessage>) {
    info!("Starting TUI");
    if let Err(e) = tui::run_tui_event_loop(shared_state, tick_tx) {
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

fn setup_midi_output(config: &config::Config) -> Option<std::sync::mpsc::Sender<MidiMessage>> {
    // Always set up MIDI output for musical graph
    info!("Setting up MIDI output");
    let (midi_tx, midi_rx) = mpsc::channel();
    midi_output::run_midi_output_thread(midi_rx, config.midi_output_device.clone());
    Some(midi_tx)
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

    // Create tick channel
    let (tick_tx, tick_rx): (Sender<EngineMessage>, Receiver<EngineMessage>) = mpsc::channel();

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

    // Start the clock thread
    initialize_clock(config, Arc::clone(&shared_state), tick_tx.clone());

    // Start the event loop thread with MIDI output
    let event_loop_shared_state = Arc::clone(&shared_state);
    info!("Starting event loop thread");
    thread::spawn(move || {
        let mut event_loop =
            event_loop::EventLoop::new(event_loop_shared_state, tick_rx, midi_output);
        event_loop.run();
    });

    (shared_state, tick_tx)
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
    let _midi_tx = setup_midi_output(&config);
    info!("MIDI output setup complete");

    // Initialize components
    let (shared_state, tick_tx) = initialize_components(config);

    // Start the UI thread
    start_ui(Arc::clone(&shared_state), tick_tx.clone());

    info!("All threads started, entering main loop");
    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
