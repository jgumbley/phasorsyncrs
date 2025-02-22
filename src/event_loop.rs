// event_loop.rs

use crate::state;
use log::{debug, error, info};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct EventLoop {
    shared_state: Arc<Mutex<state::SharedState>>,
    tick_rx: Receiver<()>,
    last_tick_time: Mutex<Option<Instant>>,
}

impl EventLoop {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>, tick_rx: Receiver<()>) -> Self {
        EventLoop {
            shared_state,
            tick_rx,
            last_tick_time: Mutex::new(None),
        }
    }

    pub fn run(&self) {
        loop {
            // Block until a tick event is received.
            match self.tick_rx.recv() {
                Ok(()) => {
                    // A tick event has been received.
                    let mut state = self.shared_state.lock().unwrap();
                    let now = Instant::now();
                    let mut last_tick_time = self.last_tick_time.lock().unwrap();

                    if let Some(last_time) = *last_tick_time {
                        let duration = now.duration_since(last_time);
                        let bpm = calculate_bpm(duration);
                        state.bpm = bpm;
                        info!("Calculated BPM: {}", bpm);
                    } else {
                        info!("First tick received, initializing last_tick_time");
                    }

                    *last_tick_time = Some(now);
                    state.tick_update();
                    debug!(
                        "Shared state updated: tick_count={}, current_beat={}, current_bar={}, bpm={}",
                        state.get_tick_count(),
                        state.get_current_beat(),
                        state.get_current_bar(),
                        state.get_bpm()
                    );
                }
                Err(e) => {
                    error!("Tick channel error: {}", e);
                    break;
                }
            }
        }
    }
}

fn calculate_bpm(duration: Duration) -> u32 {
    // 60 seconds / (duration in seconds * 24 ticks per beat)
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 {
        // Avoid division by zero
        return 60;
    }
    let bpm = 60.0 / (seconds * 24.0);
    bpm as u32
}
