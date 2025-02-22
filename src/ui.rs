// ui.rs

use crate::state;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct UI {
    shared_state: Arc<Mutex<state::SharedState>>,
}

impl UI {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>) -> Self {
        UI { shared_state }
    }

    pub fn run(&self) {
        loop {
            thread::sleep(Duration::from_millis(100));
            let state = self.shared_state.lock().unwrap();
            println!(
                "BPM: {}, Tick Count: {}, Beat: {}, Bar: {}",
                state.get_bpm(),
                state.get_tick_count(),
                state.get_current_beat(),
                state.get_current_bar()
            );
        }
    }
}
