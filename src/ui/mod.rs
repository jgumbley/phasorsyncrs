//! User interface components
//!
//! This module provides terminal-based UI components for PhasorSync, including:
//! - Beat and bar progress indicators
//! - Transport state display
//! - Real-time state inspection
//!
//! The UI is built using the indicatif library for progress bars and spinners.

mod inspector;
mod progress;

pub use inspector::run_state_inspector;
pub use progress::{create_bar_progress, create_beat_progress, create_transport_spinner};
