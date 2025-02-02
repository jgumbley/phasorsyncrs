use crate::midi::{MidiEngine, MidiError, MidiMessage, Result};
use log::{debug, warn};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::sync::{Arc, Mutex};

pub struct MidirEngine {
    #[allow(dead_code)]
    input: Option<MidiInputConnection<()>>,
    output: Option<MidiOutputConnection>,
    rx: Option<Arc<Mutex<Receiver<Vec<u8>>>>>,
}

impl MidirEngine {
    pub fn new(device_name: Option<String>) -> Result<Self> {
        let (input, rx) = if let Some(name) = &device_name {
            // Create and store midi_in instance
            let mut midi_in = MidiInput::new("phasorsyncrs-in")
                .map_err(|e| MidiError::ConnectionError(e.to_string()))?;
            midi_in.ignore(Ignore::None);

            // Find input port by name
            let in_ports = midi_in.ports();
            let in_port = in_ports
                .iter()
                .find(|p| midi_in.port_name(p).unwrap_or_default().contains(name))
                .ok_or_else(|| MidiError::ConnectionError("Input device not found".to_string()))?;

            let (tx, rx) = channel();
            let input = midi_in
                .connect(
                    in_port,
                    "phasorsyncrs-input",
                    move |_stamp, message, _| {
                        let _ = tx.send(message.to_vec());
                    },
                    (),
                )
                .map_err(|e| MidiError::ConnectionError(e.to_string()))?;
            (Some(input), Some(Arc::new(Mutex::new(rx))))
        } else {
            (None, None)
        };

        let output = if let Some(name) = device_name {
            // Create and store midi_out instance
            let midi_out = MidiOutput::new("phasorsyncrs-out")
                .map_err(|e| MidiError::ConnectionError(e.to_string()))?;

            // Find output port by name
            let out_ports = midi_out.ports();
            let out_port = out_ports
                .iter()
                .find(|p| midi_out.port_name(p).unwrap_or_default().contains(&name))
                .ok_or_else(|| MidiError::ConnectionError("Output device not found".to_string()))?;
            Some(
                midi_out
                    .connect(out_port, "phasorsyncrs-output")
                    .map_err(|e| MidiError::ConnectionError(e.to_string()))?,
            )
        } else {
            None
        };

        Ok(MidirEngine { input, output, rx })
    }

    pub fn parse_midi_message(data: &[u8]) -> Option<MidiMessage> {
        if data.is_empty() {
            debug!("Received empty MIDI message");
            return None;
        }

        debug!("Parsing MIDI message bytes: {:02X?}", data);
        let first_byte = data[0];

        // Handle System Common and System Realtime messages (0xF0-0xFF)
        if first_byte >= 0xF0 {
            match first_byte {
                0xF8 => {
                    debug!("MIDI Clock");
                    return Some(MidiMessage::Clock);
                }
                0xFA => {
                    debug!("MIDI Start");
                    return Some(MidiMessage::Start);
                }
                0xFC => {
                    debug!("MIDI Stop");
                    return Some(MidiMessage::Stop);
                }
                0xFB => {
                    debug!("MIDI Continue");
                    return Some(MidiMessage::Continue);
                }
                _ => {
                    debug!("Unhandled System message: {:02X}", first_byte);
                    return None;
                }
            }
        }

        // Handle Channel Voice Messages
        let status_byte = first_byte & 0xF0;
        let channel = first_byte & 0x0F;

        match status_byte {
            0x90 if data.len() >= 3 => {
                debug!(
                    "Note On - channel: {}, note: {}, velocity: {}",
                    channel, data[1], data[2]
                );
                Some(MidiMessage::NoteOn {
                    channel,
                    note: data[1],
                    velocity: data[2],
                })
            }
            0x80 if data.len() >= 3 => {
                debug!(
                    "Note Off - channel: {}, note: {}, velocity: {}",
                    channel, data[1], data[2]
                );
                Some(MidiMessage::NoteOff {
                    channel,
                    note: data[1],
                    velocity: data[2],
                })
            }
            0xB0 if data.len() >= 3 => {
                debug!(
                    "Control Change - channel: {}, controller: {}, value: {}",
                    channel, data[1], data[2]
                );
                Some(MidiMessage::ControlChange {
                    channel,
                    controller: data[1],
                    value: data[2],
                })
            }
            0xC0 if data.len() >= 2 => {
                debug!(
                    "Program Change - channel: {}, program: {}",
                    channel, data[1]
                );
                Some(MidiMessage::ProgramChange {
                    channel,
                    program: data[1],
                })
            }
            _ => {
                warn!("Unrecognized MIDI status byte: {:02X}", status_byte);
                if data.len() < 3 {
                    warn!(
                        "Message too short: expected at least 3 bytes for most messages, got {}",
                        data.len()
                    );
                }
                None
            }
        }
    }

    fn message_to_bytes(msg: &MidiMessage) -> Vec<u8> {
        match msg {
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => vec![0x90 | (channel & 0x0F), *note, *velocity],
            MidiMessage::NoteOff {
                channel,
                note,
                velocity,
            } => vec![0x80 | (channel & 0x0F), *note, *velocity],
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            } => vec![0xB0 | (channel & 0x0F), *controller, *value],
            MidiMessage::ProgramChange { channel, program } => {
                vec![0xC0 | (channel & 0x0F), *program]
            }
            MidiMessage::Clock => vec![0xF8],
            MidiMessage::Start => vec![0xFA],
            MidiMessage::Stop => vec![0xFC],
            MidiMessage::Continue => vec![0xFB],
        }
    }

    /// Lists available MIDI devices
    pub fn list_devices() -> Vec<String> {
        let mut devices = Vec::new();

        if let Ok(midi_in) = MidiInput::new("phasorsyncrs-list") {
            let ports = midi_in.ports();
            for port in ports {
                if let Ok(name) = midi_in.port_name(&port) {
                    devices.push(name);
                }
            }
        }

        devices
    }
}

impl MidiEngine for MidirEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()> {
        if let Some(output) = &mut self.output {
            let bytes = Self::message_to_bytes(&msg);
            debug!("Sending MIDI message: {:02X?}", bytes);
            output
                .send(&bytes)
                .map_err(|e| MidiError::SendError(e.to_string()))?;
        }
        Ok(())
    }

    fn recv(&self) -> Result<MidiMessage> {
        if let Some(rx) = &self.rx {
            let rx_guard = rx.lock().map_err(|e| MidiError::RecvError(e.to_string()))?;
            debug!("Waiting for MIDI message...");
            let data = rx_guard
                .recv()
                .map_err(|e: RecvError| MidiError::RecvError(e.to_string()))?;
            debug!("Received raw MIDI data: {:02X?}", data);
            if let Some(msg) = Self::parse_midi_message(&data) {
                debug!("Successfully parsed MIDI message: {:?}", msg);
                Ok(msg)
            } else {
                warn!("Failed to parse MIDI message from data: {:02X?}", data);
                Err(MidiError::RecvError("Invalid MIDI message".to_string()))
            }
        } else {
            Err(MidiError::RecvError("No input connection".to_string()))
        }
    }
}
