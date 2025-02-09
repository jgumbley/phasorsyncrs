use crate::midi::run_internal_clock;
use crate::SharedState;
use std::thread;

/// MIDI standard PPQ (Pulses Per Quarter Note)
pub const TICKS_PER_BEAT: u32 = 24;

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
