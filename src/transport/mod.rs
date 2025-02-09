//! Transport and timing functionality
//!
//! This module handles the transport and timing aspects of PhasorSync, including:
//! - MIDI timing simulation
//! - MIDI input processing
//! - Transport state management
//!
//! The transport system uses the MIDI standard of 24 PPQN (Pulses Per Quarter Note)
//! for timing resolution.

mod input;
mod timing;

pub use input::run_midi_input;
pub use timing::{run_timing_simulation, TICKS_PER_BEAT};
