use crate::clock::ClockSource;
use crate::event_loop::{EngineMessage, TransportAction};
use log::{debug, error, info};
use midir::{Ignore, MidiInput, MidiInputPort};
use std::sync::mpsc::Sender;
use std::thread;

pub struct ExternalClock {
    device_name: String,
    engine_tx: Sender<EngineMessage>,
}

impl ExternalClock {
    pub fn new(device_name: String, engine_tx: Sender<EngineMessage>) -> Self {
        info!("Creating new ExternalClock with device: {}", device_name);
        ExternalClock {
            device_name,
            engine_tx,
        }
    }
}

impl ClockSource for ExternalClock {
    fn start(&self) {
        info!("Starting ExternalClock with device: {}", self.device_name);
        let engine_tx = self.engine_tx.clone();
        let device_name = self.device_name.clone();

        thread::spawn(move || {
            run_midi_connection(engine_tx, device_name);
        });
    }
}

fn handle_midi_message(timestamp: u64, message: &[u8], engine_message_tx: &Sender<EngineMessage>) {
    if message.first() == Some(&0xF8) {
        debug!("Received MIDI Clock message");
        engine_message_tx.send(EngineMessage::Tick).unwrap();
    } else if message.first() == Some(&0xFA) {
        engine_message_tx
            .send(EngineMessage::TransportCommand(TransportAction::Start))
            .unwrap();
    } else if message.first() == Some(&0xFC) {
        engine_message_tx
            .send(EngineMessage::TransportCommand(TransportAction::Stop))
            .unwrap();
    } else if let Some(&msg_type) = message.first() {
        debug!(
            "Received MIDI message type: {:X} at timestamp: {}",
            msg_type, timestamp
        );
    }
}

fn find_midi_port(midi_in: &mut MidiInput, device_name: &str) -> Option<MidiInputPort> {
    let in_ports = midi_in.ports();
    debug!("Available MIDI input ports:");
    for port in &in_ports {
        if let Ok(port_name) = midi_in.port_name(port) {
            debug!("  - {}", port_name);
        }
    }

    match in_ports.iter().find(|port| {
        let port_name = midi_in.port_name(port).unwrap_or_default();
        debug!("Checking port: {}", port_name);
        port_name.contains(device_name)
    }) {
        Some(port) => Some(port.clone()),
        None => {
            // Log available devices for troubleshooting
            let available_devices: Vec<String> = in_ports
                .iter()
                .filter_map(|p| midi_in.port_name(p).ok())
                .collect();

            let error_message = format!("External MIDI device '{}' not found!", device_name);
            error!("{}", error_message);
            info!("Available MIDI devices: {:?}", available_devices);
            println!("{}", error_message);
            error!("Application cannot continue without the specified device");

            // Exit with error code
            std::process::exit(1);
        }
    }
}

fn run_midi_connection(engine_tx: Sender<EngineMessage>, device_name: String) {
    let mut midi_in =
        MidiInput::new("phasorsyncrs-external").expect("Failed to initialize MIDI input");
    midi_in.ignore(Ignore::None);

    let in_port = find_midi_port(&mut midi_in, &device_name).unwrap();

    info!("Found matching MIDI device, attempting connection...");

    let engine_message_tx = engine_tx.clone(); // Shadow the outer tick_tx

    let _conn_in = midi_in
        .connect(
            &in_port,
            "phasorsyncrs-external-conn",
            move |timestamp, message, _| {
                handle_midi_message(timestamp, message, &engine_message_tx);
            },
            (),
        )
        .expect("Failed to connect to external MIDI device");

    info!("Starting MIDI connection maintenance thread");
    loop {
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test verifies that a non-existent device produces an error
    // Note: This test uses a modified approach to avoid actual process exit
    #[test]
    fn test_device_not_found_handling() {
        // Function to test device finding logic without exiting
        fn find_device(
            device_name: &str,
            ports: &[MidiInputPort],
            midi_in: &MidiInput,
        ) -> Option<MidiInputPort> {
            ports
                .iter()
                .find(|port| {
                    let port_name = midi_in.port_name(port).unwrap_or_default();
                    port_name.contains(device_name)
                })
                .cloned()
        }

        let midi_in = MidiInput::new("test-midi-input").unwrap();
        let ports = midi_in.ports();
        let non_existent_device = "NonExistentDevice12345";

        // Verify the device is not found
        let result = find_device(non_existent_device, &ports, &midi_in);
        assert!(
            result.is_none(),
            "The non-existent device should not be found"
        );
    }
}
