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

use clock::core::ClockCore;
use std::sync::{Arc, Mutex};

/// Common trait for MIDI clock implementations
pub trait MidiClock {
    /// Get access to the core clock implementation
    fn core(&self) -> &Arc<Mutex<ClockCore>>;

    /// Start the clock
    fn start(&mut self);

    /// Stop the clock
    fn stop(&mut self);

    /// Process a clock message
    fn handle_message(&mut self, msg: ClockMessage);

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

// Set default engine type
pub type DefaultMidiEngine = MidirEngine;
