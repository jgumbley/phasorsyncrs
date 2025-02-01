use phasorsyncrs::midi::{BpmCalculator, ClockMessage};
use std::thread;
use std::time::Duration;

#[test]
fn test_bpm_calculator_start_stop() {
    let mut calc = BpmCalculator::new();

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
    let mut calc = BpmCalculator::new();
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
        assert!((bpm - 120.0).abs() < 1.0, "Expected ~120 BPM, got {}", bpm);
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_tempo_change() {
    let mut calc = BpmCalculator::new();
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
        assert!((bpm - 140.0).abs() < 2.0, "Expected ~140 BPM, got {}", bpm);
    } else {
        panic!("Expected BPM calculation, got None");
    }
}

#[test]
fn test_continue_message() {
    let mut calc = BpmCalculator::new();

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
        assert!((bpm - 120.0).abs() < 1.0, "Expected ~120 BPM, got {}", bpm);
    } else {
        panic!("Expected BPM calculation, got None");
    }
}
