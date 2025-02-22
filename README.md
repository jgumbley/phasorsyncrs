# PhasorSyncRS

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-orange)](LICENSE)

Real-time MIDI sequencer engine with external clock synchronization and transport state management.

## Key Features ▶️

- **External MIDI Clock Sync** - Slave mode synchronization (src/midi/external_clock.rs)
- **Transport State Management** - Play/stop/position tracking (src/transport/mod.rs)
- **Scheduler Core** - Tick-driven event queue (src/scheduler.rs)
- **Scheduler Core** - Tick-driven event queue (src/scheduler.rs)
- **CLI Interface** - Interactive transport control (src/cli/mod.rs)

## Quick Start 🚀

```bash
# Clone and build
git clone https://github.com/jgumbley/phasorsyncrs
cd phasorsyncrs
make run
```

## Code Structure 🗂️

```
src/
├── midi/              # MIDI I/O implementations
│   ├── external_clock.rs - Clock synchronization logic
│   └── mock_engine.rs    - Test mock implementation
├── transport/         # Transport state machine
│   └── mod.rs         - Play/stop/position tracking
├── ui/                # Status display interfaces
│   └── mod.rs         - Transport visualization
└── scheduler.rs       - Core timing engine

tests/
└── midi_tests.rs      - MIDI I/O validation tests
```

## Development Flow ⚙️

```bash
# Test with mock MIDI (no hardware required)
make run

# Run bound onto ext device (config in makefile)
make run-oxi
```

## Architectural Guidance 🏛️

Key design decisions documented in ADRs:

- [ADR02: MIDI Library Selection](docs/adr/adr02_midi_library_selection.md)
- [ADR03: Concurrency Model](docs/adr/adr03_structure_concurrency_and_instantiation.md)

## License 📄

MIT - See [LICENSE](LICENSE) for details