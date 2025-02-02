# ADR: Linux-Optimized MIDI Implementation Strategy

**Date:** 2025-02-01  
**Status:** Revised

## Context

PhasorSyncRS requires realtime MIDI I/O with:
- Strict latency guarantees (<2ms)
- Linux-only operation
- Hardware-level control
- Testable interfaces (per ADR01)

## Decision

Implement a Linux-optimized MIDI stack using:

1. **Core Layer**: `midir` (v0.6+)
   - Actively maintained Rust bindings
   - ALSA backend with real-time priorities
   - Port discovery via OS-native APIs

2. **Protocol Layer**: Type-safe abstraction
   - `midi-message` crate for standard message types
   - Custom message validation/parsing
   - Lock-free channels (crossbeam crate)

3. **Testing**: Mockable interface
   ```rust
   #[cfg_attr(feature = "test-mock", mockall::automock)]
   pub trait MidiEngine {
       fn send(&mut self, data: &[u8]) -> Result<(), AlsaError>;
       fn recv(&mut self) -> Result<Vec<u8>, AlsaError>;
   }
   ```

## Implementation Strategy

```rust
// src/midi/rtmidi.rs
pub struct RtMidiEngine {
    input: RtMidiIn,
    output: RtMidiOut,
}

impl MidiEngine for RtMidiEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()> {
        let data = msg.to_midi_bytes();
        self.output.send_message(&data).map_err(Into::into)
    }
}
```

## Performance Optimization

| Technique               | Benefit                      | ADR Alignment        |
|-------------------------|------------------------------|----------------------|
| Lock-free channels      | Cross-platform concurrency   | ADR00 §4            |
| Automatic buffer mgmt   | Zero-copy across OS layers    | ADR00 §4.1         |
| Batch processing        | Reduced syscall overhead     | ADR00 §4.1        |

## Message Abstraction (Consolidated from ADR04)

Implement a higher-level `MidiMessage` enum for type safety:
```rust
pub enum MidiMessage {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    // ... other variants
}
```

### Benefits:
- Type-safe API prevents invalid messages
- Self-documenting message structure
- Simplified testing and mocking
- Automatic serialization/deserialization via `midi-message` crate

## Consequences

- **Pros**:
  - Cross-platform compatibility
  - Modern type-safe API
  - Reduced unsafe code surface
  - Maintains TDD capabilities (ADR01)
  
- **Cons**:
  - Slightly higher abstraction overhead
  - Requires MIDI driver packages on Windows/macOS

## Alternatives Considered

1. **midir**: Cross-platform abstraction adds latency
2. **JACK**: Requires external audio server
3. **Pure C FFI**: Lacks Rust safety guarantees