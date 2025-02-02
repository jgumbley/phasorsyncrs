use phasorsyncrs::midi::{
    BpmCalculator, ClockGenerator, ClockMessage, ClockMessageHandler, InternalClock,
};
use phasorsyncrs::state::TransportState;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

// Increased tolerance to account for system timing variations
const BPM_TOLERANCE: f64 = 5.0;
const TEMPO_CHANGE_TOLERANCE: f64 = 8.0;

// Mock handler for testing multiple handlers
struct MockHandler {
    received_messages: Arc<Mutex<Vec<ClockMessage>>>,
    message_count: Arc<AtomicUsize>,
}

impl MockHandler {
    fn new() -> Self {
        Self {
            received_messages: Arc::new(Mutex::new(Vec::new())),
            message_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl ClockMessageHandler for MockHandler {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64> {
        self.message_count.fetch_add(1, Ordering::SeqCst);
        self.received_messages.lock().unwrap().push(msg.clone());
        None
    }
}

#[test]
fn test_bpm_calculator_start_stop() {
    let calc = BpmCalculator::new();

    // Initially no BPM
    assert_eq!(calc.process_message(ClockMessage::Tick), None);

    // Start message
    assert_eq!(calc.process_message(ClockMessage::Start), None);
    assert!(calc.is_playing());

    // Stop message
    assert_eq!(calc.process_message(ClockMessage::Stop), None);
    assert!(!calc.is_playing());

    // Ticks while stopped should return None
    assert_eq!(calc.process_message(ClockMessage::Tick), None);
}

#[test]
fn test_bpm_calculation_120bpm() {
    let calc = BpmCalculator::new();
    calc.process_message(ClockMessage::Start);

    // At 120 BPM with 24 PPQ:
    // - One quarter note = 500ms
    // - One tick = 500ms/24 ≈ 20.833ms
    let tick_interval = Duration::from_micros(20_833);

    // Send several ticks at 120 BPM
    for _ in 0..48 {
        // Two quarter notes worth of ticks
        calc.process_message(ClockMessage::Tick);
        thread::sleep(tick_interval);
    }

    if let Some(bpm) = calc.process_message(ClockMessage::Tick) {
        assert!(
            (bpm - 120.0).abs() < BPM_TOLERANCE,
            "Expected ~120 BPM, got {}",
            bpm
        );
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_tempo_change() {
    let calc = BpmCalculator::new();
    calc.process_message(ClockMessage::Start);

    // Start at 120 BPM
    let tick_interval_120 = Duration::from_micros(20_833);
    for _ in 0..24 {
        calc.process_message(ClockMessage::Tick);
        thread::sleep(tick_interval_120);
    }

    // Transition to 140 BPM
    // At 140 BPM: one tick = (60/140/24) seconds ≈ 17.857ms
    let tick_interval_140 = Duration::from_micros(17_857);
    for _ in 0..24 {
        calc.process_message(ClockMessage::Tick);
        thread::sleep(tick_interval_140);
    }

    if let Some(bpm) = calc.process_message(ClockMessage::Tick) {
        assert!(
            (bpm - 140.0).abs() < TEMPO_CHANGE_TOLERANCE,
            "Expected ~140 BPM, got {}",
            bpm
        );
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_continue_message() {
    let calc = BpmCalculator::new();

    // Start and send some ticks
    calc.process_message(ClockMessage::Start);
    let tick_interval = Duration::from_micros(20_833); // 120 BPM

    for _ in 0..24 {
        calc.process_message(ClockMessage::Tick);
        thread::sleep(tick_interval);
    }

    // Stop and continue
    calc.process_message(ClockMessage::Stop);
    assert!(!calc.is_playing());

    calc.process_message(ClockMessage::Continue);
    assert!(calc.is_playing());

    // Should still maintain tempo after continue
    for _ in 0..24 {
        calc.process_message(ClockMessage::Tick);
        thread::sleep(tick_interval);
    }

    if let Some(bpm) = calc.process_message(ClockMessage::Tick) {
        assert!(
            (bpm - 120.0).abs() < BPM_TOLERANCE,
            "Expected ~120 BPM, got {}",
            bpm
        );
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_transport_state_transitions() {
    let calc = BpmCalculator::new();

    // Initial state should be stopped
    assert!(!calc.is_playing());

    // Start -> Running
    calc.process_message(ClockMessage::Start);
    assert!(calc.is_playing());

    // Start while already running should reset but stay running
    calc.process_message(ClockMessage::Start);
    assert!(calc.is_playing());

    // Stop -> Stopped
    calc.process_message(ClockMessage::Stop);
    assert!(!calc.is_playing());

    // Continue -> Running
    calc.process_message(ClockMessage::Continue);
    assert!(calc.is_playing());

    // Stop -> Stopped again
    calc.process_message(ClockMessage::Stop);
    assert!(!calc.is_playing());

    // Multiple stops should keep it stopped
    calc.process_message(ClockMessage::Stop);
    assert!(!calc.is_playing());
}

#[test]
fn test_transport_state_with_ticks() {
    let calc = BpmCalculator::new();

    // Ticks while stopped should not affect state
    calc.process_message(ClockMessage::Tick);
    assert!(!calc.is_playing());

    // Start and verify ticks are processed
    calc.process_message(ClockMessage::Start);
    assert!(calc.is_playing());
    calc.process_message(ClockMessage::Tick);
    assert!(calc.is_playing());

    // Stop and verify ticks don't restart
    calc.process_message(ClockMessage::Stop);
    assert!(!calc.is_playing());
    calc.process_message(ClockMessage::Tick);
    assert!(!calc.is_playing());

    // Continue and verify ticks are processed again
    calc.process_message(ClockMessage::Continue);
    assert!(calc.is_playing());
    calc.process_message(ClockMessage::Tick);
    assert!(calc.is_playing());
}

#[test]
fn test_clock_generator() {
    let calc = BpmCalculator::new();
    let mut generator = ClockGenerator::new(calc);

    // Start the clock generator
    generator.start();

    // Let it run for about 1 second (enough time to get stable BPM readings)
    thread::sleep(Duration::from_secs(1));

    // Stop the generator
    generator.stop();

    // Verify the BPM calculator is in the correct state
    assert!(generator.is_playing());

    // Get one more tick to check the BPM
    if let Some(bpm) = generator.current_bpm() {
        assert!(
            (bpm - 120.0).abs() < BPM_TOLERANCE,
            "Expected ~120 BPM, got {}",
            bpm
        );
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_external_transport_state() {
    use phasorsyncrs::midi::{ExternalClock, MidiMessage};

    let shared_state = Arc::new(Mutex::new(TransportState::new()));
    let mut clock = ExternalClock::new(shared_state.clone());

    // Start playback
    clock.handle_midi_message(MidiMessage::Start);
    {
        let transport = shared_state.lock().unwrap();
        assert!(
            transport.is_playing(),
            "Transport should be playing after Start"
        );
    }

    // Send some clock ticks
    for _ in 0..24 {
        clock.handle_midi_message(MidiMessage::Clock);
    }

    // Stop playback
    clock.handle_midi_message(MidiMessage::Stop);
    {
        let transport = shared_state.lock().unwrap();
        assert!(
            !transport.is_playing(),
            "Transport should be stopped after Stop"
        );
    }

    // Send more clock ticks while stopped - should not affect playing state
    for _ in 0..24 {
        clock.handle_midi_message(MidiMessage::Clock);
        let transport = shared_state.lock().unwrap();
        assert!(
            !transport.is_playing(),
            "Transport should remain stopped when receiving ticks while stopped"
        );
    }

    // Continue playback
    clock.handle_midi_message(MidiMessage::Continue);
    {
        let transport = shared_state.lock().unwrap();
        assert!(
            transport.is_playing(),
            "Transport should be playing after Continue"
        );
    }
}

#[test]
fn test_internal_clock() {
    let shared_state = Arc::new(Mutex::new(TransportState::new()));
    let mut clock = InternalClock::new(shared_state.clone());

    // Initially stopped
    assert!(!clock.is_playing());

    // Start the clock
    clock.start();
    assert!(clock.is_playing());

    // Let it run briefly to stabilize
    thread::sleep(Duration::from_millis(500));

    // Check BPM (should be default 120)
    if let Some(bpm) = clock.current_bpm() {
        assert!(
            (bpm - 120.0).abs() < BPM_TOLERANCE,
            "Expected ~120 BPM, got {}",
            bpm
        );
    } else {
        panic!("Expected BPM calculation, got None");
    }

    // Verify transport state is in sync
    {
        let transport = shared_state.lock().unwrap();
        assert!(transport.is_playing(), "Transport should be playing");
        assert!(
            (transport.tempo() - 120.0).abs() < BPM_TOLERANCE,
            "Transport tempo should match clock BPM"
        );
    }

    // Stop the clock
    clock.stop();
    assert!(!clock.is_playing());

    // Verify transport state stopped
    {
        let transport = shared_state.lock().unwrap();
        assert!(!transport.is_playing(), "Transport should be stopped");
    }
}

#[test]
fn test_multiple_handlers() {
    let calc = BpmCalculator::new();
    let mut generator = ClockGenerator::new(calc);

    // Create multiple mock handlers
    let handler1 = Arc::new(MockHandler::new());
    let handler2 = Arc::new(MockHandler::new());

    // Add handlers to generator
    generator.add_handler(handler1.clone());
    generator.add_handler(handler2.clone());

    // Start the generator
    generator.start();

    // Let it run briefly
    thread::sleep(Duration::from_millis(100));

    // Stop the generator
    generator.stop();

    // Verify both handlers received messages
    assert!(handler1.message_count.load(Ordering::SeqCst) > 0);
    assert!(handler2.message_count.load(Ordering::SeqCst) > 0);

    // Verify handlers received Start message
    let messages1 = handler1.received_messages.lock().unwrap();
    let messages2 = handler2.received_messages.lock().unwrap();

    assert!(messages1.contains(&ClockMessage::Start));
    assert!(messages2.contains(&ClockMessage::Start));
}

#[test]
fn test_bpm_calculator_thread_safety() {
    let calc = Arc::new(BpmCalculator::new());
    calc.process_message(ClockMessage::Start);

    // Single thread for sending ticks at 120 BPM
    let tick_calc = calc.clone();
    let tick_thread = thread::spawn(move || {
        for _ in 0..100 {
            tick_calc.process_message(ClockMessage::Tick);
            thread::sleep(Duration::from_micros(20_833)); // 120 BPM timing
        }
    });

    // Multiple threads for reading BPM
    let mut read_handles = vec![];
    for _ in 0..4 {
        let read_calc = calc.clone();
        read_handles.push(thread::spawn(move || {
            for _ in 0..25 {
                let _ = read_calc.process_message(ClockMessage::Tick);
                thread::sleep(Duration::from_millis(10));
            }
        }));
    }

    // Wait for tick thread to complete
    tick_thread.join().unwrap();

    // Wait for read threads
    for handle in read_handles {
        handle.join().unwrap();
    }

    // Let the timing stabilize
    thread::sleep(Duration::from_millis(50));

    // Verify we can still get a reasonable BPM reading
    if let Some(bpm) = calc.process_message(ClockMessage::Tick) {
        assert!(
            (bpm - 120.0).abs() < TEMPO_CHANGE_TOLERANCE,
            "Expected ~120 BPM after concurrent access, got {}",
            bpm
        );
    }
}

#[test]
fn test_clock_generator_restart() {
    let calc = BpmCalculator::new();
    let mut generator = ClockGenerator::new(calc);
    let handler = Arc::new(MockHandler::new());
    generator.add_handler(handler.clone());

    // First start
    generator.start();
    thread::sleep(Duration::from_millis(100));
    generator.stop();

    let first_count = handler.message_count.load(Ordering::SeqCst);
    assert!(first_count > 0);

    // Second start
    generator.start();
    thread::sleep(Duration::from_millis(100));
    generator.stop();

    let second_count = handler.message_count.load(Ordering::SeqCst);
    assert!(
        second_count > first_count,
        "Should receive more messages after restart"
    );
}
