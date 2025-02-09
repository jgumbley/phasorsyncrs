use crate::midi::{InternalClock, MidiClock};
use crate::SharedState;
use std::thread;

/// MIDI standard PPQ (Pulses Per Quarter Note)
pub const TICKS_PER_BEAT: u32 = 24;

pub fn run_timing_simulation(state: SharedState) -> thread::JoinHandle<()> {
    let mut internal_clock = InternalClock::new(state.clone());
    internal_clock.start();

    thread::spawn(move || {
        // This thread exists just to maintain API compatibility
        // The actual timing is now handled by the internal clock
        loop {
            thread::park();
        }
    })
}
