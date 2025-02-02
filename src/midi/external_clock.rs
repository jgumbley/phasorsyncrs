use super::clock::{BpmCalculator, ClockMessage};
use crate::midi::{MidiEngine, MidiMessage};
use crate::SharedState;
use std::sync::Arc;

pub struct ExternalClock {
    bpm_calculator: Arc<BpmCalculator>,
    shared_state: SharedState,
}

impl ExternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            bpm_calculator: Arc::new(BpmCalculator::new()),
            shared_state,
        }
    }

    pub fn handle_midi_message(&self, msg: MidiMessage) {
        // Convert MIDI message to clock message
        let clock_msg = match msg {
            MidiMessage::Clock => Some(ClockMessage::Tick),
            MidiMessage::Start => Some(ClockMessage::Start),
            MidiMessage::Stop => Some(ClockMessage::Stop),
            MidiMessage::Continue => Some(ClockMessage::Continue),
            _ => None,
        };

        // Process clock message if applicable
        if let Some(clock_msg) = clock_msg {
            // Update BPM calculator
            if let Some(bpm) = self.bpm_calculator.process_message(clock_msg.clone()) {
                // Update transport state with new BPM and timing info
                let mut transport = self.shared_state.lock().unwrap();
                transport.set_tempo(bpm as f32);

                match clock_msg {
                    ClockMessage::Tick => transport.tick(),
                    ClockMessage::Start => transport.set_playing(true),
                    ClockMessage::Stop => transport.set_playing(false),
                    ClockMessage::Continue => transport.set_playing(true),
                }
            }
        }
    }
}

pub fn run_external_clock<T>(engine: T, shared_state: SharedState)
where
    T: MidiEngine + Send + 'static,
{
    let clock = ExternalClock::new(shared_state);

    println!("External MIDI clock started");
    loop {
        match engine.recv() {
            Ok(msg) => {
                clock.handle_midi_message(msg);
            }
            Err(e) => {
                eprintln!("Error receiving MIDI message: {}", e);
                break;
            }
        }
    }
}
