use std::error::Error;
use std::fmt;

/// Custom error type for MIDI operations
#[derive(Debug)]
pub enum MidiError {
    /// Error when sending a MIDI message
    SendError(String),
    /// Error when receiving a MIDI message
    RecvError(String),
    /// Error when connecting to a MIDI device
    ConnectionError(String),
}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MidiError::SendError(msg) => write!(f, "MIDI send error: {}", msg),
            MidiError::RecvError(msg) => write!(f, "MIDI receive error: {}", msg),
            MidiError::ConnectionError(msg) => write!(f, "MIDI connection error: {}", msg),
        }
    }
}

impl Error for MidiError {}

/// Represents a MIDI message that can be sent or received
#[derive(Debug, Clone, PartialEq)]
pub enum MidiMessage {
    /// Note On message with note number and velocity
    NoteOn { channel: u8, note: u8, velocity: u8 },
    /// Note Off message with note number and velocity
    NoteOff { channel: u8, note: u8, velocity: u8 },
    /// Control Change message with controller number and value
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Program Change message with program number
    ProgramChange { channel: u8, program: u8 },
    /// MIDI Clock timing message
    Clock,
    /// MIDI Start message
    Start,
    /// MIDI Stop message
    Stop,
    /// MIDI Continue message
    Continue,
}

/// Result type for MIDI operations
pub type Result<T> = std::result::Result<T, MidiError>;

/// Trait defining the interface for MIDI engine implementations
pub trait MidiEngine: Send {
    /// Sends a MIDI message to the device
    fn send(&mut self, msg: MidiMessage) -> Result<()>;

    /// Receives a MIDI message from the device
    fn recv(&self) -> Result<MidiMessage>;
}
