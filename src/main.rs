// main.rs

mod clock;
mod config;
mod event_loop;
mod state;
mod ui;

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = config::Config::new();

    // Create shared state
    let shared_state = Arc::new(Mutex::new(state::SharedState::new(config.bpm)));

    // Start the clock thread
    let clock_shared_state = Arc::clone(&shared_state);
    thread::spawn(move || {
        clock::InternalClock::new().start(Box::new(move || {
            if let Ok(mut state) = clock_shared_state.lock() {
                state.tick_update();
            }
        }));
    });

    // Start the event loop thread
    let event_loop_shared_state = Arc::clone(&shared_state);
    thread::spawn(move || {
        event_loop::EventLoop::new(event_loop_shared_state).run();
    });

    // Start the UI thread
    let ui_shared_state = Arc::clone(&shared_state);
    thread::spawn(move || {
        ui::UI::new(ui_shared_state).run();
    });

    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
