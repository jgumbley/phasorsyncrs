//! MIDI clock and BPM calculation functionality
pub mod core;

use crate::midi::MidiMessage;
use core::ClockCore;
use std::sync::{Arc, Mutex};
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

impl From<MidiMessage> for ClockMessage {
    fn from(msg: MidiMessage) -> Self {
        match msg {
            MidiMessage::Clock => ClockMessage::Tick,
            MidiMessage::Start => ClockMessage::Start,
            MidiMessage::Stop => ClockMessage::Stop,
            MidiMessage::Continue => ClockMessage::Continue,
            _ => ClockMessage::Tick, // Default for other MIDI messages
        }
    }
}

/// Core clock interface for handling timing and transport
pub trait Clock: Send + Sync {
    fn start(&mut self);
    fn stop(&mut self);
    fn handle_message(&mut self, msg: ClockMessage);
    fn core(&self) -> &Arc<Mutex<ClockCore>>;
}

/// Trait for handling MIDI clock messages
pub trait ClockMessageHandler: Send + Sync {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64>;
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
                        self.maintain_interval_window(&mut state);
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

    /// Maintains the interval window size
    fn maintain_interval_window(&self, state: &mut BpmState) {
        while state.intervals.len() > self.window_size {
            state.intervals.remove(0);
        }
    }
}
