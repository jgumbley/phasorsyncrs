# MIDI Clock Subsystem Refactor

## Objective
Merge internal/external clock implementations by:
1. Moving tick generation to engine implementations
2. Creating unified clock message handling
3. Removing redundant timing systems

## Required Changes

### 1. Engine Timing Implementation (InternalEngine)
**File:** `src/midi/internal_engine.rs`
```rust
// Add to InternalEngine implementation:
fn start_tick_generator(&self, bpm: f32) -> thread::JoinHandle<()> {
    let core = self.core.clone();
    thread::spawn(move || {
        let tick_duration = Duration::from_secs_f32(60.0 / (bpm * TICKS_PER_BEAT as f32));
        loop {
            thread::sleep(tick_duration);
            if let Ok(mut guard) = core.lock() {
                guard.process_message(ClockMessage::Tick);
            }
        }
    })
}
```

### 2. Clock Implementation Unification
**File:** `src/midi/clock/core.rs`
```rust
// Update ClockCore to handle generic messages:
pub enum ClockMessage {
    Tick,
    Start,
    Stop,
    Continue,
    // Remove engine-specific variants
}

// Remove ClockGenerator trait
```

### 3. InternalClock Simplification
**File:** `src/midi/internal_clock.rs`
```rust
// Remove ClockGenerator implementation
// Keep only message handling matching ExternalClock's pattern
```

### 4. Remove Redundant Timing System 
**File:** `src/transport/timing.rs`
```rust
// Delete this file entirely
```

### 5. Update Dependencies
**File:** `src/main.rs**
```rust
// Replace timing::run_timing_simulation with:
let engine = InternalEngine::new(shared_state.clone());
let _tick_thread = engine.start_tick_generator(120.0);
```

## Migration Checklist
1. [ ] Implement engine-based tick generation
2. [ ] Remove ClockGenerator trait and implementations
3. [ ] Delete transport/timing.rs
4. [ ] Update main application initialization
5. [ ] Verify all MIDI clock tests pass
6. [ ] Update documentation in docs/adr/
