// clock.rs

use crate::event_loop::EngineMessage;
use log::{info, trace};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub trait ClockSource {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>);
}

pub struct InternalClock {
    bpm: u32,
    tick_tx: Sender<EngineMessage>,
}

impl InternalClock {
    pub fn new(tick_tx: Sender<EngineMessage>) -> Self {
        info!("Creating new InternalClock with default BPM: 122");
        InternalClock { bpm: 122, tick_tx }
    }
}

impl ClockSource for InternalClock {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        info!("Starting InternalClock with BPM: {}", self.bpm);
        let beat_duration_us = 60_000_000 / self.bpm; // total microseconds per beat
        let tick_interval_us = beat_duration_us / 24; // microseconds per tick
        trace!("Calculated tick interval: {} µs", tick_interval_us);
        let tick_tx = self.tick_tx.clone();

        let start_time = Instant::now();

        thread::spawn(move || {
            info!("Internal clock thread started");
            let mut tick_count = 0;
            loop {
                for _ in 0..24 {
                    let start = Instant::now();
                    thread::sleep(Duration::from_micros(tick_interval_us as u64));
                    let end = Instant::now();
                    let sleep_duration = end.duration_since(start);
                    trace!("Sleep duration: {:?}", sleep_duration);

                    let now = Instant::now();
                    let elapsed = now.duration_since(start_time).as_millis();
                    trace!("InternalClock tick at {} ms", elapsed);

                    tick_callback();
                    tick_tx.send(EngineMessage::Tick).unwrap();
                    tick_count += 1;
                }
                trace!("Internal clock beat: {}", tick_count / 24);
            }
        });
    }
}
