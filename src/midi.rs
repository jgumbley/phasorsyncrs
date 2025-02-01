//! Module for handling MIDI device interactions

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Generates MIDI clock messages at a fixed BPM
pub struct ClockGenerator {
    bpm_calculator: Arc<BpmCalculator>,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl ClockGenerator {
    /// Creates a new clock generator that will send messages to the provided BpmCalculator
    pub fn new(bpm_calculator: BpmCalculator) -> Self {
        Self {
            bpm_calculator: Arc::new(bpm_calculator),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Starts the clock at 120 BPM
    pub fn start(&mut self) {
        if self.thread_handle.is_some() {
            return; // Already running
        }

        // Send start message before spawning thread
        self.bpm_calculator.process_message(ClockMessage::Start);

        let bpm_calculator = Arc::clone(&self.bpm_calculator);
        let running = Arc::clone(&self.running);

        self.running.store(true, Ordering::SeqCst);

        self.thread_handle = Some(thread::spawn(move || {
            // Calculate tick interval: (60 seconds / 120 BPM) / 24 ticks per beat
            let tick_interval = Duration::from_micros(20_833); // 20.833ms per tick at 120 BPM

            while running.load(Ordering::SeqCst) {
                let tick_start = Instant::now();

                // Send tick message
                bpm_calculator.process_message(ClockMessage::Tick);

                // Calculate sleep duration to maintain precise timing
                let elapsed = tick_start.elapsed();
                if elapsed < tick_interval {
                    thread::sleep(tick_interval - elapsed);
                }
            }
        }));
    }

    /// Stops the clock thread but maintains the playing state
    pub fn stop(&mut self) {
        // Set running to false to stop the thread
        self.running.store(false, Ordering::SeqCst);

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Returns the current BPM if available
    pub fn current_bpm(&self) -> Option<f64> {
        self.bpm_calculator.process_message(ClockMessage::Tick)
    }

    /// Returns whether the clock is currently playing
    pub fn is_playing(&self) -> bool {
        self.bpm_calculator.is_playing()
    }
}

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
    state: std::sync::Mutex<BpmState>,
    ppq: u32,           // Pulses (ticks) Per Quarter note
    window_size: usize, // Number of intervals to average
}

#[derive(Debug)]
struct BpmState {
    last_tick_time: Option<Instant>,
    is_playing: bool,
    tick_count: u32,
    intervals: Vec<Duration>,
}

impl Default for BpmCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl BpmCalculator {
    /// Returns whether the calculator is currently in playing state
    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().is_playing
    }

    /// Creates a new BPM calculator with standard MIDI timing (24 PPQ)
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(BpmState {
                last_tick_time: None,
                is_playing: false,
                tick_count: 0,
                intervals: Vec::with_capacity(32),
            }),
            ppq: 24,         // Standard MIDI timing
            window_size: 24, // Average over one quarter note worth of ticks
        }
    }

    /// Process a MIDI clock message and return the current BPM if it can be calculated
    pub fn process_message(&self, msg: ClockMessage) -> Option<f64> {
        let mut state = self.state.lock().unwrap();
        match msg {
            ClockMessage::Start => {
                state.is_playing = true;
                state.last_tick_time = None;
                state.tick_count = 0;
                state.intervals.clear();
                None
            }
            ClockMessage::Stop => {
                state.is_playing = false;
                None
            }
            ClockMessage::Continue => {
                state.is_playing = true;
                None
            }
            ClockMessage::Tick => {
                if !state.is_playing {
                    return None;
                }

                let now = Instant::now();
                if let Some(last_time) = state.last_tick_time {
                    let interval = now.duration_since(last_time);
                    state.intervals.push(interval);

                    // Keep only the most recent intervals within our window
                    if state.intervals.len() > self.window_size {
                        state.intervals.remove(0);
                    }
                }

                state.last_tick_time = Some(now);
                state.tick_count += 1;

                // Need at least a few intervals to calculate meaningful BPM
                if state.intervals.len() >= 3 {
                    let avg_interval: Duration =
                        state.intervals.iter().sum::<Duration>() / state.intervals.len() as u32;
                    let ticks_per_minute = 60.0 / avg_interval.as_secs_f64();
                    Some(ticks_per_minute / self.ppq as f64)
                } else {
                    None
                }
            }
        }
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
