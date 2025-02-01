# PhasorsyncRS Technical Debt Inventory

Prioritized list of architectural improvements needed to align with ADRs:

## 🚩 P0 - Critical Path Blockers

### 1. Incomplete MIDI Engine Implementation
- **ADR Reference**: [ADR02](docs/adr/adr02_midi_library_selection.md)
- **Current State**:
  - No `MidiEngine` trait implementation
  - ALSA integration limited to device listing
  - Missing proper message queue abstraction
- **Impact**: Cannot process real MIDI IO
- **Action**:
  ```rust
  // Pseudo-code for missing components
  pub trait MidiEngine {
      fn send(&mut self, msg: MidiMessage) -> Result<()>;
      fn recv(&mut self) -> Result<MidiMessage>;
      fn list_devices(&self) -> Vec<String>;
  }
  
  struct AlsaMidiEngine { /* ALSA implementation */ }
  struct MockMidiEngine { /* Test implementation */ }
  ```

### 2. Partial Concurrency Model
- **ADR Reference**: [ADR03](docs/adr/adr03_structure_concurrency_and_instantiation.md)
- **Current State**:
  - Shared state exists but no processing thread
  - No channel-based communication between components
- **Impact**: Cannot meet real-time requirements
- **Action**:
  ```rust
  // Suggested threading structure
  fn main() {
      let (cmd_tx, cmd_rx) = crossbeam_channel::bounded(100);
      let (midi_tx, midi_rx) = crossbeam_channel::bounded(1000);
      
      thread::spawn(|| engine_processing_loop(cmd_rx, midi_tx));
      thread::spawn(|| cli_monitor_loop(midi_rx));
  }
  ```

## ⚠️ P1 - Architectural Integrity Risks

### 3. Test Strategy Gaps
- **ADR Reference**: [ADR01](docs/adr/adr01_tdd_and_unit_size_structure.md)
- **Current State**:
  - Basic test files exist but lack:
    - Property-based tests
    - Fuzzy testing
    - Real-time simulation
- **Impact**: Hard to maintain tempo-sensitive logic
- **Action**:
  ```rust
  // Example test enhancement
  #[test]
  fn tempo_calculation_under_load() {
      let engine = TestMidiEngine::with_latency(50ms);
      // Validate BPM accuracy with simulated jitter
  }
  ```

## 📝 P2 - Documentation Debt

### 4. Missing Architectural Overview
- **Current State**:
  - ADRs exist in isolation
  - No component interaction diagrams
  - No data flow documentation
- **Impact**: Hard to onboard new developers
- **Action**:
  ```markdown
  ## Architectural Components
  
  ```plantuml
  [MIDI Hardware] <- [ALSA Driver] <-> [MidiEngine] -> [Processing Thread]
  ```
  ```

### 5. Incomplete Module Docs
- **Current State**:
  - Minimal rustdoc comments
  - No error handling guidelines
- **Impact**: Hard to maintain safety guarantees
- **Action**:
  ```rust
  /// # Safety
  /// Requires exclusive access to ALSA sequencer
  unsafe fn claim_sequencer() { ... }
  ```

## Prioritization Summary

| Level | Debt Item                      | Owner  | Target Sprint |
|-------|--------------------------------|--------|---------------|
| P0    | MIDI Engine Implementation     | Core   | Sprint 1      |
| P0    | Concurrency Model              | Core   | Sprint 1      |
| P1    | Test Strategy                  | QA     | Sprint 2      |
| P2    | Documentation                  | Docs   | Sprint 3      |

## New Refactoring Prompts

### 🔄 MIDI Module Refactoring Plan

1. **Create `midi/engine` module**
```rust
// Prompt: Create midi/engine.rs with MidiEngine trait per ADR02
pub trait MidiEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()>;
    fn recv(&mut self) -> Result<MidiMessage>;
    fn list_devices(&self) -> Vec<String>;
}
```

2. **Implement ALSA backend**
```rust
// Prompt: Create midi/backend/alsa.rs with AlsaEngine struct
struct AlsaEngine {
    seq: alsa::seq::Seq,
    // ...
}

impl MidiEngine for AlsaEngine { /* ADR02-compliant impl */ }
```

3. **Extract Clock Generator**
```rust
// Prompt: Move ClockGenerator to midi/clock.rs with focused API
pub struct TempoController {
    bpm: Arc<AtomicF64>,
    thread_handle: JoinHandle<()>,
}

impl TempoController {
    pub fn adjust_bpm(&self, new_bpm: f64) { /* ... */ }
}
```

4. **Create Device Manager**
```rust
// Prompt: Extract device listing to midi/devices.rs
pub struct DeviceManager {
    engine: Box<dyn MidiEngine>,
}

impl DeviceManager {
    pub fn refresh_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        self.engine.list_devices()
    }
}
```

5. **Error Handling Unification**
```rust
// Prompt: Create midi/error.rs with unified error type
#[derive(Debug, thiserror::Error)]
pub enum MidiError {
    #[error("ALSA error: {0}")]
    Alsa(#[from] alsa::Error),
    #[error("Threading error")]
    Threading,
    #[error("Invalid state: {0}")]
    State(String),
}
```

6. **Update Root Module**
```rust
// Prompt: Update midi.rs to re-export modules
pub mod engine;
pub mod clock;
pub mod devices;
pub mod error;

pub use engine::{MidiEngine, AlsaEngine};
pub use clock::TempoController;