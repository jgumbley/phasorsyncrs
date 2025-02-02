#![cfg(feature = "test-mock")]

use phasorsyncrs::midi::{run_external_clock, MidiEngine, MidiMessage};
use phasorsyncrs::transport::Transport;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_system_message_parsing() {
    use phasorsyncrs::midi::midir_engine::MidirEngine;

    // Test MIDI Clock message (0xF8)
    let clock_msg = vec![0xF8];
    let parsed = MidirEngine::parse_midi_message(&clock_msg);
    assert_eq!(parsed, Some(MidiMessage::Clock));

    // Test other system messages
    let start_msg = vec![0xFA];
    let stop_msg = vec![0xFC];
    let continue_msg = vec![0xFB];

    assert_eq!(
        MidirEngine::parse_midi_message(&start_msg),
        Some(MidiMessage::Start)
    );
    assert_eq!(
        MidirEngine::parse_midi_message(&stop_msg),
        Some(MidiMessage::Stop)
    );
    assert_eq!(
        MidirEngine::parse_midi_message(&continue_msg),
        Some(MidiMessage::Continue)
    );

    // Verify channel messages still work
    let note_on_msg = vec![0x90, 60, 100]; // Note On, channel 0, note 60, velocity 100
    assert_eq!(
        MidirEngine::parse_midi_message(&note_on_msg),
        Some(MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100
        })
    );
}

#[test]
fn test_midi_message_equality() {
    // Test MidiMessage enum equality comparisons
    assert_eq!(
        MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100
        },
        MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100
        }
    );

    assert_eq!(MidiMessage::Clock, MidiMessage::Clock);
    assert_eq!(MidiMessage::Start, MidiMessage::Start);
    assert_eq!(MidiMessage::Stop, MidiMessage::Stop);
    assert_eq!(MidiMessage::Continue, MidiMessage::Continue);

    assert_ne!(
        MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100
        },
        MidiMessage::NoteOff {
            channel: 0,
            note: 60,
            velocity: 100
        }
    );
}

// Mock implementation for testing
struct MockMidiEngine {
    #[allow(dead_code)]
    devices: Vec<String>,
    should_timeout: bool,
}

impl MockMidiEngine {
    pub fn list_devices() -> Vec<String> {
        vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
    }

    fn new() -> Self {
        Self {
            devices: vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()],
            should_timeout: false,
        }
    }

    fn with_timeout() -> Self {
        Self {
            devices: vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()],
            should_timeout: true,
        }
    }
}

impl MidiEngine for MockMidiEngine {
    fn send(&mut self, _msg: MidiMessage) -> phasorsyncrs::midi::Result<()> {
        Ok(())
    }

    fn recv(&self) -> phasorsyncrs::midi::Result<MidiMessage> {
        if self.should_timeout {
            thread::sleep(Duration::from_secs(6)); // Sleep longer than the timeout
        }
        Ok(MidiMessage::Clock)
    }
}

#[test]
fn test_mock_midi_engine() {
    let mut engine = MockMidiEngine::new();

    // Test device listing
    let devices = MockMidiEngine::list_devices();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0], "Mock Device 1");
    assert_eq!(devices[1], "Mock Device 2");

    // Test sending a message
    let result = engine.send(MidiMessage::NoteOn {
        channel: 0,
        note: 60,
        velocity: 100,
    });
    assert!(result.is_ok());

    // Test receiving a message
    let result = engine.recv();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), MidiMessage::Clock);
}

#[test]
#[ignore = "slow test: involves timeout waiting"]
fn test_external_clock_timeout() {
    let shared_state = Arc::new(Mutex::new(Transport::new()));
    let engine = MockMidiEngine::with_timeout();

    // Set initial playing state to true
    {
        let mut transport = shared_state.lock().unwrap();
        transport.set_playing(true);
    }

    // Run the external clock in a separate thread
    let shared_state_clone = shared_state.clone();
    let handle = thread::spawn(move || {
        run_external_clock(engine, shared_state_clone);
    });

    // Wait for the timeout to occur
    thread::sleep(Duration::from_secs(7));

    // Verify the transport state was updated
    let transport = shared_state.lock().unwrap();
    assert!(!transport.is_playing());

    handle.join().unwrap();
}
