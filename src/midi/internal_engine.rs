use std::sync::{Arc, Mutex};
use crate::midi::clock::core::ClockCore;
use crate::midi::MidiEngine;
use crate::midi::MidiMessage;
use std::thread;

pub struct InternalEngine {
    core: Arc<Mutex<ClockCore>>,
}

impl MidiEngine for InternalEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<(), String> {
        self.core.lock().unwrap().process_message(msg);
        Ok(())
    }
}

impl InternalEngine {
    pub fn start(&self) -> thread::JoinHandle<()> {
        let core = self.core.clone();
        thread::spawn(move || {
            while core.lock().unwrap().is_running() {
                // Clock generation logic
            }
        })
    }
}