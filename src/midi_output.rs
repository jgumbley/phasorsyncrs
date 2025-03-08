use log::{debug, error, info};
use midir::{MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

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

pub struct MidiOutputManager {
    connection: Option<MidiOutputConnection>,
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
        let midi_out = MidiOutput::new("phasorsyncrs-output")?;

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
        let midi_out = MidiOutput::new("phasorsyncrs-output")?;

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

    pub fn send(&mut self, message: MidiMessage) -> Result<(), Box<dyn Error>> {
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
    pub fn process_tick_events(&mut self, current_tick: u64, new_events: Vec<MidiMessage>) {
        // Flush scheduled events due at current_tick
        self.process_scheduled_events(current_tick);

        // Process new events
        for event in new_events {
            self.process_single_event(event, current_tick);
        }
    }

    // Process scheduled events for the current tick
    fn process_scheduled_events(&mut self, current_tick: u64) {
        if let Some(events) = self.scheduled_notes.remove(&current_tick) {
            for event in events {
                if let Err(e) = self.send(event) {
                    error!("Failed to send scheduled MIDI event: {}", e);
                }
            }
        }
    }

    // Process a single MIDI event
    fn process_single_event(&mut self, event: MidiMessage, current_tick: u64) {
        match &event {
            MidiMessage::NoteOn { .. } => {
                self.process_note_on(&event, current_tick);
            }
            _ => {
                if let Err(e) = self.send(event) {
                    error!("Failed to send MIDI event: {}", e);
                }
            }
        }
    }

    // Process a note-on event
    fn process_note_on(&mut self, note_event: &MidiMessage, current_tick: u64) {
        if let MidiMessage::NoteOn {
            channel,
            note,
            velocity,
            duration_ticks,
        } = note_event
        {
            // Send NoteOn immediately
            if let Err(e) = self.send(MidiMessage::NoteOn {
                channel: *channel,
                note: *note,
                velocity: *velocity,
                duration_ticks: 0, // Not needed when sending
            }) {
                error!("Failed to send NoteOn: {}", e);
            }

            // Schedule the corresponding NoteOff
            let target_tick = current_tick + duration_ticks;
            self.scheduled_notes
                .entry(target_tick)
                .or_default()
                .push(MidiMessage::NoteOff {
                    channel: *channel,
                    note: *note,
                });
        }
    }

    // Utility method to list all available MIDI output ports
    pub fn list_available_ports() -> Result<Vec<String>, Box<dyn Error>> {
        let midi_out = MidiOutput::new("phasorsyncrs-port-lister")?;
        let ports = midi_out.ports();
        let port_names = ports
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect();
        Ok(port_names)
    }
}

// Helper function to connect to MIDI output
fn connect_midi_output(
    output_manager: &mut MidiOutputManager,
    device_name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // List available ports for debugging
    match MidiOutputManager::list_available_ports() {
        Ok(ports) => info!("Available MIDI output ports: {:?}", ports),
        Err(e) => error!("Failed to list MIDI ports: {}", e),
    }

    match device_name {
        Some(name) => {
            info!("Attempting to connect to MIDI device: {}", name);
            output_manager.connect_to_device(&name)
        }
        None => {
            info!("No device specified, connecting to first available MIDI output");
            output_manager.connect_to_first_available()
        }
    }
}

// Helper function to process MIDI messages
fn process_midi_messages(output_manager: &mut MidiOutputManager, rx: Receiver<MidiMessage>) {
    info!("MIDI output thread started and connected successfully");

    // Process incoming messages
    while let Ok(message) = rx.recv() {
        if let Err(e) = output_manager.send(message) {
            error!("Failed to send MIDI message: {}", e);
        }
    }

    info!("MIDI output thread stopping");
}

pub fn run_midi_output_thread(rx: Receiver<MidiMessage>, device_name: Option<String>) {
    thread::spawn(move || {
        let mut output_manager = MidiOutputManager::new();

        // Connect to MIDI output
        if let Err(e) = connect_midi_output(&mut output_manager, device_name) {
            error!("Failed to connect MIDI output: {}", e);
            return;
        }

        // Process messages
        process_midi_messages(&mut output_manager, rx);
    });
}

// For testing purposes, this function can trigger a simple MIDI test
// when enabled through config
pub fn send_test_note(tx: &Sender<MidiMessage>) -> Result<(), Box<dyn Error>> {
    info!("Sending test note (Middle C)");

    // Send note on
    tx.send(MidiMessage::NoteOn {
        channel: 1,
        note: 60, // Middle C
        velocity: 100,
        duration_ticks: 0, // No duration for test note, we'll handle it manually
    })?;

    // Create a thread that will send the note off after 500ms
    // Note: This is only for testing! In production, we'll use the scheduler
    let tx_clone = tx.clone();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(500));
        let _ = tx_clone.send(MidiMessage::NoteOff {
            channel: 1,
            note: 60,
        });
        info!("Test note released");
    });

    Ok(())
}

// Helper function to list available MIDI ports
fn list_available_midi_ports(midi_out: &MidiOutput) -> Result<(), Box<dyn Error>> {
    let ports = midi_out.ports();
    if ports.is_empty() {
        return Err("No MIDI output ports available".into());
    }

    info!("Available MIDI output ports:");
    for (i, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_out.port_name(port) {
            info!("  [{}] '{}'", i, name);
        }
    }
    Ok(())
}


