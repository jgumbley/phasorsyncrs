//! MIDI functionality module

mod alsa;
mod clock;
mod engine;

// Re-export engine types
pub use engine::{MidiEngine, MidiMessage, Result};

// Re-export clock types
pub use clock::{BpmCalculator, ClockGenerator, ClockMessage};

// Re-export device listing function
pub use alsa::list_devices;
