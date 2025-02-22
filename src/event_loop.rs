// event_loop.rs

use crate::state;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct EventLoop {
    shared_state: Arc<Mutex<state::SharedState>>,
}

impl EventLoop {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>) -> Self {
        EventLoop { shared_state }
    }

    pub fn run(&self) {
        loop {
            thread::sleep(Duration::from_millis(10)); // Simulate tick event
            let mut state = self.shared_state.lock().unwrap();
            state.tick_update();
        }
    }
}
