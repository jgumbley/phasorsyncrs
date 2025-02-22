// main.rs

mod clock;
mod config;
mod event_loop;
mod external_clock;
mod logging;
mod state;
mod ui;

use log::{debug, info};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

fn initialize_clock(
    config: config::Config,
    shared_state: Arc<Mutex<state::SharedState>>,
    tick_tx: Sender<()>,
) {
    let clock_shared_state = Arc::clone(&shared_state);
    info!("Starting clock thread");
    let tick_tx_clone = tick_tx.clone();
    thread::spawn(move || {
        let clock: Box<dyn clock::ClockSource> = match config.clock_source {
            config::ClockSource::Internal => {
                info!("Initializing internal clock");
                Box::new(clock::InternalClock::new())
            }
            config::ClockSource::External => {
                info!("Initializing external clock");
                let device = config
                    .bind_to_device
                    .clone()
                    .expect("Device binding required for external sync");
                let external_clock = external_clock::ExternalClock::new(device);
                Box::new(external_clock)
            }
        };

        info!("Starting clock");
        clock.start(Box::new(move || {
            if let Ok(mut state) = clock_shared_state.lock() {
                state.tick_update();
                if config.clock_source == config::ClockSource::External {
                    tick_tx_clone.send(()).unwrap();
                }
            }
        }));
    });
}

fn start_event_loop(shared_state: Arc<Mutex<state::SharedState>>, tick_rx: Receiver<()>) {
    let event_loop_shared_state = Arc::clone(&shared_state);
    info!("Starting event loop thread");
    thread::spawn(move || {
        let event_loop = event_loop::EventLoop::new(event_loop_shared_state, tick_rx);
        event_loop.run();
    });
}

fn start_ui(shared_state: Arc<Mutex<state::SharedState>>) {
    let ui_shared_state = Arc::clone(&shared_state);
    info!("Starting UI thread");
    thread::spawn(move || {
        ui::UI::new(ui_shared_state).run();
    });
}

fn initialize_logging() {
    // Initialize logging
    if let Err(e) = logging::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
}

fn main() {
    initialize_logging();

    info!("Starting Phasorsyncrs");

    // Load configuration
    let config = config::Config::new();
    info!("Configuration loaded");
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

    // Create shared state
    let shared_state = Arc::new(Mutex::new(state::SharedState::new(config.bpm)));
    info!("Shared state initialized with BPM: {}", config.bpm);

    // Create tick channel
    let (tick_tx, tick_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

    // Start the clock thread
    initialize_clock(config, Arc::clone(&shared_state), tick_tx);

    // Start the event loop thread
    start_event_loop(Arc::clone(&shared_state), tick_rx);

    // Start the UI thread
    start_ui(Arc::clone(&shared_state));

    info!("All threads started, entering main loop");
    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
