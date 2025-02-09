use crate::midi::clock::core::ClockCore;
use crate::midi::{ClockMessage, MidiEngine, MidiError, MidiMessage, Result};
use crate::SharedState;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct InternalEngine {
    core: Arc<Mutex<ClockCore>>,
}

impl MidiEngine for InternalEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()> {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(msg.into());
        }
        Ok(())
    }

    fn recv(&self) -> Result<MidiMessage> {
        Err(MidiError::RecvError(
            "Internal engine does not support receiving MIDI messages".to_string(),
        ))
    }
}

impl InternalEngine {
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            core: ClockCore::new(shared_state),
        }
    }

    pub fn start_tick_generator(&self, bpm: f32) -> thread::JoinHandle<()> {
        let core = self.core.clone();
        thread::spawn(move || {
            let tick_duration = Duration::from_secs_f32(60.0 / (bpm * 24.0)); // 24 TICKS_PER_BEAT
            loop {
                thread::sleep(tick_duration);
                if let Ok(mut guard) = core.lock() {
                    guard.process_message(ClockMessage::Tick);
                }
            }
        })
    }
}
