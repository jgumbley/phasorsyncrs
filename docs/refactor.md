# MIDI Module Refactoring Guide

## Objective
Create a unified timing architecture where:
- Clock implementations handle timing source specifics
- Engines focus on MIDI message transport
- Main construction becomes declarative

## Phase 1: Core Interface Definition

### 1.1 Create Clock Trait
```rust
// src/midi/clock/mod.rs
pub trait Clock: Send + Sync {
    fn start(&mut self);
    fn stop(&mut self);
    fn handle_message(&mut self, msg: ClockMessage);
    fn core(&self) -> &Arc<Mutex<ClockCore>>;
}
```

### 1.2 Implement for Existing Clocks
```rust
// src/midi/internal_clock.rs
impl Clock for InternalClock {
    // Existing implementations match trait requirements
}

// src/midi/external_clock.rs 
impl Clock for ExternalClock {
    // Existing implementations match trait requirements
}
```

## Phase 2: Engine Adapters

### 2.1 Create Engine Trait
```rust
// src/midi/engine.rs
pub trait MidiEngine {
    fn send(&mut self, msg: MidiMessage) -> Result<()>;
    fn recv(&self) -> Result<MidiMessage>;
    fn into_clock(self, shared_state: SharedState) -> Box<dyn Clock>;
}
```

### 2.2 Implement Engine Conversions
```rust
// src/midi/internal_engine.rs
impl MidiEngine for InternalEngine {
    fn into_clock(self, shared_state: SharedState) -> Box<dyn Clock> {
        Box::new(InternalClock::new(shared_state))
    }
}

// src/midi/midir_engine.rs
impl MidiEngine for MidirEngine {
    fn into_clock(self, shared_state: SharedState) -> Box<dyn Clock> {
        Box::new(ExternalClock::new(shared_state))
    }
}
```

## Phase 3: Unified Construction

### 3.1 Update Main Initialization
```rust
// src/main.rs
fn initialize_system(engine: impl MidiEngine, shared_state: &SharedState) {
    let clock = engine.into_clock(shared_state.clone());
    clock.start();
    
    // Common initialization continues...
}
```

### 3.2 Remove Conditional Paths
Delete:
- Separate external_clock/internal_clock initialization blocks
- Duplicated state handling code

## Phase 4: Message Flow Update

### 4.1 Centralize Message Conversion
```rust
// src/midi/clock/core.rs
pub fn convert_message(
    engine_type: EngineType,
    msg: MidiMessage
) -> Option<ClockMessage> {
    match engine_type {
        EngineType::Internal => InternalConverter::convert(msg),
        EngineType::Midi => MidirConverter::convert(msg),
    }
}
```

## Verification Steps

1. Clock Symmetry Test
```rust
fn test_clock_interface_symmetry() {
    let mut clocks: Vec<Box<dyn Clock>> = vec![
        Box::new(InternalClock::mock()),
        Box::new(ExternalClock::mock())
    ];
    
    // Verify common interface functionality
}
```

2. Engine Conversion Test
```rust
fn test_engine_conversion() {
    let internal = InternalEngine::new();
    let midir = MidirEngine::new();
    
    assert!(internal.into_clock().is::<InternalClock>());
    assert!(midir.into_clock().is::<ExternalClock>());
}
```

3. Construction Sanity Check
```rust
fn test_unified_construction() {
    let engine = select_engine_based_on_config();
    initialize_system(engine);
    // Verify operational state
}
```

## Migration Order
1. Trait definitions (Clock + MidiEngine)
2. Engine conversion implementations
3. Main.rs refactoring
4. Delete old initialization paths
5. Update documentation