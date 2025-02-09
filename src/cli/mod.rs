//! Command-line interface functionality
//!
//! This module provides command-line argument parsing and device validation for PhasorSync.
//! It handles:
//! - Parsing command line arguments using clap
//! - Listing available MIDI devices
//! - Validating MIDI device selection

mod args;

pub use args::{handle_device_list, validate_device, Args};
