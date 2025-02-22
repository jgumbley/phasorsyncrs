# ADR: MIDI Implementation Strategy

**Date:** 2025-02-01
**Status:** Revised

## Context

PhasorSyncRS requires realtime MIDI I/O with:
- Strict latency guarantees (<2ms)
- Hardware-level control
- Testable interfaces (per ADR01)

## Decision

Implement a MIDI stack using:

1. **Core Layer**: `midir` (v0.6+)
   - Actively maintained Rust bindings
   - Port discovery via OS-native APIs


## Implementation Strategy

The application uses the `midir` crate for MIDI I/O. The `ExternalClock` struct in `src/external_clock.rs` handles the MIDI input.

## Performance Optimization

| Technique               | Benefit                      | ADR Alignment        |
|-------------------------|------------------------------|----------------------|
| Lock-free channels      | Cross-platform concurrency   | ADR00 §4            |
| Automatic buffer mgmt   | Zero-copy across OS layers    | ADR00 §4.1         |
| Batch processing        | Reduced syscall overhead     | ADR00 §4.1        |

## Consequences

- **Pros**:
  - Actively maintained Rust bindings
  - Maintains TDD capabilities (ADR01)
  
- **Cons**:
  - Slightly higher abstraction overhead

## Alternatives Considered

1. **midir**: Cross-platform abstraction adds latency
2. **JACK**: Requires external audio server
3. **Pure C FFI**: Lacks Rust safety guarantees