use log::{debug, error, info};
use midir::{MidiOutput as MidirOutput, MidiOutputConnection as MidirOutputConnection};
use std::collections::HashMap;
use std::error::Error;

pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
        duration_ticks: u64,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    AllNotesOff {
        channel: u8,
    },
}

pub trait MidiOutput {
    fn send(&mut self, message: MidiMessage) -> Result<(), Box<dyn Error>>;
    fn process_tick_events(&mut self, current_tick: u64, new_events: Vec<MidiMessage>);
}

pub struct MidiOutputManager {
    connection: Option<MidirOutputConnection>,
    // New field: a mapping from target tick to scheduled MIDI messages.
    scheduled_notes: HashMap<u64, Vec<MidiMessage>>,
}

impl Default for MidiOutputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiOutputManager {
    pub fn new() -> Self {
        MidiOutputManager {
            connection: None,
            scheduled_notes: HashMap::new(),
        }
    }

    pub fn connect_to_first_available(&mut self) -> Result<(), Box<dyn Error>> {
        let midi_out = MidirOutput::new("phasorsyncrs-output")?;

        // Get available output ports
        let out_ports = midi_out.ports();
        if out_ports.is_empty() {
            return Err("No MIDI output ports available".into());
        }

        // Just use the first available port for simplicity
        let port = &out_ports[0];
        let port_name = midi_out.port_name(port)?;

        info!("Connecting to MIDI output port: {}", port_name);
        let connection = midi_out.connect(port, "phasorsyncrs-output-conn")?;
        self.connection = Some(connection);
        Ok(())
    }

    pub fn connect_to_device(&mut self, device_name: &str) -> Result<(), Box<dyn Error>> {
        let midi_out = MidirOutput::new("phasorsyncrs-output")?;

        let out_ports = midi_out.ports();
        let available_ports: Vec<String> = out_ports
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect();

        info!("Available MIDI output ports: {:?}", available_ports);

        let port = out_ports
            .iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .unwrap_or_default()
                    .contains(device_name)
            })
            .ok_or_else(|| {
                error!("MIDI output device '{}' not found", device_name);
                info!("Available devices: {:?}", available_ports);
                "MIDI output device not found"
            })?;

        let port_name = midi_out.port_name(port)?;
        info!("Connecting to MIDI output port: {}", port_name);

        let connection = midi_out.connect(port, "phasorsyncrs-output-conn")?;
        self.connection = Some(connection);
        Ok(())
    }

    // Process any scheduled events for the current tick
    fn process_scheduled_events(&mut self, current_tick: u64) {
        if let Some(scheduled_events) = self.scheduled_notes.remove(&current_tick) {
            for event in scheduled_events {
                if let Err(e) = self.send(event) {
                    error!("Failed to send scheduled MIDI event: {}", e);
                }
            }
        }
    }

    // Process new events and schedule any necessary Note Off events
    fn process_new_events(&mut self, current_tick: u64, new_events: Vec<MidiMessage>) {
        for event in new_events {
            match event {
                MidiMessage::NoteOn {
                    channel,
                    note,
                    velocity,
                    duration_ticks,
                } => {
                    // Send NoteOn immediately
                    if let Err(e) = self.send(MidiMessage::NoteOn {
                        channel,
                        note,
                        velocity,
                        duration_ticks: 0, // Not needed when sending
                    }) {
                        error!("Failed to send NoteOn: {}", e);
                    }

                    // Schedule the corresponding NoteOff
                    let target_tick = current_tick + duration_ticks;
                    self.scheduled_notes
                        .entry(target_tick)
                        .or_default()
                        .push(MidiMessage::NoteOff { channel, note });
                }
                _ => {
                    if let Err(e) = self.send(event) {
                        error!("Failed to send MIDI event: {}", e);
                    }
                }
            }
        }
    }
}

impl MidiOutput for MidiOutputManager {
    fn send(&mut self, message: MidiMessage) -> Result<(), Box<dyn Error>> {
        let conn = self
            .connection
            .as_mut()
            .ok_or("MIDI output not connected")?;

        match message {
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
                duration_ticks: _, // Ignore duration_ticks when sending
            } => {
                let msg = [0x90 | (channel & 0x0F), note, velocity];
                debug!(
                    "Sending MIDI Note On: ch={}, note={}, vel={}",
                    channel, note, velocity
                );
                conn.send(&msg)?;
            }
            MidiMessage::NoteOff { channel, note } => {
                let msg = [0x80 | (channel & 0x0F), note, 0];
                debug!("Sending MIDI Note Off: ch={}, note={}", channel, note);
                conn.send(&msg)?;
            }
            MidiMessage::AllNotesOff { channel } => {
                let msg = [0xB0 | (channel & 0x0F), 123, 0];
                debug!("Sending All Notes Off: ch={}", channel);
                conn.send(&msg)?;
            }
        }
        Ok(())
    }

    // Process MIDI events for the current tick
    fn process_tick_events(&mut self, current_tick: u64, new_events: Vec<MidiMessage>) {
        // First, process any scheduled events for the current tick
        self.process_scheduled_events(current_tick);

        // Then process any new events
        self.process_new_events(current_tick, new_events);
    }
}
