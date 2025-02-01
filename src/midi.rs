//! Module for handling MIDI device interactions

#[cfg(not(feature = "test-mock"))]
pub fn list_devices() -> Vec<String> {
    // TODO: Implement actual MIDI device listing
    // This will be the real implementation
    vec![] // Return empty for now until real implementation
}

#[cfg(feature = "test-mock")]
pub fn list_devices() -> Vec<String> {
    // Mock implementation for tests
    vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
}
