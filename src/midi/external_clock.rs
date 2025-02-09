use super::clock::{core::ClockCore, ClockMessage};
use crate::midi::{MidiClock, MidiEngine, MidiMessage};
use crate::SharedState;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ExternalClock {
    core: Arc<Mutex<ClockCore>>,
    last_message_time: Instant,
}

impl ExternalClock {
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            core: ClockCore::new(shared_state), // This already returns Arc<Mutex<ClockCore>>
            last_message_time: Instant::now(),
        }
    }

    pub fn handle_midi_message(&mut self, msg: MidiMessage) {
        self.last_message_time = Instant::now();

        // Convert MIDI message to clock message
        let clock_msg = self.convert_midi_to_clock_message(msg);

        // Process clock message if applicable
        if let Some(clock_msg) = clock_msg {
            if let Ok(mut core) = self.core.lock() {
                core.process_message(clock_msg);
            }
        }
    }

    fn convert_midi_to_clock_message(&self, msg: MidiMessage) -> Option<ClockMessage> {
        match msg {
            MidiMessage::Clock => Some(ClockMessage::Tick),
            MidiMessage::Start => Some(ClockMessage::Start),
            MidiMessage::Stop => Some(ClockMessage::Stop),
            MidiMessage::Continue => Some(ClockMessage::Continue),
            _ => None,
        }
    }

    fn check_connection_status(&self) -> bool {
        if self.last_message_time.elapsed() > INACTIVITY_TIMEOUT {
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

impl MidiClock for ExternalClock {
    fn start(&mut self) {
        info!("External MIDI clock started");
    }

    fn stop(&mut self) {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(ClockMessage::Stop);
        }
        info!("External MIDI clock stopped");
    }

    fn is_playing(&self) -> bool {
        if let Ok(core) = self.core.lock() {
            core.is_playing()
        } else {
            false
        }
    }

    fn current_bpm(&self) -> Option<f64> {
        if let Ok(core) = self.core.lock() {
            core.current_bpm()
        } else {
            None
        }
    }

    fn handle_message(&mut self, msg: ClockMessage) {
        if let Ok(mut core) = self.core.lock() {
            core.process_message(msg);
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
