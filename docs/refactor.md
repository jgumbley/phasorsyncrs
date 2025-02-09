# MIDI Clock Subsystem Refactor (Revised Minimal Plan)

## Current Progress Status
✅ Completed Core Structure:
```rust
src/midi/clock/core.rs
src/midi/clock/mod.rs
```

🚧 Remaining Critical Path:
1. Remove duplicate transport logic from:
   - `src/midi/internal_clock.rs` (lines 45-78)
   - `src/midi/external_clock.rs` (lines 32-65)
2. Centralize BPM calculation in `ClockCore`

## Revised Implementation Steps

### Phase 1a: Transport Handler Finalization (1-2 hours)
```rust
// Remove from internal_clock.rs
- fn handle_transport_message(&mut self, msg: ClockMessage) {
-     // Duplicated logic
- }

// Remove from external_clock.rs  
- fn process_transport_state(&mut self) {
-     // Duplicated logic
- }
```

### Phase 1b: BPM Unification (2-3 hours)
```rust
// In core.rs
pub fn calculate_bpm(&mut self) -> Option<f64> {
    // Consolidated calculation
}

// Remove from:
- internal_clock.rs: calculate_timing()
- external_clock.rs: estimate_external_bpm()
```

### Phase 2: Safe Trait Migration
```rust
// Transitional implementation
trait LegacyMidiClock {
    // Existing interface
}

impl LegacyMidiClock for InternalClock {
    // Delegate to ClockCore
    fn is_playing(&self) -> bool {
        self.core.lock().unwrap().is_playing()
    }
}
```

## Validation Protocol
1. Run after each phase:
```bash
cargo test --test midi_engine_tests
cargo test --test scheduler_tests
```

2. Manual verification steps:
```bash
# Start internal clock
cargo run -- internal-clock --bpm 120
# Send external MIDI clock messages
midi-send clock-start
```

## Rollback Plan
Revert to tag `pre-clock-refactor` if tests fail:
```bash
git checkout pre-clock-refactor -- src/midi/clock/
```

## Estimated Completion
3-4 hours of focused work vs original 2-day estimate