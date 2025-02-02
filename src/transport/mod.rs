use crate::SharedState;
use std::{thread, time::Duration};

pub const TICKS_PER_BEAT: u32 = 24; // MIDI standard PPQ

pub fn run_timing_simulation(state: SharedState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            {
                let mut transport = state.lock().unwrap();
                if !transport.is_playing {
                    transport.is_playing = true;
                }

                transport.tick_count += 1;

                // Update beat and bar
                if transport.tick_count % TICKS_PER_BEAT == 0 {
                    transport.beat += 1;
                    if transport.beat > 4 {
                        // Assuming 4/4 time signature
                        transport.beat = 1;
                        transport.bar += 1;
                    }
                }
            }

            // Sleep for duration based on BPM
            // At 120 BPM, one beat is 500ms, so one tick is ~20.8ms
            thread::sleep(Duration::from_millis(21));
        }
    })
}

pub fn run_midi_input(
    mut engine: crate::midi::MidirEngine,
    _state: SharedState,
) -> thread::JoinHandle<()> {
    use crate::midi::MidiEngine;

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
