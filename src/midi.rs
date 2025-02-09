//! MIDI functionality module

mod clock;
mod engine;
mod external_clock;
mod internal_clock;
pub mod midir_engine;
#[cfg(feature = "test-mock")]
pub mod mock_engine;

// Re-export engine types
pub use engine::{MidiEngine, MidiError, MidiMessage, Result};

// Re-export clock types
pub use clock::{BpmCalculator, ClockGenerator, ClockMessage, ClockMessageHandler};

// Re-export external clock
pub use external_clock::{run_external_clock, ExternalClock};

// Re-export internal clock
pub use internal_clock::{run_internal_clock, InternalClock};

// Re-export midir implementation
pub use midir_engine::MidirEngine;

// Re-export device listing function
#[cfg(not(feature = "test-mock"))]
pub use midir_engine::MidirEngine as DefaultMidiEngine;
#[cfg(feature = "test-mock")]
pub use mock_engine::MockMidiEngine as DefaultMidiEngine;

/// Trait defining common behavior for all MIDI clocks
pub trait MidiClock {
    /// Starts the clock
    fn start(&mut self);

    /// Stops the clock
    fn stop(&mut self);

    /// Checks if the clock is currently playing
    fn is_playing(&self) -> bool;

    /// Gets the current BPM (if available)
    fn current_bpm(&self) -> Option<f64>;

    /// Handles incoming MIDI messages
    fn handle_message(&mut self, msg: ClockMessage);
}
