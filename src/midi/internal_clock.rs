use super::clock::{BpmCalculator, ClockGenerator, ClockMessage, ClockMessageHandler};
use crate::SharedState;
use log::info;
use std::sync::Arc;
use std::time::Duration;

struct TransportHandler {
    shared_state: SharedState,
}

impl ClockMessageHandler for TransportHandler {
    fn handle_message(&self, msg: ClockMessage) -> Option<f64> {
        match msg {
            ClockMessage::Start => {
                if let Ok(transport) = self.shared_state.lock() {
                    transport.set_playing(true);
                    info!("Internal clock started playback");
                }
            }
            ClockMessage::Stop => {
                if let Ok(transport) = self.shared_state.lock() {
                    transport.set_playing(false);
                    info!("Internal clock stopped playback");
                }
            }
            ClockMessage::Continue => {
                if let Ok(transport) = self.shared_state.lock() {
                    transport.set_playing(true);
                    info!("Internal clock resumed playback");
                }
            }
            ClockMessage::Tick => {
                if let Ok(transport) = self.shared_state.lock() {
                    transport.tick();
                }
            }
        }
        None
    }
}

pub struct InternalClock {
    clock_generator: ClockGenerator,
    transport_handler: Arc<TransportHandler>,
}

impl InternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        let mut clock_generator = ClockGenerator::new(BpmCalculator::new());

        // Create and register the transport handler
        let handler = Arc::new(TransportHandler { shared_state });

        // Create a new Arc<dyn ClockMessageHandler> from the handler
        let handler_trait: Arc<dyn ClockMessageHandler> = handler.clone();
        clock_generator.add_handler(handler_trait);

        Self {
            clock_generator,
            transport_handler: handler,
        }
    }

    pub fn start(&mut self) {
        self.clock_generator.start();
        info!("Internal clock started");
    }

    pub fn stop(&mut self) {
        self.clock_generator.stop();
        // Send stop message to handlers after stopping the thread
        self.transport_handler.handle_message(ClockMessage::Stop);
        info!("Internal clock stopped");
    }

    pub fn is_playing(&self) -> bool {
        // Check the transport state directly
        if let Ok(transport) = self.transport_handler.shared_state.lock() {
            transport.is_playing()
        } else {
            false
        }
    }

    pub fn current_bpm(&self) -> Option<f64> {
        self.clock_generator.current_bpm()
    }
}

pub fn run_internal_clock(shared_state: SharedState) {
    let mut clock = InternalClock::new(shared_state);
    info!("Internal clock initialized");

    clock.start();

    // Keep the thread alive and check status periodically
    loop {
        std::thread::sleep(Duration::from_secs(1));

        // TODO: Add proper shutdown mechanism
        // For now, this will run indefinitely
    }
}
