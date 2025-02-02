//! MIDI functionality module

mod clock;
mod engine;
mod external_clock;
pub mod midir_engine;
#[cfg(feature = "test-mock")]
pub mod mock_engine;

// Re-export engine types
pub use engine::{MidiEngine, MidiMessage, Result};

// Re-export clock types
pub use clock::{BpmCalculator, ClockGenerator, ClockMessage};

// Re-export external clock
pub use external_clock::{run_external_clock, ExternalClock};

// Re-export midir implementation
pub use midir_engine::MidirEngine;

// Re-export device listing function
#[cfg(not(feature = "test-mock"))]
pub use midir_engine::MidirEngine as DefaultMidiEngine;
#[cfg(feature = "test-mock")]
pub use mock_engine::MockMidiEngine as DefaultMidiEngine;
