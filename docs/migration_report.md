# Midimoke->PhasorSyncRS Rust Migration Report

## Overview
This report outlines the key considerations for migrating the Python implementation of Midimoke to Rust. It identifies constructive patterns to preserve, critical errors to avoid, and areas requiring clarification.

## Constructive Patterns to Preserve

### Event Hierarchy Architecture
The Python implementation's event system uses class inheritance with custom sorting logic. In Rust, this can be mapped to an enum-based event system with trait implementations for ordering.

```rust
pub enum MidiEvent {
    NoteOn { tick: u64, pitch: u8, channel: u8 },
    NoteOff { tick: u64, pitch: u8, channel: u8 },
    ControlChange { tick: u64, controller: u8, value: u8 }
}

impl Ord for MidiEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Implementation would handle ordering based on tick, type, and insertion order
        unimplemented!()
    }
}
```

### Immutable Pattern Core
Maintain Python's immutability paradigm using Rust's `Arc` for thread-safe sharing.

```rust
#[derive(Clone)]
pub struct Pattern {
    events: Arc<[MidiEvent]>,
    tick_length: u64
}
```

### MIDI Clock Synchronization
Preserve Python's moving average approach but with thread-safe atomics.

```rust
struct MidiSync {
    clock_samples: ArrayDeque<Duration, 24>, // Fixed-size buffer
    bpm: AtomicF64,
    running: AtomicBool
}

impl MidiSync {
    fn handle_clock(&mut self) {
        // Implementation would update BPM based on clock samples
        unimplemented!()
    }
}
```

## Critical Errors to Avoid

### Blocking I/O in Callbacks
Use async I/O with backpressure instead of blocking calls.

```rust
// BAD: Direct MIDI send in handler
fn handle_note(&self) {
    self.port.send(&msg).unwrap(); // May block
}

// GOOD: Use bounded channel
async fn midi_sender(rx: Receiver<MidiMessage>) {
    while let Some(msg) = rx.recv().await {
        port.send(msg).unwrap();
    }
}
```

### Recursive Playback Stack
Replace recursive calls with iterative loops.

```rust
// Safe Rust alternative
fn playback_loop(sender: mpsc::Sender<MidiEvent>) {
    let mut current_tick = 0;
    loop {
        let notes = movement.next(current_tick);
        sender.send(notes).unwrap();
        thread::sleep(calculate_wait(current_tick));
        current_tick += tick_delta;
    }
}
```

### Implicit Event Equality
Enforce explicit field comparisons through derive macros.

```rust
#[derive(PartialEq, Eq, Hash)]
struct NoteEvent {
    tick: u64,
    pitch: u8,
    channel: u8,
    insertion_order: u64
}
```

## Functional Clarifications Needed

### Realtime Guarantees
Determine if kernel-level realtime priority is required.

```rust
#[cfg(target_os = "linux")]
fn set_realtime_priority() {
    let param = libc::sched_param {
        sched_priority: 99
    };
    unsafe {
        libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
    }
}
```

### Error Recovery Strategy
Define automatic reconnect policies and buffer management.

```rust
enum MidiError {
    PortDisconnected,
    BufferOverrun { dropped_events: usize },
    ClockDrift { delta_ms: f64 }
}

impl RecoveryPolicy {
    fn handle_error(&self, error: MidiError) {
        match error {
            MidiError::PortDisconnected => self.reconnect(),
            // ... other recovery paths
        }
    }
}
```

### Pattern Mutation Rules
Formalize mutation rules and DSL format.

```rust
trait PatternMutation {
    fn mutate(&self, rng: &mut ThreadRng, ctx: &MutationContext) -> Pattern;
}

struct MarkovMutation {
    order: usize,
    transition_matrix: HashMap<NotePair, f64>
}
```

## Next Steps
1. Establish realtime requirements (Linux RT kernel vs userspace)
2. Define MIDI recovery policy thresholds
3. Finalize mutation rule DSL format

## Testing
To test the prototype:

```bash
cargo run --features rtmidi --example sync-playback