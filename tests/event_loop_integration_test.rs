extern crate phasorsyncrs;

use phasorsyncrs::event_loop::{EngineMessage, EventLoop};
use phasorsyncrs::state::{SharedState, TransportState};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn integration_test_event_loop_two_ticks() {
    // Initialize the shared state with an initial BPM of 120.
    let shared_state = Arc::new(Mutex::new(SharedState::new(120)));
    // Set the transport state to 'Playing' so that tick updates are processed.
    {
        let mut state = shared_state.lock().unwrap();
        state.transport_state = TransportState::Playing;
    }

    // Set up the mpsc channel.
    let (engine_tx, engine_rx) = mpsc::channel();

    // Create the event loop instance.
    let event_loop = EventLoop::new(Arc::clone(&shared_state), engine_rx, None);

    // Spawn the event loop in a separate thread.
    let handle = thread::spawn(move || {
        let mut event_loop = event_loop;
        event_loop.run();
    });

    // Send two tick messages, with a small delay between them.
    engine_tx.send(EngineMessage::Tick).unwrap();
    thread::sleep(Duration::from_millis(100));
    engine_tx.send(EngineMessage::Tick).unwrap();

    // Close the channel so that the event loop will exit.
    drop(engine_tx);

    // Wait for the event loop thread to finish.
    handle.join().expect("Event loop thread panicked");

    // Verify that the shared state was updated.
    let state = shared_state.lock().unwrap();
    assert_eq!(
        state.get_tick_count(),
        2,
        "Tick count should be 2 after 2 ticks"
    );
    // Verify that BPM has been recalculated and is greater than 0.
    assert!(
        state.get_bpm() > 0,
        "BPM should be recalculated and greater than 0"
    );
}
