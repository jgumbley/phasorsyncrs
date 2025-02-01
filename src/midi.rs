//! Module for handling MIDI device interactions

use std::time::{Duration, Instant};

/// Represents different types of MIDI clock messages
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ClockMessage {
    Tick,
    Start,
    Stop,
    Continue,
}

/// Handles BPM calculation from MIDI clock messages
#[derive(Debug)]
pub struct BpmCalculator {
    last_tick_time: Option<Instant>,
    is_playing: bool,
    tick_count: u32,
    ppq: u32, // Pulses (ticks) Per Quarter note
    intervals: Vec<Duration>,
    window_size: usize, // Number of intervals to average
}

impl Default for BpmCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl BpmCalculator {
    /// Returns whether the calculator is currently in playing state
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Creates a new BPM calculator with standard MIDI timing (24 PPQ)
    pub fn new() -> Self {
        Self {
            last_tick_time: None,
            is_playing: false,
            tick_count: 0,
            ppq: 24, // Standard MIDI timing
            intervals: Vec::with_capacity(32),
            window_size: 24, // Average over one quarter note worth of ticks
        }
    }

    /// Process a MIDI clock message and return the current BPM if it can be calculated
    pub fn process_message(&mut self, msg: ClockMessage) -> Option<f64> {
        match msg {
            ClockMessage::Start => {
                self.is_playing = true;
                self.reset();
                None
            }
            ClockMessage::Stop => {
                self.is_playing = false;
                None
            }
            ClockMessage::Continue => {
                self.is_playing = true;
                None
            }
            ClockMessage::Tick => {
                if !self.is_playing {
                    return None;
                }

                let now = Instant::now();
                if let Some(last_time) = self.last_tick_time {
                    let interval = now.duration_since(last_time);
                    self.intervals.push(interval);

                    // Keep only the most recent intervals within our window
                    if self.intervals.len() > self.window_size {
                        self.intervals.remove(0);
                    }
                }

                self.last_tick_time = Some(now);
                self.tick_count += 1;

                // Need at least a few intervals to calculate meaningful BPM
                if self.intervals.len() >= 3 {
                    Some(self.calculate_bpm())
                } else {
                    None
                }
            }
        }
    }

    /// Reset the calculator state
    pub fn reset(&mut self) {
        self.last_tick_time = None;
        self.tick_count = 0;
        self.intervals.clear();
    }

    /// Calculate BPM based on the average interval between ticks
    fn calculate_bpm(&self) -> f64 {
        let avg_interval: Duration =
            self.intervals.iter().sum::<Duration>() / self.intervals.len() as u32;
        let ticks_per_minute = 60.0 / avg_interval.as_secs_f64();
        ticks_per_minute / self.ppq as f64
    }
}

#[cfg(not(feature = "test-mock"))]
pub fn list_devices() -> Vec<String> {
    let seq = match alsa::Seq::open(None, None, false) {
        Ok(s) => s,
        Err(_) => return vec![], // Return empty list if we can't open sequencer
    };

    let mut devices = Vec::new();

    // Create client iterator and iterate through clients
    let client_iter = alsa::seq::ClientIter::new(&seq);

    for client_info in client_iter {
        let client_id = client_info.get_client();
        let client_name = client_info.get_name().unwrap_or_default();

        // Create port iterator and iterate through ports
        let port_iter = alsa::seq::PortIter::new(&seq, client_id);

        for port_info in port_iter {
            let port_id = port_info.get_port();

            // Create address for port info lookup
            let addr = alsa::seq::Addr {
                client: client_id,
                port: port_id,
            };

            if let Ok(port_info) = seq.get_any_port_info(addr) {
                if let Ok(port_name) = port_info.get_name() {
                    let caps = port_info.get_capability();
                    let mut capabilities = Vec::new();

                    if caps.contains(alsa::seq::PortCap::READ) {
                        capabilities.push("Input");
                    }
                    if caps.contains(alsa::seq::PortCap::WRITE) {
                        capabilities.push("Output");
                    }

                    if !capabilities.is_empty() {
                        devices.push(format!(
                            "{} - {} ({}:{}) [{}]",
                            client_name,
                            port_name,
                            client_id,
                            port_id,
                            capabilities.join("/")
                        ));
                    }
                }
            }
        }
    }

    devices
}

#[cfg(feature = "test-mock")]
pub fn list_devices() -> Vec<String> {
    // Mock implementation for tests - simple format as expected by tests
    vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
}
