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

Implement a Linux-native MIDI stack using:

1. **Core Layer**: `alsa-rs` (v0.7+)
   - Direct ALSA API access via safe Rust bindings
   - Zero-copy buffer management
   - Real-time thread priority configuration

2. **Protocol Layer**: Custom implementation
   - `bytes` crate for efficient buffer handling
   - `nom` for message validation/parsing
   - Lock-free ring buffers (heapless crate)

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
// src/midi/alsa.rs
pub struct AlsaMidi {
    seq: alsa::Seq,
    in_port: i32,
    out_port: i32,
}

impl MidiEngine for AlsaMidi {
    fn send(&mut self, data: &[u8]) -> Result<(), AlsaError> {
        let ev = alsa::seq::Event::new();
        // ALSA-specific event packing
        self.seq.event_output(ev)?;
        self.seq.drain_output()?;
        Ok(())
    }
}
```

## Performance Optimization

| Technique               | Benefit                      | ADR Alignment        |
|-------------------------|------------------------------|----------------------|
| Realtime thread prio    | Guaranteed scheduling        | ADR00 §4            |
| Preallocated buffers    | No heap allocation in hot path | ADR00 §4.1         |
| Lock-free structures    | Concurrent access without locks | ADR00 §4.1        |

## Consequences

- **Pros**:
  - Native Linux performance (μs-level latency)
  - Direct hardware access without abstraction layers
  - Mockable interface enables full TDD (ADR01)
  
- **Cons**:
  - Linux-specific implementation
  - Requires ALSA development headers
  - Increased unsafe code surface

## Alternatives Considered

1. **midir**: Cross-platform abstraction adds latency
2. **JACK**: Requires external audio server
3. **Pure C FFI**: Lacks Rust safety guarantees