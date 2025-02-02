use crate::midi::{MidiEngine, MidiMessage, Result};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::mpsc::{channel, Receiver};

pub struct MidirEngine {
    #[allow(dead_code)]
    input: Option<MidiInputConnection<()>>,
    output: Option<MidiOutputConnection>,
    rx: Option<Receiver<Vec<u8>>>,
}

impl MidirEngine {
    pub fn new(device_name: Option<String>) -> Result<Self> {
        let (input, rx) = if let Some(name) = &device_name {
            // Create and store midi_in instance
            let mut midi_in = MidiInput::new("phasorsyncrs-in")?;
            midi_in.ignore(Ignore::None);

            // Find input port by name
            let in_ports = midi_in.ports();
            let in_port = in_ports
                .iter()
                .find(|p| midi_in.port_name(p).unwrap_or_default().contains(name))
                .ok_or("Input device not found")?;

            let (tx, rx) = channel();
            let input = midi_in.connect(
                in_port,
                "phasorsyncrs-input",
                move |_stamp, message, _| {
                    let _ = tx.send(message.to_vec());
                },
                (),
            )?;
            (Some(input), Some(rx))
        } else {
            (None, None)
        };

        let output = if let Some(name) = device_name {
            // Create and store midi_out instance
            let midi_out = MidiOutput::new("phasorsyncrs-out")?;

            // Find output port by name
            let out_ports = midi_out.ports();
            let out_port = out_ports
                .iter()
                .find(|p| midi_out.port_name(p).unwrap_or_default().contains(&name))
                .ok_or("Output device not found")?;
            Some(midi_out.connect(out_port, "phasorsyncrs-output")?)
        } else {
            None
        };

        Ok(MidirEngine { input, output, rx })
    }

    fn parse_midi_message(data: &[u8]) -> Option<MidiMessage> {
        if data.is_empty() {
            return None;
        }

        match data[0] & 0xF0 {
            0x90 if data.len() >= 3 => Some(MidiMessage::NoteOn {
                channel: data[0] & 0x0F,
                note: data[1],
                velocity: data[2],
            }),
            0x80 if data.len() >= 3 => Some(MidiMessage::NoteOff {
                channel: data[0] & 0x0F,
                note: data[1],
                velocity: data[2],
            }),
            0xB0 if data.len() >= 3 => Some(MidiMessage::ControlChange {
                channel: data[0] & 0x0F,
                controller: data[1],
                value: data[2],
            }),
            0xC0 if data.len() >= 2 => Some(MidiMessage::ProgramChange {
                channel: data[0] & 0x0F,
                program: data[1],
            }),
            0xF8 => Some(MidiMessage::Clock),
            0xFA => Some(MidiMessage::Start),
            0xFC => Some(MidiMessage::Stop),
            0xFB => Some(MidiMessage::Continue),
            _ => None,
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
}

impl MidiEngine for MidirEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()> {
        if let Some(output) = &mut self.output {
            let bytes = Self::message_to_bytes(&msg);
            output.send(&bytes)?;
        }
        Ok(())
    }

    fn recv(&mut self) -> Result<MidiMessage> {
        if let Some(rx) = &self.rx {
            let data = rx.recv()?;
            if let Some(msg) = Self::parse_midi_message(&data) {
                Ok(msg)
            } else {
                Err("Invalid MIDI message".into())
            }
        } else {
            Err("No input connection".into())
        }
    }

    fn list_devices(&self) -> Vec<String> {
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
