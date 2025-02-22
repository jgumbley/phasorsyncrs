// event_loop.rs

use crate::state;
use log::{debug, error};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct EventLoop {
    shared_state: Arc<Mutex<state::SharedState>>,
    tick_rx: Receiver<()>,
}

impl EventLoop {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>, tick_rx: Receiver<()>) -> Self {
        EventLoop {
            shared_state,
            tick_rx,
        }
    }

    pub fn run(&self) {
        loop {
            // Block until a tick event is received.
            match self.tick_rx.recv() {
                Ok(()) => {
                    // A tick event has been received.
                    let mut state = self.shared_state.lock().unwrap();
                    state.tick_update();
                    debug!(
                        "Shared state updated: tick_count={}, current_beat={}, current_bar={}",
                        state.get_tick_count(),
                        state.get_current_beat(),
                        state.get_current_bar()
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
