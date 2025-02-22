// clock.rs

use std::thread;
use std::time::Duration;

#[allow(dead_code)]
pub trait ClockSource {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>);
}

pub struct InternalClock {
    bpm: u32,
}

impl InternalClock {
    pub fn new() -> Self {
        InternalClock { bpm: 120 }
    }

    pub fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        let interval = (60_000 / self.bpm) as u64;
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(interval));
            tick_callback();
        });
    }
}
