use phasorsyncrs::midi::{BpmCalculator, ClockGenerator, ClockMessage};
use std::thread;
use std::time::Duration;

// Increased tolerance to account for system timing variations
const BPM_TOLERANCE: f64 = 5.0;
const TEMPO_CHANGE_TOLERANCE: f64 = 8.0;

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
