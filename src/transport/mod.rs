use crate::SharedState;
use std::{thread, time::Duration};

pub const TICKS_PER_BEAT: u32 = 24; // MIDI standard PPQ

pub fn run_timing_simulation(state: SharedState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if let Ok(transport) = state.lock() {
                transport.tick();
            }

            // Sleep for duration based on BPM
            // At 120 BPM, one beat is 500ms, so one tick is ~20.8ms
            thread::sleep(Duration::from_millis(21));
        }
    })
}

pub fn run_midi_input<T>(engine: T, _state: SharedState) -> thread::JoinHandle<()>
where
    T: crate::midi::MidiEngine + Send + 'static,
{
    thread::spawn(move || {
        println!("MIDI input thread started");
        loop {
            match engine.recv() {
                Ok(msg) => {
                    println!("Received MIDI message: {:?}", msg);
                    // TODO: Handle different MIDI message types
                }
                Err(e) => {
                    eprintln!("Error receiving MIDI message: {}", e);
                    break;
                }
            }
        }
    })
}
