# PhasorSyncRS

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org/)

[![License](https://img.shields.io/badge/license-MIT-orange)](LICENSE)

A real-time MIDI sequencer engine written in Rust, focusing on immutability and temporal precision.

## Key Features

- **Event-driven architecture** with precise timing control

- **Immutable data structures** for thread-safe pattern manipulation

- **Modular design** supporting multiple MIDI transport backends

- **Extensible pattern mutation** system with DSL support

# PhasorSyncRS Product Vision

## Core Objective

Deliver a foundational Rust implementation enabling bidirectional MIDI communication with extensible architecture to support future speculative features including:

- Real-time music theory analysis

- Neurofeedback integration

- Non-linear time representation

- Quantum computing interfaces

## Architectural Principles

1. **Modular Design**

- Decoupled MIDI I/O layer

- Plugin architecture for processing modules

- Event-driven core system

2. **Extensibility First**

- Clean separation between stable interfaces and experimental implementations

- Semantic versioning from initial release

- Comprehensive API documentation

3. **Cross-Domain Foundation**

- Temporal representation system supporting both linear and non-linear models

- Abstract music theory constructs independent of Western notation

- Hardware abstraction layer for MIDI devices

## First Release Requirements

- ✅ Full MIDI 1.0 spec compliance

- ✅ Round-trip verification with 3+ device types

- ✅ Benchmark: <2ms latency on Raspberry Pi 4

- ✅ Extensible event pipeline architecture

- ✅ CI/CD foundation with hardware-in-loop testing

## Dependencies

- **Rustup (for managing Rust toolchain)**

- **Cargo (Rust's package manager and build tool)**

- **Rustfmt (for code formatting)**

- **Clippy (for linter checks)**

To install these tools, follow the instructions at [official Rustup installation guide](https://rustup.rs/).

After installing rustup, you can install the necessary components by running:

rustup component add rustfmt clippy

This will ensure that rustfmt and clippy are available for use.

Then, to initialize the project, run:

cargo init

And to use the Makefile, you can run commands like:

make build

make test

make check

make fmt

make clippy

make doc

## Getting Started

### Prerequisites

- Rust 1.75+ (using [rustup](https://rustup.rs/))

- MIDI-capable hardware or virtual port

### Basic Usage

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

## License

MIT License - See [LICENSE](LICENSE) for details
