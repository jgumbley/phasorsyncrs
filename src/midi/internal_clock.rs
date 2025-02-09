use super::clock::{core::ClockCore, ClockGenerator, ClockMessage, ClockMessageHandler};
use crate::midi::MidiClock;
use crate::SharedState;
use log::info;
use std::sync::{Arc, Mutex};

// Define a struct to wrap our handler
struct CoreMessageHandler {
    core: Arc<Mutex<ClockCore>>,
}

impl ClockMessageHandler for CoreMessageHandler {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64> {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(msg)
        } else {
            None
        }
    }
}

pub struct InternalClock {
    clock_generator: ClockGenerator,
    core: Arc<Mutex<ClockCore>>,
}

impl InternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        let core = ClockCore::new(shared_state);
        let mut clock_generator = ClockGenerator::new(core.lock().unwrap().bpm_calculator());

        // Create a handler that forwards messages to the core
        let handler = Arc::new(CoreMessageHandler { core: core.clone() });

        clock_generator.add_handler(handler);

        Self {
            clock_generator,
            core,
        }
    }
}

impl MidiClock for InternalClock {
    fn core(&self) -> &Arc<Mutex<ClockCore>> {
        &self.core
    }

    fn start(&mut self) {
        self.clock_generator.start();
        info!("Internal clock started");
    }

    fn stop(&mut self) {
        self.clock_generator.stop();
        // Send stop message to core after stopping the thread
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
