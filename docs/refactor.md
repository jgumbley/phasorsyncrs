# MIDI Message Generation Refactor (Target: v0.2.1)

## Objective
Move internal clock message generation into MidirEngine while maintaining testability

## Critical Path (1h-2h)
```rust
// STEP 1: Add message source to MidirEngine (src/midi/midir_engine.rs)
+struct InternalMessageThread {
+    core: Arc<Mutex<ClockCore>>,
+}

// STEP 2: Move thread spawn from InternalClock (src/midi/internal_clock.rs L45-78)
- thread::spawn(move || {
-     // Clock generation loop
- });
+impl MidirEngine {
+    fn start_internal_clock(&mut self, core: Arc<Mutex<ClockCore>>) {
+        let mut thread = InternalMessageThread { core };
+        thread.spawn();
+    }
+}

// STEP 3: Update InternalClock constructor (src/midi/internal_clock.rs)
- pub fn new(bpm: f64) -> Self {
-     let core = ClockCore::new(bpm);
-     // Spawn thread here
- }
+ pub fn new(bpm: f64, engine: &mut impl MidiEngine) -> Self {
+     let core = ClockCore::new(bpm);
+     engine.start_internal_clock(core.clone());
+ }
```

## Validation Protocol
```bash
# Check thread management moved
rg "thread::spawn" src/midi/internal_clock.rs

# Run core functionality tests
cargo test --test midi_tests -- --test-threads=1

# Verify engine initialization 
cargo run --example clock_demo
```

## Rollback Plan
```bash
git checkout HEAD -- src/midi/midir_engine.rs src/midi/internal_clock.rs
```

## Post-Refactor Structure
```
MidirEngine
├── Device management
└── InternalMessageThread
    └── Clock generation loop

InternalClock
└── Configuration