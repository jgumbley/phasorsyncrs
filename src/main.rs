// main.rs

mod clock;
mod config;
mod event_loop;
mod state;
mod ui;
mod external_clock;
mod logging;

use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, debug};

fn main() {
    // Initialize logging
    if let Err(e) = logging::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

    info!("Starting Phasorsyncrs");

    // Load configuration
    let config = config::Config::new();
    info!("Configuration loaded");
    debug!("Clock source: {:?}", match config.clock_source {
        config::ClockSource::Internal => "Internal",
        config::ClockSource::External => "External",
    });
    if let Some(device) = &config.bind_to_device {
        debug!("Bound to MIDI device: {}", device);
    }

    // Create shared state
    let shared_state = Arc::new(Mutex::new(state::SharedState::new(config.bpm)));
    info!("Shared state initialized with BPM: {}", config.bpm);

    // Start the clock thread
    let clock_shared_state = Arc::clone(&shared_state);
    info!("Starting clock thread");
    thread::spawn(move || {
        let clock: Box<dyn clock::ClockSource> = match config.clock_source {
            config::ClockSource::Internal => {
                info!("Initializing internal clock");
                Box::new(clock::InternalClock::new())
            },
            config::ClockSource::External => {
                info!("Initializing external clock");
                let device = config.bind_to_device.expect("Device binding required for external sync");
                Box::new(external_clock::ExternalClock::new(device))
            }
        };

        info!("Starting clock");
        clock.start(Box::new(move || {
            if let Ok(mut state) = clock_shared_state.lock() {
                state.tick_update();
            }
        }));
    });

    // Start the event loop thread
    let event_loop_shared_state = Arc::clone(&shared_state);
    info!("Starting event loop thread");
    thread::spawn(move || {
        event_loop::EventLoop::new(event_loop_shared_state).run();
    });

    // Start the UI thread
    let ui_shared_state = Arc::clone(&shared_state);
    info!("Starting UI thread");
    thread::spawn(move || {
        ui::UI::new(ui_shared_state).run();
    });

    info!("All threads started, entering main loop");
    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
