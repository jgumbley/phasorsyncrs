//! MIDI clock and BPM calculation functionality

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Trait for handling MIDI clock messages
pub trait ClockMessageHandler: Send + Sync {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64>;
}

/// Generates MIDI clock messages at a fixed BPM
pub struct ClockGenerator {
    bpm_calculator: Arc<BpmCalculator>,
    handlers: Vec<Arc<dyn ClockMessageHandler>>,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl ClockGenerator {
    /// Creates a new clock generator that will send messages to the provided BpmCalculator
    pub fn new(bpm_calculator: BpmCalculator) -> Self {
        Self {
            bpm_calculator: Arc::new(bpm_calculator),
            handlers: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Add a message handler
    pub fn add_handler(&mut self, handler: Arc<dyn ClockMessageHandler>) {
        self.handlers.push(handler);
    }

    /// Starts the clock at 120 BPM
    pub fn start(&mut self) {
        if self.thread_handle.is_some() {
            return; // Already running
        }

        // Send start message before spawning thread
        self.bpm_calculator.process_message(ClockMessage::Start);
        for handler in &self.handlers {
            handler.handle_message(ClockMessage::Start);
        }

        let bpm_calculator = Arc::clone(&self.bpm_calculator);
        let handlers = self.handlers.clone();
        let running = Arc::clone(&self.running);

        self.running.store(true, Ordering::SeqCst);

        self.thread_handle = Some(thread::spawn(move || {
            // Calculate tick interval: (60 seconds / 120 BPM) / 24 ticks per beat
            let tick_interval = Duration::from_micros(20_833); // 20.833ms per tick at 120 BPM

            while running.load(Ordering::SeqCst) {
                let tick_start = Instant::now();

                // Send tick message to all handlers
                bpm_calculator.process_message(ClockMessage::Tick);
                for handler in &handlers {
                    handler.handle_message(ClockMessage::Tick);
                }

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
        self.bpm_calculator.current_bpm()
    }

    /// Returns whether the clock is currently playing
    pub fn is_playing(&self) -> bool {
        // Consider the clock playing if it was started, even if the thread is stopped
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

impl ClockMessageHandler for BpmCalculator {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64> {
        self.process_message(msg)
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

                    // Only include reasonable intervals (filter out extreme values)
                    if interval.as_micros() > 1000 && interval.as_micros() < 100_000 {
                        state.intervals.push(interval);

                        // Keep only the most recent intervals within our window
                        while state.intervals.len() > self.window_size {
                            state.intervals.remove(0);
                        }
                    }
                }

                state.last_tick_time = Some(now);
                state.tick_count += 1;

                // Need at least a few intervals to calculate meaningful BPM
                if state.intervals.len() >= 3 {
                    // Sort intervals and take the median section to avoid outliers
                    let mut intervals = state.intervals.clone();
                    intervals.sort_by_key(|d| d.as_nanos());

                    let start_idx = intervals.len() / 4;
                    let end_idx = (intervals.len() * 3) / 4;
                    let median_intervals = &intervals[start_idx..end_idx];

                    let avg_interval: Duration =
                        median_intervals.iter().sum::<Duration>() / median_intervals.len() as u32;
                    let ticks_per_minute = 60.0 / avg_interval.as_secs_f64();
                    Some(ticks_per_minute / self.ppq as f64)
                } else {
                    None
                }
            }
        }
    }

    /// Returns the current BPM if it can be calculated
    pub fn current_bpm(&self) -> Option<f64> {
        let state = self.state.lock().unwrap();
        if state.intervals.len() >= 3 {
            let mut intervals = state.intervals.clone();
            intervals.sort_by_key(|d| d.as_nanos());

            let start_idx = intervals.len() / 4;
            let end_idx = (intervals.len() * 3) / 4;
            let median_intervals = &intervals[start_idx..end_idx];

            let avg_interval: Duration =
                median_intervals.iter().sum::<Duration>() / median_intervals.len() as u32;
            let ticks_per_minute = 60.0 / avg_interval.as_secs_f64();
            Some(ticks_per_minute / self.ppq as f64)
        } else {
            None
        }
    }
}
