use phasorsyncrs::midi::{MidiEngine, MidiMessage};

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

impl MockMidiEngine {
    pub fn list_devices() -> Vec<String> {
        vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
    }

    fn new() -> Self {
        Self {
            devices: vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()],
        }
    }
}

// Mock implementation for testing
struct MockMidiEngine {
    #[allow(dead_code)]
    devices: Vec<String>,
}

impl MidiEngine for MockMidiEngine {
    fn send(&mut self, _msg: MidiMessage) -> phasorsyncrs::midi::Result<()> {
        Ok(())
    }

    fn recv(&self) -> phasorsyncrs::midi::Result<MidiMessage> {
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
