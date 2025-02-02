use crate::SharedState;
use std::{thread, time::Duration};

pub const TICKS_PER_BEAT: u32 = 24; // MIDI standard PPQ

#[derive(Debug)]
pub struct Transport {
    pub bpm: f32,
    pub tick_count: u32,
    pub beat: u32,
    pub bar: u32,
    pub is_playing: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport {
    pub fn new() -> Self {
        Self {
            bpm: 120.0,
            tick_count: 0,
            beat: 1,
            bar: 1,
            is_playing: false,
        }
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn tempo(&self) -> f32 {
        self.bpm
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn tick(&mut self) {
        // Only process ticks if we're playing
        if !self.is_playing {
            return;
        }

        self.tick_count += 1;

        // Update beat and bar
        if self.tick_count % TICKS_PER_BEAT == 0 {
            self.beat += 1;
            if self.beat > 4 {
                // Assuming 4/4 time signature
                self.beat = 1;
                self.bar += 1;
            }
        }
    }
}

pub fn run_timing_simulation(state: SharedState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            {
                let mut transport = state.lock().unwrap();
                transport.tick();
            }

            // Sleep for duration based on BPM
            // At 120 BPM, one beat is 500ms, so one tick is ~20.8ms
            thread::sleep(Duration::from_millis(21));
        }
    })
}

pub fn run_midi_input<T>(engine: T, _state: SharedState) -> thread::JoinHandle<()>
where
    T: crate::midi::MidiEngine + Send + 'static,
{
    thread::spawn(move || {
        println!("MIDI input thread started");
        loop {
            match engine.recv() {
                Ok(msg) => {
                    println!("Received MIDI message: {:?}", msg);
                    // TODO: Handle different MIDI message types
                }
                Err(e) => {
                    eprintln!("Error receiving MIDI message: {}", e);
                    break;
                }
            }
        }
    })
}
