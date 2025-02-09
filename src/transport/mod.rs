//! Transport functionality
//!
//! This module handles the transport aspects of PhasorSync, including:
//! - MIDI input processing
//! - Transport state management
//!
//! The transport system uses the MIDI standard of 24 PPQN (Pulses Per Quarter Note)
//! for timing resolution.

mod input;

pub use input::run_midi_input;

// MIDI standard timing resolution
pub const TICKS_PER_BEAT: u32 = 24;
