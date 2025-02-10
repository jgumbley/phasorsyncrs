use super::clock::{core::ClockCore, Clock, ClockMessage};
use crate::midi::InternalEngine;
use crate::SharedState;
use log::info;
use std::sync::{Arc, Mutex};

pub struct InternalClock {
    core: Arc<Mutex<ClockCore>>,
    engine: InternalEngine,
    tick_thread: Option<std::thread::JoinHandle<()>>,
}

impl InternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        let engine = InternalEngine::new(shared_state.clone());
        let core = ClockCore::new(shared_state);
        Self {
            core,
            engine,
            tick_thread: None,
        }
    }
}

impl Clock for InternalClock {
    fn core(&self) -> &Arc<Mutex<ClockCore>> {
        &self.core
    }

    fn start(&mut self) {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(ClockMessage::Start);
        }
        // Start tick generation at 120 BPM
        self.tick_thread = Some(self.engine.start_tick_generator(120.0));
        info!("Internal clock started");
    }

    fn stop(&mut self) {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(ClockMessage::Stop);
        }
        info!("Internal clock stopped");
    }

    fn handle_message(&mut self, msg: ClockMessage) {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(msg);
        }
    }
}
