use crate::midi::{MidiEngine, MidiMessage, Result};

pub struct MockMidiEngine;

impl MockMidiEngine {
    pub fn new(_device_name: Option<String>) -> Result<Self> {
        Ok(MockMidiEngine)
    }
}

impl MidiEngine for MockMidiEngine {
    fn send(&mut self, _msg: MidiMessage) -> Result<()> {
        Ok(())
    }

    fn recv(&mut self) -> Result<MidiMessage> {
        Ok(MidiMessage::Clock)
    }

    fn list_devices(&self) -> Vec<String> {
        vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
    }
}
