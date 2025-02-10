//! MIDI functionality for PhasorSync
//!
//! This module provides MIDI communication capabilities, including:
//! - Core MIDI message types and error handling
//! - MIDI clock generation and BPM calculation
//! - Real MIDI device communication via midir
//! - Mock implementations for testing
//!
//! The main components are:
//! - [`MidiEngine`] trait for sending and receiving MIDI messages
//! - [`MidirEngine`] for real MIDI device communication
//! - [`MockMidiEngine`] for testing
//! - [`BpmCalculator`] for MIDI timing
//!
mod clock;
mod engine;
mod external_clock;
mod internal_clock;
mod internal_engine;
pub mod midir_engine; // Make the module public
pub mod mock_engine; // Make the module public

// Re-export main types from engine
pub use engine::{MidiEngine, MidiError, MidiMessage, Result};

// Re-export concrete implementations
pub use internal_engine::InternalEngine;
pub use midir_engine::MidirEngine;
pub use mock_engine::MockMidiEngine;

// Re-export clock functionality
pub use clock::{BpmCalculator, Clock, ClockMessage, ClockMessageHandler};

// Re-export internal clock functionality
pub use internal_clock::InternalClock;

// Re-export external clock functionality
pub use external_clock::{run_external_clock, ExternalClock};

/// Extended trait for MIDI clock implementations with convenience methods
pub trait MidiClock: Clock {
    /// Check if the clock is currently playing
    fn is_playing(&self) -> bool {
        if let Ok(core) = self.core().lock() {
            core.is_playing()
        } else {
            false
        }
    }

    /// Get the current BPM if available
    fn current_bpm(&self) -> Option<f64> {
        if let Ok(core) = self.core().lock() {
            core.current_bpm()
        } else {
            None
        }
    }
}

// Implement MidiClock for any type that implements Clock
impl<T: Clock> MidiClock for T {}

// Set default engine type
pub type DefaultMidiEngine = MidirEngine;

// New enum for MIDI engine types
pub enum MidiEngineType {
    Internal(InternalClock),
    External(MidirEngine),
    Mock(MockMidiEngine),
}
