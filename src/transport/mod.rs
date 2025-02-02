use crate::midi::run_internal_clock;
use crate::SharedState;
use std::thread;

pub const TICKS_PER_BEAT: u32 = 24; // MIDI standard PPQ

pub fn run_timing_simulation(state: SharedState) -> thread::JoinHandle<()> {
    // Replace the basic timing simulation with our new internal clock
    run_internal_clock(state.clone());
    thread::spawn(move || {
        // This thread exists just to maintain API compatibility
        // The actual timing is now handled by the internal clock
        loop {
            thread::park();
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
