# PhasorSyncRS

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPLv3-orange)](LICENSE)

Real-time MIDI sequencer engine with external clock synchronization and transport state management.

## Key Features ▶️

- **External MIDI Clock Sync** - Slave mode synchronization (src/external_clock.rs)
- **Transport State Management** - Play/stop/position tracking (src/state.rs)
- **Scheduler Core** - Tick-driven event queue (src/event_loop.rs)
- **CLI Interface** - Interactive transport control (src/ui.rs)

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
│   └── external_clock.rs - Clock synchronization logic
├── state.rs           # Transport state machine
├── ui.rs              # Status display interfaces
└── event_loop.rs      - Core timing engine

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

- [ADR00: Guiding Principles](docs/adr/adr00_guiding_principles.md)
- [ADR01: TDD and Unit Size Structure](docs/adr/adr01_tdd_and_unit_size_structure.md)
- [ADR02: MIDI Library Selection](docs/adr/adr02_midi_library_selection.md)
- [ADR03: Concurrency Model](docs/adr/adr03_structure_concurrency_and_instantiation.md)

## License 📄

GNU Affero General Public License v3 - See [LICENSE](LICENSE) for details