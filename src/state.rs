// state.rs

use crate::config::{BARS_PER_PHRASE, BEATS_PER_BAR, TICKS_PER_BEAT};

pub struct SharedState {
    pub bpm: u32,
    tick_count: u64,
    current_beat: u32,
    current_bar: u32,
}

impl SharedState {
    pub fn new(bpm: u32) -> Self {
        SharedState {
            bpm,
            tick_count: 0,
            current_beat: 0,
            current_bar: 0,
        }
    }

    pub fn tick_update(&mut self) {
        self.tick_count += 1;

        // Calculate the tick position within the current beat.
        let _tick_in_beat = self.tick_count % TICKS_PER_BEAT;

        // Calculate the current beat (0-indexed) within a bar.
        let beat_number = (self.tick_count / TICKS_PER_BEAT) % BEATS_PER_BAR;

        // Calculate the current bar (0-indexed) within a phrase.
        let bar_number = (self.tick_count / (TICKS_PER_BEAT * BEATS_PER_BAR)) % BARS_PER_PHRASE;

        // Update the shared state with 1-indexed values for display.
        self.current_beat = (beat_number + 1) as u32;
        self.current_bar = (bar_number + 1) as u32;
    }

    pub fn get_bpm(&self) -> u32 {
        self.bpm
    }

    pub fn get_tick_count(&self) -> u64 {
        self.tick_count
    }

    #[allow(dead_code)]
    pub fn get_current_beat(&self) -> u32 {
        self.current_beat
    }

    pub fn get_current_bar(&self) -> u32 {
        self.current_bar
    }
}
