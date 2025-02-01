# ADR04: MIDI Message Type Abstraction

**Date:** 2025-02-01  
**Status:** Accepted  
**Supersedes:** Part of ADR02 (MIDI interface definition)

## Context

ADR02 defined the MidiEngine trait with raw byte array interfaces:

```rust
fn send(&mut self, data: &[u8]) -> Result<(), AlsaError>;
fn recv(&mut self) -> Result<Vec<u8>, AlsaError>;
```

This low-level interface requires consumers to handle raw MIDI bytes, which:
- Is error-prone
- Lacks type safety
- Makes code harder to read and maintain
- Complicates testing

## Decision

Implement a higher-level MidiMessage enum that represents MIDI messages as strongly-typed values:

```rust
pub enum MidiMessage {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    Clock,
    Start,
    Stop,
    Continue,
}

pub trait MidiEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()>;
    fn recv(&mut self) -> Result<MidiMessage>;
    fn list_devices(&self) -> Vec<String>;
}
```

## Rationale

1. **Type Safety**: The enum makes it impossible to construct invalid MIDI messages
2. **Readability**: Message intent is clear from the variant name and fields
3. **Testability**: Easier to write tests with meaningful message types
4. **Error Handling**: Better error messages when invalid operations are attempted
5. **Documentation**: Self-documenting code through type names

## Implementation Strategy

1. **Protocol Layer**:
   - Implement From/TryFrom traits for converting between MidiMessage and raw bytes
   - Use nom for parsing incoming MIDI bytes into MidiMessage values
   - Optimize message serialization for zero-allocation

2. **Testing**:
   - Mock implementations can work with high-level messages
   - Test cases are more readable and maintainable
   - Property-based testing for message conversion

## Consequences

### Positive

- More ergonomic API for consumers
- Reduced chance of runtime errors
- Better IDE support through type information
- Clearer test cases
- Self-documenting code

### Negative

- Additional conversion overhead
- Slightly increased memory usage
- Not all MIDI message types are covered (can be extended as needed)

## References

- ADR02: Linux-Optimized MIDI Implementation Strategy
- [MIDI 1.0 Specification](https://www.midi.org/specifications)