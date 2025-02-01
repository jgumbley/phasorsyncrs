# PhasorSyncRS

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-orange)](LICENSE)

A real-time MIDI sequencer engine written in Rust, focusing on immutability and temporal precision.

## Key Features
- **Event-driven architecture** with precise timing control
- **Immutable data structures** for thread-safe pattern manipulation
- **Modular design** supporting multiple MIDI transport backends
- **Extensible pattern mutation** system with DSL support

## Getting Started

### Prerequisites
- Rust 1.75+ (using [rustup](https://rustup.rs/))
- MIDI-capable hardware or virtual port

```bash
cargo add phasorsyncrs --features rtmidi
```

### Basic Usage

```rust
use phasorsyncrs::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = SequencerEngine::new()
        .with_transport(RtMidiTransport::default())?
        .with_bpm(120);
        
    engine.load_pattern(Pattern::acid_line())?;
    engine.run()?;
    
    Ok(())
}
```

## Core Concepts

### Temporal Architecture
- **Tick-driven scheduler** with sub-millisecond resolution
- **Atomic timing** synchronization using hardware clocks
- **Event buffering** with priority queue implementation

### Pattern System
- **Immutable event sequences** using Arc-based sharing
- **Markov mutation** engine with runtime-configurable rules
- **Polyphonic voice management** with channel allocation

## Development Status

⚠️ Early Alpha - Core sequencing functionality implemented. API subject to change.

## Roadmap
- [x] MIDI 1.0 core implementation
- [ ] JACK audio backend support
- [ ] WebMIDI integration
- [ ] Mutation rule DSL

## License
MIT License - See [LICENSE](LICENSE) for details
