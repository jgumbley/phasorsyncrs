// clock.rs

use std::thread;
use std::time::Duration;
use log::{debug, info};

#[allow(dead_code)]
pub trait ClockSource {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>);
}

pub struct InternalClock {
    bpm: u32,
}

impl InternalClock {
    pub fn new() -> Self {
        info!("Creating new InternalClock with default BPM: 120");
        InternalClock { bpm: 120 }
    }
}

impl ClockSource for InternalClock {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        info!("Starting InternalClock with BPM: {}", self.bpm);
        let interval = (60_000 / self.bpm) as u64;
        debug!("Calculated tick interval: {}ms", interval);

        thread::spawn(move || {
            info!("Internal clock thread started");
            let mut tick_count = 0;
            loop {
                thread::sleep(Duration::from_millis(interval));
                tick_callback();
                tick_count += 1;
                if tick_count % 24 == 0 {  // Log every quarter note (24 MIDI ticks)
                    debug!("Internal clock tick: {}", tick_count);
                }
            }
        });
    }
}
