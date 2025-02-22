use crate::clock::ClockSource;
use log::{debug, info};
use midir::{Ignore, MidiInput};
use std::sync::mpsc::Sender;
use std::thread;

pub struct ExternalClock {
    device_name: String,
    tick_tx: Sender<()>,
}

impl ExternalClock {
    pub fn new(device_name: String, tick_tx: Sender<()>) -> Self {
        info!("Creating new ExternalClock with device: {}", device_name);
        ExternalClock {
            device_name,
            tick_tx,
        }
    }
}

impl ClockSource for ExternalClock {
    fn start(&self, tick_callback: Box<dyn Fn() + Send + 'static>) {
        info!("Starting ExternalClock with device: {}", self.device_name);
        let tick_tx = self.tick_tx.clone();
        let device_name = self.device_name.clone();

        // Keep the connection alive in a separate thread.
        thread::spawn(move || {
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
                    port_name.contains(&device_name)
                })
                .expect("External MIDI device not found");

            info!("Found matching MIDI device, attempting connection...");

            let _conn_in = midi_in
                .connect(
                    in_port,
                    "phasorsyncrs-external-conn",
                    move |timestamp, message, _| {
                        if message.first() == Some(&0xF8) {
                            // On MIDI Clock message, invoke tick_callback
                            debug!("Received MIDI Clock message");
                            tick_callback();
                            tick_tx.send(()).unwrap();
                        } else if let Some(&msg_type) = message.first() {
                            debug!(
                                "Received MIDI message type: {:X} at timestamp: {}",
                                msg_type, timestamp
                            );
                        }
                    },
                    (),
                )
                .expect("Failed to connect to external MIDI device");

            info!("Starting MIDI connection maintenance thread");
            loop {
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    }
}
