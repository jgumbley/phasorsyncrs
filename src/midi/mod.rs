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
//! - [`ClockGenerator`] and [`BpmCalculator`] for MIDI timing
//!
mod clock;
mod engine;
mod external_clock;
mod internal_clock;
pub mod midir_engine; // Make the module public
pub mod mock_engine; // Make the module public

// Re-export main types from engine
pub use engine::{MidiEngine, MidiError, MidiMessage, Result};

// Re-export concrete implementations
pub use midir_engine::MidirEngine;
pub use mock_engine::MockMidiEngine;

// Re-export clock functionality
pub use clock::{BpmCalculator, ClockGenerator, ClockMessage, ClockMessageHandler};

// Re-export internal clock functionality
pub use internal_clock::{run_internal_clock, InternalClock};

// Re-export external clock functionality
pub use external_clock::{run_external_clock, ExternalClock};

// Export MidiClock trait
pub trait MidiClock {
    fn start(&mut self);
    fn stop(&mut self);
    fn is_playing(&self) -> bool;
    fn current_bpm(&self) -> Option<f64>;
    fn handle_message(&mut self, msg: ClockMessage);
}

// Set default engine type
pub type DefaultMidiEngine = MidirEngine;
