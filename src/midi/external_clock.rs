use super::clock::{BpmCalculator, ClockMessage};
use crate::midi::{MidiEngine, MidiMessage};
use crate::SharedState;
use log::{error, info};
use std::sync::Arc;
use std::time::{Duration, Instant};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ExternalClock {
    bpm_calculator: Arc<BpmCalculator>,
    shared_state: SharedState,
    last_message_time: Instant,
}

impl ExternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            bpm_calculator: Arc::new(BpmCalculator::new()),
            shared_state,
            last_message_time: Instant::now(),
        }
    }

    pub fn handle_midi_message(&mut self, msg: MidiMessage) {
        self.last_message_time = Instant::now();

        // Convert MIDI message to clock message
        let clock_msg = match msg {
            MidiMessage::Clock => Some(ClockMessage::Tick),
            MidiMessage::Start => Some(ClockMessage::Start),
            MidiMessage::Stop => Some(ClockMessage::Stop),
            MidiMessage::Continue => Some(ClockMessage::Continue),
            _ => None,
        };

        // Process clock message if applicable
        if let Some(clock_msg) = clock_msg {
            // Handle transport state changes immediately
            match clock_msg {
                ClockMessage::Start => {
                    if let Ok(transport) = self.shared_state.lock() {
                        transport.set_playing(true);
                        info!("External MIDI clock started playback");
                    }
                }
                ClockMessage::Stop => {
                    if let Ok(transport) = self.shared_state.lock() {
                        transport.set_playing(false);
                        info!("External MIDI clock stopped playback");
                    }
                }
                ClockMessage::Continue => {
                    if let Ok(transport) = self.shared_state.lock() {
                        transport.set_playing(true);
                        info!("External MIDI clock resumed playback");
                    }
                }
                _ => {}
            }

            // Update BPM calculator and handle ticks
            if let Some(bpm) = self.bpm_calculator.process_message(clock_msg.clone()) {
                if let Ok(mut transport) = self.shared_state.lock() {
                    transport.set_tempo(bpm);
                    info!("External MIDI clock tempo updated to {} BPM", bpm);
                }

                // Handle tick in a separate lock to minimize lock contention
                if let ClockMessage::Tick = clock_msg {
                    if let Ok(transport) = self.shared_state.lock() {
                        transport.tick();
                    }
                }
            }
        }
    }

    fn check_connection_status(&self) -> bool {
        if self.last_message_time.elapsed() > INACTIVITY_TIMEOUT {
            if let Ok(transport) = self.shared_state.lock() {
                transport.set_playing(false);
            }
            error!(
                "External MIDI clock connection timeout - no messages received for {:?}. Last message was received at {:?}",
                INACTIVITY_TIMEOUT,
                self.last_message_time
            );
            false
        } else {
            true
        }
    }
}

pub fn run_external_clock<T>(engine: T, shared_state: SharedState)
where
    T: MidiEngine + Send + 'static,
{
    let mut clock = ExternalClock::new(shared_state);
    info!("External MIDI clock initialized and waiting for input");

    // Create a channel for the receiver thread
    let (tx, rx) = std::sync::mpsc::channel();
    let engine = std::sync::Arc::new(std::sync::Mutex::new(engine));
    let engine_clone = engine.clone();

    // Spawn a thread to handle receiving MIDI messages
    std::thread::spawn(move || loop {
        let recv_result = engine_clone.lock().unwrap().recv();
        match recv_result {
            Ok(msg) => {
                if tx.send(msg).is_err() {
                    error!("Failed to send MIDI message through channel - receiver dropped");
                    break;
                }
            }
            Err(e) => {
                error!("MIDI engine receive error: {}", e);
                break;
            }
        }
    });

    loop {
        if !clock.check_connection_status() {
            break;
        }

        // Wait for a message with timeout
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(msg) => {
                clock.handle_midi_message(msg);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout is handled by check_connection_status on next loop
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                error!("MIDI message receiver thread disconnected - this could indicate a device disconnection or thread panic");
                break;
            }
        }
    }

    info!("External MIDI clock stopped - shutting down clock thread");
}
