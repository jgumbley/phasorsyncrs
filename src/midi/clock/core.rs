use super::{BpmCalculator, ClockMessage};
use crate::SharedState;
use log::info;
use std::sync::{Arc, Mutex};

/// Handles transport state changes and BPM calculations in a unified way
pub struct ClockCore {
    bpm_calculator: Arc<BpmCalculator>,
    shared_state: SharedState,
}

impl ClockCore {
    pub fn new(shared_state: SharedState) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            bpm_calculator: Arc::new(BpmCalculator::new()),
            shared_state,
        }))
    }

    fn handle_transport_state(&mut self, msg: &ClockMessage) {
        let (playing, action) = match msg {
            ClockMessage::Start => (true, "started"),
            ClockMessage::Stop => (false, "stopped"),
            ClockMessage::Continue => (true, "resumed"),
            ClockMessage::Tick => return,
        };

        if let Ok(transport) = self.shared_state.lock() {
            transport.set_playing(playing);
            info!("Clock {} playback", action);
        }
    }

    fn handle_tick(&mut self) {
        if let Ok(transport) = self.shared_state.lock() {
            transport.tick();
        }
    }

    fn update_tempo(&mut self, bpm: f64) {
        if let Ok(mut transport) = self.shared_state.lock() {
            transport.set_tempo(bpm);
            info!("Clock tempo updated to {} BPM", bpm);
        }
    }

    pub fn process_message(&mut self, msg: ClockMessage) -> Option<f64> {
        // Handle transport state changes
        self.handle_transport_state(&msg);

        // Handle tick separately
        if let ClockMessage::Tick = msg {
            self.handle_tick();
        }

        // Process BPM calculation
        let bpm = self.bpm_calculator.process_message(msg);
        if let Some(bpm) = bpm {
            self.update_tempo(bpm);
        }
        bpm
    }

    pub fn is_playing(&self) -> bool {
        if let Ok(transport) = self.shared_state.lock() {
            transport.is_playing()
        } else {
            false
        }
    }

    pub fn current_bpm(&self) -> Option<f64> {
        self.bpm_calculator.current_bpm()
    }

    pub fn bpm_calculator(&self) -> Arc<BpmCalculator> {
        self.bpm_calculator.clone()
    }
}
