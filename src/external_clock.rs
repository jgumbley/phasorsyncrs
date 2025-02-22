use crate::clock::ClockSource;
use log::{debug, info};
use midir::{Ignore, MidiInput};
use std::thread;

pub struct ExternalClock {
    device_name: String,
}

impl ExternalClock {
    pub fn new(device_name: String) -> Self {
        info!("Creating new ExternalClock with device: {}", device_name);
        ExternalClock { device_name }
    }
}

impl ClockSource for ExternalClock {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        info!("Starting ExternalClock with device: {}", self.device_name);

        let mut midi_in =
            MidiInput::new("phasorsyncrs-external").expect("Failed to initialize MIDI input");
        midi_in.ignore(Ignore::None);

        // Find the input port by name
        let in_ports = midi_in.ports();
        debug!("Available MIDI input ports:");
        for port in &in_ports {
            if let Ok(port_name) = midi_in.port_name(port) {
                debug!("  - {}", port_name);
            }
        }

        let in_port = in_ports
            .iter()
            .find(|port| {
                let port_name = midi_in.port_name(port).unwrap_or_default();
                debug!("Checking port: {}", port_name);
                port_name.contains(&self.device_name)
            })
            .expect("External MIDI device not found");

        info!("Found matching MIDI device, attempting connection...");

        // Connect and spawn a thread to listen for messages
        let _conn_in = midi_in
            .connect(
                in_port,
                "phasorsyncrs-external-conn",
                move |timestamp, message, _| {
                    if let Some(&msg_type) = message.first() {
                        debug!(
                            "Received MIDI message type: {:X} at timestamp: {}",
                            msg_type, timestamp
                        );
                        if msg_type == 0xF8 {
                            // On MIDI Clock message, invoke tick_callback
                            debug!("Received MIDI Clock message");
                            tick_callback();
                        }
                    }
                },
                (),
            )
            .expect("Failed to connect to external MIDI device");

        info!("Successfully connected to external MIDI device");

        // Keep the connection alive in a separate thread.
        thread::spawn(|| {
            info!("Starting MIDI connection maintenance thread");
            loop {
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    }
}
