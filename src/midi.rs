//! Module for handling MIDI device interactions

/// Lists all available MIDI devices
pub fn list_devices() -> Vec<String> {
    // TODO: Implement actual MIDI device listing
    // For now return mock data for TDD
    vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
}
