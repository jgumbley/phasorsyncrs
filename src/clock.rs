// clock.rs

use log::{debug, info};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
pub trait ClockSource {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>);
}

pub struct InternalClock {
    bpm: u32,
    tick_tx: Sender<()>,
}

impl InternalClock {
    pub fn new(tick_tx: Sender<()>) -> Self {
        info!("Creating new InternalClock with default BPM: 120");
        InternalClock { bpm: 120, tick_tx }
    }
}

impl ClockSource for InternalClock {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        info!("Starting InternalClock with BPM: {}", self.bpm);
        let interval = (60_000 / self.bpm) as u64;
        debug!("Calculated tick interval: {}ms", interval);
        let tick_tx = self.tick_tx.clone();

        thread::spawn(move || {
            info!("Internal clock thread started");
            let mut tick_count = 0;
            loop {
                let tick_interval = interval / 24;
                for _ in 0..24 {
                    thread::sleep(Duration::from_millis(tick_interval));
                    tick_callback();
                    tick_tx.send(()).unwrap();
                    tick_count += 1;
                }
                debug!("Internal clock beat: {}", tick_count / 24);
            }
        });
    }
}
