use phasorsyncrs::ui::{
    create_bar_progress, create_beat_progress, create_transport_spinner, run_state_inspector,
};
use phasorsyncrs::{create_shared_state, midi::run_internal_clock};
use std::thread;
use std::time::Duration;

#[test]
fn test_beat_progress_creation() {
    let progress = create_beat_progress();
    assert_eq!(progress.length().unwrap(), 24); // TICKS_PER_BEAT
    assert_eq!(progress.position(), 0);
}

#[test]
fn test_bar_progress_creation() {
    let progress = create_bar_progress();
    assert_eq!(progress.length().unwrap(), 4); // BEATS_PER_BAR
    assert_eq!(progress.position(), 0);
}

#[test]
fn test_transport_spinner_creation() {
    let spinner = create_transport_spinner();
    assert!(spinner.length().is_none()); // Spinners don't have length
}

#[test]
fn test_state_inspector_thread() {
    let shared_state = create_shared_state();
    let handle = run_state_inspector(shared_state.clone());

    // Let it run briefly
    thread::sleep(Duration::from_millis(100));

    // Verify thread is running
    assert!(!handle.is_finished());

    // Modify state and verify thread continues
    {
        let mut state = shared_state.lock().unwrap();
        state.set_playing(true);
        state.set_tempo(140.0);
    }

    thread::sleep(Duration::from_millis(100));
    assert!(!handle.is_finished());
}

#[test]
fn test_state_inspector_with_playback() {
    let shared_state = create_shared_state();
    let handle = run_state_inspector(shared_state.clone());

    // Start playback and let it accumulate some ticks
    {
        let mut state = shared_state.lock().unwrap();
        state.set_playing(true);
        state.set_tempo(120.0);
    }

    // Let it run and update displays
    thread::sleep(Duration::from_millis(500));

    // Verify thread is still running
    assert!(!handle.is_finished());

    // Stop playback
    {
        let state = shared_state.lock().unwrap();
        state.set_playing(false);
    }

    thread::sleep(Duration::from_millis(100));
    assert!(!handle.is_finished());
}

#[test]
fn test_state_inspector_with_tempo_changes() {
    let shared_state = create_shared_state();
    let handle = run_state_inspector(shared_state.clone());

    // Test various tempo changes
    {
        let mut state = shared_state.lock().unwrap();
        state.set_playing(true);
        state.set_tempo(120.0);
    }

    thread::sleep(Duration::from_millis(100));

    {
        let mut state = shared_state.lock().unwrap();
        state.set_tempo(240.0);
    }

    thread::sleep(Duration::from_millis(100));

    assert!(!handle.is_finished());
}

#[test]
fn test_state_inspector_with_beat_bar_changes() {
    let shared_state = create_shared_state();

    // Start the internal clock in a separate thread
    let clock_state = shared_state.clone();
    let _clock_handle = thread::spawn(move || {
        run_internal_clock(clock_state);
    });

    // Start the state inspector
    let handle = run_state_inspector(shared_state.clone());

    // Give the clock time to initialize
    thread::sleep(Duration::from_millis(100));

    // Simulate beat and bar changes
    {
        let mut state = shared_state.lock().unwrap();
        state.set_playing(true);
        state.set_tempo(120.0);
    }

    // Let it run through several beats
    thread::sleep(Duration::from_millis(500));

    {
        let state = shared_state.lock().unwrap();
        assert!(state.get_tick_count() > 0);
        assert!(state.get_beat() >= 1);
    }

    assert!(!handle.is_finished());
}

#[test]
fn test_state_inspector_lock_failure_handling() {
    let shared_state = create_shared_state();
    let handle = run_state_inspector(shared_state.clone());

    // Create a temporary lock to simulate lock failure scenario
    let _lock = shared_state.lock().unwrap();

    // Let the inspector try to update while we hold the lock
    thread::sleep(Duration::from_millis(100));

    // Verify thread continues running despite lock contention
    assert!(!handle.is_finished());

    // Release lock and verify continued operation
    drop(_lock);
    thread::sleep(Duration::from_millis(100));
    assert!(!handle.is_finished());
}

#[test]
fn test_state_inspector_rapid_updates() {
    let shared_state = create_shared_state();
    let handle = run_state_inspector(shared_state.clone());

    // Rapidly toggle play state
    for _ in 0..10 {
        {
            let mut state = shared_state.lock().unwrap();
            state.set_playing(!state.is_playing());
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Verify thread handled rapid updates
    assert!(!handle.is_finished());
}
